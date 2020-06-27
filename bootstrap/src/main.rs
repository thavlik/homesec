#[macro_use]
extern crate anyhow;

use anyhow::{Result, Error};
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

mod election;

use election::*;

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

fn elect_master(socket: &mut UdpSocket, broadcast_addr: &str, hid: Uuid, is_master: bool) -> Result<(SocketAddr, Uuid)> {
    let mut d = Election::new();
    let mut buf = [0; 128];
    let delay = Duration::from_secs(1);
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                let msg: Message = bincode::deserialize(&buf[..n])?;
                d.process_message(addr, &msg)?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => panic!("socket IO error: {}", e),
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

fn listen_for_existing_master(socket: &mut UdpSocket, wait_period: Duration) -> Result<Option<(SocketAddr, Uuid)>> {
    let start = SystemTime::now();
    let delay = Duration::from_secs(1);
    let mut buf = [0; 128];
    loop {
        let elapsed = SystemTime::now().duration_since(start).unwrap();
        if elapsed > wait_period {
            return Ok(None);
        }
        match socket.recv_from(&mut buf) {
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

fn run_master(socket: &mut UdpSocket, hid: Uuid, broadcast_addr: &str) -> Result<()> {
    // TODO: start k8s master
    let token = format!("TODO");
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

fn run_worker(socket: &mut UdpSocket) -> Result<()> {
    // TODO: start k8s worker
    loop {
        std::thread::sleep(Duration::from_millis(1000));
    }
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
    let (master_addr, master_hid) = listen_for_existing_master(&mut socket, wait_period)?
        .unwrap_or(elect_master(&mut socket, &broadcast_addr, hid, is_master)?);
    if master_hid == hid {
        println!("elected master");
        run_master(&mut socket, hid, &broadcast_addr)
    } else {
        println!("elected {} as master, hid={}", master_addr, master_hid);
        run_worker(&mut socket)
    }
}