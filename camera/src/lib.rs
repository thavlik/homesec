#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate crossbeam;
#[macro_use]
extern crate pyo3;
use pyo3::prelude::*;
use pyo3::exceptions::RuntimeError;
use pyo3::{PyErr, types::{PyString, PyBytes}};
use crossbeam::channel::{Sender, Receiver};
use std::{ptr, ffi::{c_void, CStr}};
use std::path::PathBuf;
use std::os::raw::c_char;
use anyhow::{Result, Error};
use futures::StreamExt;
use std::{net::SocketAddr, sync::{Arc, Weak, Mutex}};
use quinn::{ClientConfig, ClientConfigBuilder};

#[cfg(test)]
mod test;

lazy_static! {
    static ref RUNTIME: Arc<Mutex<tokio::runtime::Runtime>> = Arc::new(Mutex::new(tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap()));
}

/// Dummy certificate verifier that treats any certificate as valid.
/// NOTE, such verification is vulnerable to MITM attacks, but convenient for testing.
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        _presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}

async fn connect(server_addr: SocketAddr) -> Result<(quinn::Endpoint, quinn::Connection)> {
    warn!("configuring client");
    let client_cfg = configure_client();

    warn!("building endpoint...");
    let mut endpoint_builder = quinn::Endpoint::builder();
    endpoint_builder.default_client_config(client_cfg);

    let addr = "127.0.0.1:0".parse()?;
    warn!("binding endpoint {}", &addr);
    let (endpoint, _) = endpoint_builder.bind(&addr)?;

    warn!("connecting to server...");
    let quinn::NewConnection { connection, .. } = endpoint
        .connect(&server_addr, "localhost")?
        .await?;

    warn!("[client] connected: addr={}", connection.remote_address());

    Ok((endpoint, connection))
}

fn configure_client() -> ClientConfig {
    let mut cfg = ClientConfigBuilder::default().build();
    let tls_cfg: &mut rustls::ClientConfig = Arc::get_mut(&mut cfg.crypto).unwrap();
    // this is only available when compiled with "dangerous_configuration" feature
    tls_cfg
        .dangerous()
        .set_certificate_verifier(SkipServerVerification::new());
    cfg
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Frame {
    pixel_data: Vec<u8>,
}

pub struct Output {
    conn: quinn::Connection,
    endpoint: quinn::Endpoint,
    stop_recv: Receiver<()>,
}

struct StreamInner {
    width: usize,
    height: usize,
    outputs: Mutex<Vec<Arc<Output>>>,
}

#[pyclass]
pub struct Stream {
    stop: Sender<()>,
    inner: Arc<StreamInner>,
}

#[pymethods]
impl Stream {
    #[new]
    fn new(width: usize, height: usize, dest: &str) -> PyResult<Self> {
        let (ready_send, ready_recv) = crossbeam::channel::bounded(1);
        let (stop_send, stop_recv) = crossbeam::channel::bounded(1);
        let inner = Arc::new(StreamInner{
            width,
            height,
            outputs: Mutex::new(Vec::new()),
        });
        let _inner = inner.clone();
        RUNTIME.clone()
            .lock()
            .unwrap()
            .block_on(async move {
                ready_send.send(open_stream(_inner, dest, stop_recv).await);
            });
        match ready_recv.recv() {
            Ok(result) => {
                if let Err(e) = result {
                    return Err(PyErr::new::<RuntimeError, _>(e.to_string()));
                }
                Ok(Stream{
                    inner,
                    stop: stop_send,
                })
            },
            Err(e) => {
                Err(PyErr::new::<RuntimeError, _>(e.to_string()))
            }
        }
    }
    
    fn send_frame(&mut self, data: &[u8]) {
        let payload = bytes::Bytes::from(bincode::serialize(&Frame{
            pixel_data: Vec::from(data),
        }).expect("serialize frame"));
        let outputs = self.inner.outputs
            .lock()
            .unwrap()
            .clone();
        for output in outputs.iter() {
            match output.conn.send_datagram(payload.clone()) {
                Err(e) => {
                    error!("send_datagram: {}", e);
                },
                _ => {},
            }
        }
    }
}

impl std::ops::Drop for Stream {
    fn drop(&mut self) {
        if let Err(e) = self.stop.send(()) {
            error!("error stopping stream: {}", e);
        }
    }
}

async fn open_stream(inner: Arc<StreamInner>, dest: &str, stop_recv: Receiver<()>) -> Result<()> {
    let server_addr: SocketAddr = dest.parse()?;
    loop {
        if let Ok(_) = stop_recv.try_recv() {
            // Cancel the reconnect attempt if user requests shutdown
            return Ok(());
        }
        match connect(server_addr.clone()).await {
            Ok((endpoint, conn)) => {
                inner.outputs.lock().unwrap().push(Arc::new(Output{
                    endpoint,
                    conn,
                    stop_recv,
                }));
                return Ok(());
            },
            Err(e) => {
                error!("error connecting to {}: {}", &server_addr, e);
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    }
    Ok(())
}

#[pymodule]
fn stream(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Stream>()
}

// TODO: refactor non-python code into module so cargo test will works again
// TODO: refactor Stream into mod
// TODO: write test code
