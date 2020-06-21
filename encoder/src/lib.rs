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
//use crossbeam::channel::{Sender, Receiver};
use std::{ptr, ffi::{c_void}};
//use std::path::PathBuf;
//use std::os::raw::c_char;
//use anyhow::{Result, Error};
//use paradise_core::{Frame, device::{DeviceSpec, Endpoint}};
//use futures::StreamExt;
//use std::{net::SocketAddr, sync::{Arc, Weak, Mutex}};
//use quinn::{ClientConfig, ClientConfigBuilder};

/// Dummy certificate verifier that treats any certificate as valid.
/// NOTE, such verification is vulnerable to MITM attacks, but convenient for testing.
struct SkipServerVerification;

//mod stream;
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

lazy_static! {
    static ref RUNTIME: Arc<Mutex<tokio::runtime::Runtime>> = Arc::new(Mutex::new(tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap()));
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

    fn add_output(&self, output: Output) -> Result<()> {
        let mut outputs = self.outputs.lock().unwrap();
        if let Some(_) = outputs.iter().find(|o| o.spec.name == output.spec.name) {
            return Err(Error::msg(format!("an output with the name '{}' already exists", output.spec.name)));
        }
        warn!("Adding output '{}' ({}, insecure={})",
              &output.spec.name,
              &output.spec.addr,
              &output.spec.insecure);
        outputs.push(output);
        warn!("{} total outputs", outputs.len());
        Ok(())
    }

    fn io_proc(&self, buffer: &[u8], sample_time: f64) -> Result<()> {
        let payload = bytes::Bytes::from(bincode::serialize(&Frame{
            buffer: Vec::from(buffer),
            sample_time,
        })?);
        let outputs = match self.outputs.try_lock() {
            Ok(l) => l,
            Err(e) => return Err(anyhow!("{:?}", e)),
        };
        for output in &*outputs {
            match output.conn.send_datagram(payload.clone()) {
                Ok(()) => {}
                Err(e) => {
                    error!("failed to send datagram to output '{}': {}", &output.spec.name, e);
                }
            }
        }
        Ok(())
    }

    fn stop(&self) {
        // TODO: wait for stoppage
        self.stop.lock()
            .unwrap()
            .send(())
            .unwrap();
    }
}

#[no_mangle]
pub extern "C" fn rust_io_proc(driver: *const c_void, buffer: *const u8, buffer_size: u32, sample_time: f64) {
    let driver: Arc<Driver> = match unsafe {
        Weak::from_raw(driver as _)
    }.upgrade() {
        Some(driver) => driver,
        None => {
            error!("ioproc: driver is deallocated");
            return;
        }
    };
    let buffer = unsafe {
        std::slice::from_raw_parts(buffer, buffer_size as _)
    };
    match driver.io_proc(buffer, sample_time) {
        Err(e) => {
            error!("ioproc: {:?}", e)
        }
        _ => {}
    }
}
 */

struct Encoder {
    width: i32,
    height: i32,
}

#[no_mangle]
pub extern "C" fn new_encoder(width: i32, height: i32) -> *mut c_void {
    Box::into_raw(Box::new(Encoder{width, height})) as _
}

#[no_mangle]
pub extern "C" fn free_encoder(encoder: *mut c_void) {
    unsafe {
        Box::from_raw(encoder as *mut Encoder);
    }
}

#[no_mangle]
pub extern "C" fn encode_frame(encoder: *mut c_void, frame: *const c_void) -> i32 {
    let encoder: &mut Encoder = unsafe { &mut *(encoder as *mut Encoder) };
    0
}

#[no_mangle]
pub extern "C" fn mul(x: i32, y: i32) -> i32 {
    x * y
}



