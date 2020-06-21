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

mod stream;
/*

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

fn init_logger() -> Result<()> {
    std::panic::set_hook(Box::new(|panic_info| {
        let location = if let Some(location) = panic_info.location() {
            format!("{}", location)
        } else {
            format!("unknown")
        };
        let message = if let Some(message) = panic_info.message() {
            format!("{}", message)
        } else {
            format!("(no message available)")
        };
        error!("panic occurred [{}]: {}", location, message);
    }));
    #[cfg(target_os = "macos")]
        {
            use syslog::{Facility, Formatter3164, BasicLogger};
            use log::{SetLoggerError, LevelFilter};
            let formatter = Formatter3164 {
                facility: Facility::LOG_USER,
                hostname: None,
                process: "proxyaudio".into(),
                pid: std::process::id() as _,
            };
            let mut writer = match syslog::unix(formatter) {
                Ok(writer) => writer,
                Err(e) => return Err(Error::msg(format!("{:?}", e))),
            };
            log::set_boxed_logger(Box::new(BasicLogger::new(writer)))
                .map(|()| log::set_max_level(LevelFilter::max()));
        }
    Ok(())
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

async fn driver_entry(driver: Arc<Driver>, stop: Receiver<()>) -> Result<()> {
    if driver.spec.endpoints.len() == 0 {
        return Err(Error::msg("no endpoints"));
    }
    for endpoint in &driver.spec.endpoints {
        let driver = driver.clone();
        let endpoint = endpoint.clone();
        tokio::spawn(async move {
            driver.connect_with_retry(endpoint).await;
        });
    }
    Ok(())
}

pub struct Output {
    pub spec: Endpoint,
    pub conn: quinn::Connection,
    pub endpoint: quinn::Endpoint,
}

pub struct Driver {
    // TODO: spec for inputs and outputs
    outputs: Mutex<Vec<Output>>,
    spec: DeviceSpec,
    stop: Mutex<Sender<()>>,
}

impl Driver {
    /// "Fire and forget" connect method
    async fn connect_with_retry(&self, endpoint: Endpoint) {
        warn!("connecting to '{}' ({}, insecure={})",
              &endpoint.name,
              &endpoint.addr,
              &endpoint.insecure);
        let server_addr: SocketAddr = match endpoint.addr.parse() {
            Ok(v) => v,
            Err(e) => {
                error!("error parsing addr '{}' for endpoint '{}': {}", &endpoint.addr, &endpoint.name, e);
                return;
            }
        };
        loop {
            match connect(server_addr.clone()).await {
                Ok((e, conn)) => {
                    match self.add_output(Output {
                        spec: endpoint,
                        endpoint: e,
                        conn,
                    }) {
                        Err(e) => {
                            error!("failed to add output: {}", e);
                        },
                        _ => {},
                    }
                    return;
                },
                Err(e) => {
                    error!("error connecting to {}: {}", &server_addr, e);
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            }
        }
    }
}
 */

#[pyclass]
#[derive(Default)]
pub struct Stream {
    width: i32,
    height: i32,
    dest: String,
}

async fn stream_entry(stop_recv: Receiver<()>) -> Result<()> {
    Ok(())
}

#[pymethods]
impl Stream {
    #[new]
    fn new(width: usize, height: usize, dest: &str) -> PyResult<Self> {
        let (ready_send, ready_recv) = crossbeam::channel::bounded(1);
        let (stop_send, stop_recv) = crossbeam::channel::bounded(1);
        RUNTIME.clone()
            .lock()
            .unwrap()
            .block_on(async move {
                ready_send.send(stream_entry(stop_recv).await);
            });
        match ready_recv.recv() {
            Ok(result) => match result {
                Ok(()) => {
                    //warn!("device has signaled ready state");
                    Ok(Stream::default())
                },
                Err(e) => {
                    //error!("failed to initialize: {:?}", e);
                    //Ok(Stream::default())
                    Err(PyErr::new::<RuntimeError, _>(e.to_string()))
                }
            },
            Err(e) => {
                //error!("ready channel error: {:?}", e);
                //Ok(Stream::default())
                Err(PyErr::new::<RuntimeError, _>(e.to_string()))
            }
        }
    }
    
    fn send_frame(&mut self, py: Python, data: &[u8]) {

    }
}

impl Stream {
    // "Fire and forget" connect script
    async fn connect_with_retry(&self, addr: &str) {
        //let server_addr: SocketAddr = match addr.parse() {
        //    Ok(v) => v,
        //    Err(e) => {
        //    }
        //};
        loop {
            //match connect(server_addr.clone()).await {
            //    Ok((e, conn)) => {
            //    },
            //    Err(e) => {
            //        error!("error connecting to {}: {}", &server_addr, e);
            //        std::thread::sleep(std::time::Duration::from_secs(5));
            //    }
            //}
        }
    }
}

#[pymodule]
fn stream(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Stream>()
}




