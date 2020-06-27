use std::net::SocketAddr;
use anyhow::Result;
use std::process::Command;
use std::io::{self, Write};

fn build_for_arm() -> Result<()> {
    let output = Command::new("cargo")
        .args(&["build", "--target", "armv7-unknown-linux-gnueabihf"])
        .current_dir("../bootstrap")
        .output()
        .expect("build failed");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    Ok(())
}

fn get_addresses() -> Result<Vec<SocketAddr>> {
    Ok(vec!["192.168.0.102".parse().unwrap()])
}

fn basic_daemon_install() -> Result<()> {
    let addresses = get_addresses()?;
    Ok(())
}

fn main() {
    build_for_arm().unwrap();
}
