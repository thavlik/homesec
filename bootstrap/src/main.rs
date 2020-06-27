#[macro_use]
extern crate anyhow;

use anyhow::{Result};
use std::io;
use std::net::{SocketAddr, UdpSocket};

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

fn elect_leader() -> Result<SocketAddr> {
    let port = get_port()?;
    let broadcast_addr = get_broadcast_address(port)?;
    let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    socket.set_nonblocking(true)?;
    socket.set_broadcast(true)?;
    // Send a reset message so the process starts over for all nodes
    socket.send_to(&bincode::serialize(&Message::Reset)?[..], &broadcast_addr)?;
    let mut buf = [0; 128];
    let mut d = Election::new();
    let delay = std::time::Duration::from_secs(1);
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                match bincode::deserialize(&buf[..n])? {
                    Message::Appearance(msg) => d.handle_appearance(addr, &msg)?,
                    Message::CastVote(vote) => d.cast_vote(vote.candidate, vote.voter)?,
                    Message::Reset => d.reset(),
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => panic!("socket IO error: {}", e),
        }
        if let Some(leader) = d.check_result() {
            return Ok(leader);
        }
        let msg = Message::Appearance(AppearanceMessage {
            is_master: false,
            priority: 0,
        });
        let encoded: Vec<u8> = bincode::serialize(&msg)?;
        socket.send_to(&encoded[..], &broadcast_addr)?;
        std::thread::sleep(delay);
    }
}

fn main() -> Result<()> {
    let port = get_port()?;
    let broadcast_addr = get_broadcast_address(port)?;
    let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    socket.set_nonblocking(true)?;
    socket.set_broadcast(true)?;
    let mut buf = [0; 128];
    let mut d = Election::new();
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                let msg: Message = bincode::deserialize(&buf[..n]).expect("deserialize");
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => panic!("socket IO error: {}", e),
        }
        let msg = Message::Appearance(AppearanceMessage {
            is_master: false,
            priority: 0,
        });
        let encoded: Vec<u8> = bincode::serialize(&msg).expect("serialize");
        socket.send_to(&encoded[..], &broadcast_addr)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Ok(())
}
