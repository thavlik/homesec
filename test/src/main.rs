#[macro_use]
extern crate anyhow;

use anyhow::{Error, Result};
use std::net::{UdpSocket, SocketAddr};
use std::process::Command;
use std::io::{self, Write};
use std::time::{SystemTime, Duration};

use homesec_bootstrap::{Message, AppearanceMessage};

const BUFFER_SIZE: usize = 8192;

fn build_for_arm() -> Result<()> {
    let output = Command::new("sh")
        .args(&[
            "-c",
            "rm ../target/armv7-unknown-linux-gnueabihf/debug/homesec_bootstrap || true && cargo build --target armv7-unknown-linux-gnueabihf",
        ])
        .current_dir("../bootstrap")
        .output()
        .expect("build failed");
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        Err(anyhow!("command failed with exit code {}", output.status))
    } else {
        println!("successfully built bootstrap");
        Ok(())
    }
}

fn get_addresses() -> Result<Vec<String>> {
    Ok(vec![
        String::from("192.168.0.100"),
        String::from("192.168.0.102"),
        String::from("192.168.0.103"),
    ])
}

fn install_bootstrap(addresses: &[String]) -> Result<()> {
    for address in addresses {
        println!("installing to {}", address);
        let spec = include_str!("../../bootstrap/extra/homesec-bootstrap.service");
        let encoded = base64::encode(format!("echo \"{}\" | base64 --decode > /etc/systemd/system/homesec-bootstrap.service", base64::encode(spec)));
        let output = Command::new("ssh")
            .args(&[
                &format!("pi@{}", &address),
                "bash",
                "-c",
                &format!("set -e; echo {} | base64 --decode | sudo bash -", &encoded),
            ])
            .output()
            .expect("mv failed");
        if !output.status.success() {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            return Err(anyhow!("command failed with exit code {}", output.status));
        }
        let dest = format!("pi@{}:/tmp/homesec-bootstrap", address);
        let output = Command::new("scp")
            .args(&["../target/armv7-unknown-linux-gnueabihf/debug/homesec_bootstrap", &dest])
            .current_dir("../bootstrap")
            .output()
            .expect("build failed");
        if !output.status.success() {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            return Err(anyhow!("command failed with exit code {}", output.status));
        }
        let encoded = base64::encode("set -e; rm /usr/bin/homesec-bootstrap || true && mv /tmp/homesec-bootstrap /usr/bin/homesec-bootstrap && systemctl restart homesec-bootstrap.service");
        let output = Command::new("ssh")
            .args(&[
                &format!("pi@{}", &address),
                "bash",
                "-c",
                &format!("set -e; echo \"{}\" | base64 --decode | sudo bash -", &encoded),
            ])
            .output()
            .expect("mv failed");
        if !output.status.success() {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            return Err(anyhow!("command failed with exit code {}", output.status));
        }
        println!("successfully installed bootstrap to {}", address);
    }
    Ok(())
}

fn prepare() -> Result<Vec<String>> {
    let addresses = get_addresses()?;
    build_for_arm()?;
    install_bootstrap(&addresses[..])?;
    Ok(addresses)
}

fn get_port() -> Result<i32> {
    if let Ok(port) = std::env::var("PORT") {
        return Ok(port.parse::<i32>()?);
    }
    return Ok(43000);
}

fn main() -> Result<()> {
    let port = get_port()?;
    let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    socket.set_nonblocking(true)?;
    socket.set_broadcast(true)?;
    let addresses = prepare()?;
    println!("bootstrap.service updated on {} devices", addresses.len());
    let mut buf = [0; BUFFER_SIZE];
    let timeout = Duration::from_secs(30);
    let start = SystemTime::now();
    let mut happened = false;
    loop {
        let elapsed = SystemTime::now().duration_since(start).unwrap();
        if elapsed > timeout {
            break;
        }
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                let msg: Message = bincode::deserialize(&buf[..n]).expect("deser");
                match msg {
                    Message::ElectionResult(result) => {
                        println!("observed election result, addr={}, hid={}", result.addr, result.hid);
                        happened = true;
                        break;
                    }
                    _ => {}
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(Error::from(e)),
        }
    }
    if !happened {
        return Err(anyhow!("master was not elected before {:?} timeout", timeout));
    }
    let mut happened = false;
    loop {
        let elapsed = SystemTime::now().duration_since(start).unwrap();
        if elapsed > timeout {
            break;
        }
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                let msg: Message = bincode::deserialize(&buf[..n]).expect("deser");
                match msg {
                    Message::ConnectionDetails(details) => {
                        println!("observed k3s connection details, addr={}, hid={}, token={}", addr, details.hid, &details.token);
                        happened = true;
                        break;
                    }
                    _ => {}
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(Error::from(e)),
        }
    }
    if !happened {
        return Err(anyhow!("master did not broadcast connection details before {:?} timeout", timeout));
    }
    println!("Success");
    Ok(())
}
