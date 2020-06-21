use anyhow::{anyhow, Error, Result, Context, bail};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::{self, Path, PathBuf};
use std::process::Command;
use quinn::{
    ServerConfig,
    ServerConfigBuilder,
    TransportConfig,
    CertificateChain,
    PrivateKey,
    Certificate,
};
use futures::future::{Abortable, AbortHandle, Aborted};
use std::{
    ascii,
    io,
    str,
    net::SocketAddr,
    sync::{Arc, mpsc, Mutex, atomic::{AtomicU64, Ordering}},
    fs,
};
use crossbeam::channel::{Sender, Receiver};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, SystemTime};
use futures::{StreamExt, TryFutureExt};

#[allow(unused)]
pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-27"];

#[cfg(test)]
mod test {
    use super::*;

    async fn basic_stream_server(
        addr: SocketAddr,
        mut send_conn: Sender<()>,
        send_data: Arc<Mutex<(Sender<()>, bool)>>,
    ) -> Result<()> {
        let mut transport_config = TransportConfig::default();
        transport_config.stream_window_uni(0);
        let mut server_config = ServerConfig::default();
        server_config.transport = Arc::new(transport_config);
        let mut server_config = ServerConfigBuilder::new(server_config);
        server_config.protocols(ALPN_QUIC_HTTP);
        let dirs = directories::ProjectDirs::from("org", "quinn", "quinn-examples").unwrap();
        let path = dirs.data_local_dir();
        let cert_path = path.join("cert.der");
        let key_path = path.join("key.der");
        let (cert, key): (Vec<u8>, Vec<u8>) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
            Ok(x) => x,
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
                let key = cert.serialize_private_key_der();
                let cert = cert.serialize_der().unwrap();
                fs::create_dir_all(&path).context("failed to create certificate directory")?;
                fs::write(&cert_path, &cert).context("failed to write certificate")?;
                fs::write(&key_path, &key).context("failed to write private key")?;
                (cert, key)
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        let key = PrivateKey::from_der(&key)?;
        let cert = Certificate::from_der(&cert)?;
        server_config.certificate(CertificateChain::from_certs(vec![cert]), key)?;
        let mut endpoint = quinn::Endpoint::builder();
        endpoint.listen(server_config.build());
        let mut incoming = {
            let (endpoint, incoming) = endpoint.bind(&addr)?;
            incoming
        };
        while let Some(conn) = incoming.next().await {
            let quinn::NewConnection {
                connection,
                mut datagrams,
                ..
            } = conn.await.expect("failed to accept incoming connection");
            send_conn.send(())?;
            while let Some(data) = datagrams.next().await {
                //let frame: Frame = bincode::deserialize(data?.as_ref())?;
                //let mut send_data = send_data.lock().unwrap();
                //if let (s, true) = &*send_data {
                //    s.send(())?;
                //    send_data.1 = false;
                //}
            }
        }
        Ok(())
    }

    #[tokio::test(threaded_scheduler)]
    async fn test_basic_stream() {
        let port = portpicker::pick_unused_port().expect("pick port");
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        let (mut send_conn, recv_conn) = crossbeam::channel::unbounded();
        let (mut send_data, recv_data) = crossbeam::channel::unbounded();
        let send_data = Arc::new(Mutex::new((send_data, true)));
        let _send_data = send_data.clone();
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let future = Abortable::new(async move {
            basic_stream_server(addr, send_conn, _send_data).await
        }, abort_registration);
    }
}
