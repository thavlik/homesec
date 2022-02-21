#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate clap;

use clap::Clap;
use anyhow::{Result, Error};
use std::process::Command;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};
use uuid::Uuid;
use std::io::{self, Write};
use std::path::Path;

mod election;

use election::*;

const broadcast_ip = 10.0.0.255;

#[derive(Clap)]
pub enum SubCommand {
    /// Run the systemd service
    #[clap(name = "daemon")]
    Daemon,

    /// Remove the service
    #[clap(name = "remove")]
    Remove,
}

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Tom Havlik")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

const BUFFER_SIZE: usize = 8192;

fn get_port() -> Result<i32> {
    if let Ok(port) = std::env::var("PORT") {
        let port = port.parse::<i32>()?;
        println!("PORT environment variable set to {}", port);
        Ok(port)
    } else {
        let default_port = 43000;
        println!("defaulting to port {}", default_port);
        Ok(default_port)
    }
}

fn get_broadcast_address(port: i32) -> Result<String> {
    // TODO: probe eth0
    if let Ok(broadcast_addr) = std::env::var("BROADCAST_ADDR") {
        return Ok(broadcast_addr);
    }
    Ok(format!("{}:{}", broadcast_ip, port))
}

fn get_hid() -> Result<Uuid> {
    let path = "/etc/hid";
    if std::path::Path::new(path).exists() {
	println!("found existing hid at {}", path);
        Ok(std::fs::read_to_string(path)?.trim().parse()?)
    } else {
        let hid = Uuid::new_v4();
        let s = hid.to_hyphenated().to_string();
        println!("generated novel HID {} at {}", s, path);
        std::fs::write(path, s)?;
        Ok(hid)
    }
}

fn elect_master(socket: &mut UdpSocket, broadcast_addr: &str, hid: Uuid, is_master: bool, buf: &mut [u8]) -> Result<(SocketAddr, Uuid)> {
    println!("electing master");
    let mut d = Election::new();
    let delay = Duration::from_millis(1000);
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
                println!("master elected: {}, {}", addr, hid);
                return Ok((addr, hid));
            }
            (None, true) => {
                println!("broadcasting reset message");
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
        println("broadcasting appearance message");
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
    println!("listening for existing master");
    let start = SystemTime::now();
    let delay = Duration::from_millis(100);
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
    let path = "/var/lib/rancher/k3s/server/node-token";
    if !Path::new(path).exists() {
        return Err(anyhow!("k3s node token not found at {}", path));
    }
    println!("k3s node token found at {}", path);
    Ok(String::from(std::fs::read_to_string(path)?.trim()))
}

fn run_master(hid: Uuid, socket: &mut UdpSocket, broadcast_addr: &str, buf: &mut [u8]) -> Result<()> {
    println!("running k3s master install script");
    let output = Command::new("sh")
        .args(&[
            "-c",
            &format!("set -e; curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC=\"server --disable traefik --kube-apiserver-arg enable-admission-plugins=PodSecurityPolicy,NodeRestriction\" K3S_NODE_NAME=pi-{} sh -s -", hid),
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
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}

fn wait_for_next_election(socket: &mut UdpSocket, buf: &mut [u8]) -> Result<()> {
    loop {
        match socket.recv_from(buf) {
            Ok((n, addr)) => {
                let msg: Message = bincode::deserialize(&buf[..n])?;
                match msg {
                    Message::Reset => {
                        return Ok(());
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

fn wait_for_connection_details(socket: &mut UdpSocket, buf: &mut [u8]) -> Result<(SocketAddr, ConnectionDetails)> {
    let start = SystemTime::now();
    let timeout = Duration::from_secs(150);
    loop {
        let elapsed = SystemTime::now().duration_since(start).unwrap();
        if elapsed > timeout {
            return Err(anyhow!("timed out waiting for connection details from master"));
        }
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
    Ok(wait_for_next_election(socket, buf)?)
}

const MASTER_PATH: &'static str = "/etc/k3s-master";

fn get_master_status() -> Result<bool> {
    Ok(Path::new(MASTER_PATH).exists())
}

fn set_master_status(value: bool) -> Result<()> {
    if value {
        if !get_master_status()? {
            //if let Err(err) = Command::new("/usr/local/bin/k3s-uninstall.sh").output() {
            //    if err.raw_os_error().unwrap_or(2) != 2 {
            //        // exit code 2 is NotFound
            //        return Err(Error::from(err));
            //    }
            //}
            std::fs::write(MASTER_PATH, "")?;
        }
    } else if get_master_status()? {
        //if let Err(err) = Command::new("/usr/local/bin/k3s-agent-uninstall.sh").output() {
        //    if err.raw_os_error().unwrap_or(2) != 2 {
        //        return Err(Error::from(err));
        //    }
        //}
        std::fs::remove_file(MASTER_PATH)?;
    }
    Ok(())
}

fn daemon_main() -> Result<()> {
    println!("starting daemon");
    let hid = get_hid()?;
    let mut is_master = get_master_status()?;
    println!("hid={}", hid);
    let port = get_port()?;
    let broadcast_addr = get_broadcast_address(port)?;
    let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    socket.set_nonblocking(true)?;
    socket.set_broadcast(true)?;
    let mut buf = [0; BUFFER_SIZE];
    if !is_master {
        println!("finding master");
        let wait_period = Duration::from_secs(5);
        let (master_addr, master_hid) = listen_for_existing_master(&mut socket, wait_period, &mut buf[..])?
            .unwrap_or(elect_master(&mut socket, &broadcast_addr, hid, is_master, &mut buf[..])?);
        is_master = master_hid == hid;
        set_master_status(is_master)?;
        if is_master {
            println!("this node was elected master");
        } else {
            println!("elected {} as master, hid={}", master_addr, master_hid);
        }
    } else {
        println!("waiting for master to broadcast connection details");
    }
    if is_master {
        Ok(run_master(hid, &mut socket, &broadcast_addr, &mut buf[..])?)
    } else {
        Ok(run_agent(hid, &mut socket, &mut buf[..])?)
    }
}

fn disable_systemd_service() -> Result<()> {
    let output = Command::new("sudo")
        .args(&[
            "systemctl",
            "stop",
            "homesec-bootstrap.service",
        ])
        .output()?;
    if !output.status.success() {
        if !std::str::from_utf8(&output.stderr).unwrap().contains("Failed to connect to bus: No such file or directory") {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            return Err(anyhow!("systemctl command failed with exit code {}", output.status));
        }
    }
    Ok(())
}

fn remove_cluster_preferences() -> Result<()> {
    match std::fs::remove_file("/etc/k3s-master") {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::from(e)),
    }
}

fn remove_main() -> Result<()> {
    disable_systemd_service()?;
    remove_cluster_preferences()?;
    if Path::new("/usr/local/bin/k3s-uninstall.sh").exists() {
        let output = Command::new("/usr/local/bin/k3s-uninstall.sh")
            .output()?;
        if !output.status.success() {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            return Err(anyhow!("k3s-uninstall.sh failed with exit code {}", output.status));
        }
    }
    if Path::new("/usr/local/bin/k3s-agent-uninstall.sh").exists() {
        let output = Command::new("/usr/local/bin/k3s-agent-uninstall.sh")
            .output()?;
        if !output.status.success() {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            return Err(anyhow!("k3s-agent-uninstall.sh failed with exit code {}", output.status));
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Daemon => daemon_main(),
        SubCommand::Remove => remove_main(),
    }
}
