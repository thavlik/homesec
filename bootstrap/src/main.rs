#[macro_use]
extern crate anyhow;

use anyhow::{Result, Error};
use std::process::Command;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};
use uuid::Uuid;
use std::io::{self, Write};

mod election;

use election::*;

const BUFFER_SIZE: usize = 8192;

fn get_port() -> Result<i32> {
    if let Ok(port) = std::env::var("PORT") {
        return Ok(port.parse::<i32>()?);
    }
    return Ok(43000);
}

fn get_broadcast_address(port: i32) -> Result<String> {
    // TODO: probe eth0
    if let Ok(broadcast_addr) = std::env::var("BROADCAST_ADDR") {
        return Ok(broadcast_addr);
    }
    Ok(format!("192.168.0.255:{}", port))
}

fn get_hid() -> Result<Uuid> {
    Ok(std::fs::read_to_string("/etc/hid")?.trim().parse()?)
}

fn elect_master(socket: &mut UdpSocket, broadcast_addr: &str, hid: Uuid, is_master: bool, buf: &mut [u8]) -> Result<(SocketAddr, Uuid)> {
    let mut d = Election::new();
    let delay = Duration::from_secs(1);
    loop {
        match socket.recv_from(buf) {
            Ok((n, addr)) => {
                let msg: Message = bincode::deserialize(&buf[..n])?;
                d.process_message(addr, &msg)?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(Error::from(e)),
        }
        match d.check_result() {
            (Some((addr, hid)), false) => {
                let msg = Message::ElectionResult(ElectionResult {
                    addr,
                    hid,
                });
                let encoded: Vec<u8> = bincode::serialize(&msg)?;
                socket.send_to(&encoded[..], &broadcast_addr)?;
                return Ok((addr, hid));
            }
            (None, true) => {
                let encoded: Vec<u8> = bincode::serialize(&Message::Reset)?;
                socket.send_to(&encoded[..], &broadcast_addr)?;
            }
            (None, false) => {}
            _ => unreachable!(),
        }
        if let Some((addr, hid)) = d.check_vote() {
            let msg = Message::CastVote(CastVote {
                addr,
                hid,
            });
            let encoded: Vec<u8> = bincode::serialize(&msg)?;
            socket.send_to(&encoded[..], &broadcast_addr)?;
        }
        // Send an appearance message
        let msg = Message::Appearance(AppearanceMessage {
            priority: d.priority,
            hid,
            is_master,
        });
        let encoded: Vec<u8> = bincode::serialize(&msg)?;
        socket.send_to(&encoded[..], &broadcast_addr)?;
        std::thread::sleep(delay);
    }
}

fn listen_for_existing_master(socket: &mut UdpSocket, wait_period: Duration, buf: &mut [u8]) -> Result<Option<(SocketAddr, Uuid)>> {
    let start = SystemTime::now();
    let delay = Duration::from_secs(1);
    loop {
        let elapsed = SystemTime::now().duration_since(start).unwrap();
        if elapsed > wait_period {
            return Ok(None);
        }
        match socket.recv_from(buf) {
            Ok((n, addr)) => {
                let msg: Message = bincode::deserialize(&buf[..n])?;
                match msg {
                    Message::Appearance(msg) if msg.is_master => {
                        return Ok(Some((addr, msg.hid)));
                    }
                    _ => {}
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(Error::from(e)),
        }
        std::thread::sleep(delay);
    }
}

fn get_node_token() -> Result<String> {
    Ok(String::from(std::fs::read_to_string("/var/lib/rancher/k3s/server/node-token")?.trim()))
}

fn run_master(hid: Uuid, socket: &mut UdpSocket, broadcast_addr: &str, buf: &mut [u8]) -> Result<()> {
    println!("running k3s master install script");
    let output = Command::new("sh")
        .args(&[
            "-c",
            format!("set -e; curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC=\"server --kube-apiserver-arg enable-admission-plugins=PodSecurityPolicy,NodeRestriction\" K3S_NODE_NAME=pi-{} sh -s -", hid),
        ])
        .output()
        .expect("build failed");
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        return Err(anyhow!("k3s master install failed with exit code {}", output.status));
    }
    let token = get_node_token()?;
    println!("k3s install script successful, token={}", &token);
    loop {
        let msg = Message::ConnectionDetails(ConnectionDetails {
            hid,
            token: token.clone(),
        });
        let encoded: Vec<u8> = bincode::serialize(&msg)?;
        socket.send_to(&encoded[..], &broadcast_addr)?;
        std::thread::sleep(Duration::from_millis(1000));
    }
    Ok(())
}

fn wait_for_connection_details(socket: &mut UdpSocket, buf: &mut [u8]) -> Result<(SocketAddr, ConnectionDetails)> {
    let start = SystemTime::now();
    let timeout = Duration::from_secs(30);
    loop {
        let elapsed = SystemTime::now().duration_since(start).unwrap();
        if elapsed > timeout {
            return Err(anyhow!("timed out waiting for connection details from master"));
        }
        // TODO: add timeout
        match socket.recv_from(buf) {
            Ok((n, addr)) => {
                let msg: Message = bincode::deserialize(&buf[..n])?;
                match msg {
                    Message::ConnectionDetails(details) => {
                        return Ok((addr, details));
                    }
                    _ => {}
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(Error::from(e)),
        }
        std::thread::sleep(Duration::from_millis(1000));
    }
}

fn run_agent(hid: Uuid, socket: &mut UdpSocket, buf: &mut [u8]) -> Result<()> {
    let (addr, details) = wait_for_connection_details(socket, buf)?;
    println!("received connection details, addr={}, token={}", addr, &details.token);
    let addr: String = match addr {
        SocketAddr::V4(addr) => addr.ip().to_string(),
        SocketAddr::V6(addr) => addr.ip().to_string(),
    };
    println!("running k3s agent install script");
    let output = Command::new("sh")
        .args(&[
            "-c",
            &format!("set -e; curl -sfL https://get.k3s.io | K3S_URL=https://{}:6443 K3S_TOKEN={} K3S_NODE_NAME=pi-{} sh -s -", &addr, &details.token, hid),
        ])
        .output()
        .expect("build failed");
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        return Err(anyhow!("k3s master install failed with exit code {}", output.status));
    }
    println!("k3s agent install script successful");
    Ok(())
}

fn main() -> Result<()> {
    let hid = get_hid()?;
    let mut is_master = false;
    println!("hid={}", hid);
    println!("electing master");
    let port = get_port()?;
    let broadcast_addr = get_broadcast_address(port)?;
    let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    socket.set_nonblocking(true)?;
    socket.set_broadcast(true)?;
    let wait_period = Duration::from_secs(5);
    let mut buf = [0; BUFFER_SIZE];
    let (master_addr, master_hid) = listen_for_existing_master(&mut socket, wait_period, &mut buf[..])?
        .unwrap_or(elect_master(&mut socket, &broadcast_addr, hid, is_master, &mut buf[..])?);
    if master_hid == hid {
        println!("this node was elected master");
        Ok(run_master(hid, &mut socket, &broadcast_addr, &mut buf[..])?)
    } else {
        println!("elected {} as master, hid={}", master_addr, master_hid);
        Ok(run_agent(hid, &mut socket, &mut buf[..])?)
    }
}