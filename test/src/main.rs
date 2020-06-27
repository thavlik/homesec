use std::net::SocketAddr;
use anyhow::Result;

fn get_addresses() -> Result<Vec<SocketAddr>> {
    Ok(vec!["192.168.0.102".parse().unwrap()])
}

fn basic_daemon_install() -> Result<()> {
    let addresses = get_addresses()?;
    Ok(())
}

fn main() {

}