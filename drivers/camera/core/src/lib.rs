#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate anyhow;
use std::ffi::{CString, CStr, c_void};
use std::{sync::{Arc, Mutex}, net::SocketAddr};
use std::os::raw::c_char;
use rav1e::{Context, EncoderConfig, Config, config::SpeedSettings};
use anyhow::{Result, Error};
use quinn::{Connection, Endpoint, ClientConfig, ClientConfigBuilder};

lazy_static! {
    static ref RUNTIME: Arc<Mutex<tokio::runtime::Runtime>> = Arc::new(Mutex::new(tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap()));
}

pub struct Service {
    width: usize,
    height: usize,
    ctx: Context<u16>,
    conn: Connection,
    endpoint: Endpoint,
}

impl Service {
    pub fn new(width: usize, height: usize, endpoint: Endpoint, conn: Connection) -> Self {
        let mut enc = EncoderConfig::default();
        enc.width = width;
        enc.height = height;
        enc.speed_settings = SpeedSettings::from_preset(9);
        let cfg = Config::new().with_encoder_config(enc);
        let ctx: Context<u16> = cfg.new_context().unwrap();
        Self {
            width,
            height,
            ctx,
            endpoint,
            conn,
        }
    }

    pub fn send_frame(&self, data: &[u8]) {
        //self.ctx.send_frame()
    }
}

#[no_mangle]
pub extern fn new_service(width: u32, height: u32, endpoint: *const c_char) -> *mut Service {
    let endpoint = unsafe { CStr::from_ptr(endpoint) }.to_str().unwrap();
    let endpoint: SocketAddr = endpoint.parse().unwrap();
    let result = RUNTIME.clone()
        .lock()
        .unwrap()
        .block_on(connect(endpoint));
    let (endpoint, conn) = result.unwrap();
    let svc = Service::new(width as _, height as _, endpoint, conn);
    Box::into_raw(Box::new(svc))
}

#[no_mangle]
pub extern fn free_service(svc: *mut Service) {
    unsafe {
        Box::from_raw(svc);
    }
}

#[no_mangle]
pub extern fn send_frame(svc: *mut Service, data: *const u8) {
    let svc = unsafe { &mut *svc };
    let data = unsafe { std::slice::from_raw_parts(data, svc.width * svc.height * 3) };
    svc.send_frame(data);
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