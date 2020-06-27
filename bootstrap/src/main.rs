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

fn elect_leader(is_master: bool) -> Result<SocketAddr> {
    let port = get_port()?;
    let broadcast_addr = get_broadcast_address(port)?;
    let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    socket.set_nonblocking(true)?;
    socket.set_broadcast(true)?;

    let mut d = Election::new();
    let mut buf = [0; 128];
    let delay = std::time::Duration::from_secs(1);

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
            (Some(leader), false) => {
                let msg = Message::ElectionResult(ElectionResult {
                    addr: leader,
                });
                let encoded: Vec<u8> = bincode::serialize(&msg)?;
                socket.send_to(&encoded[..], &broadcast_addr)?;
                return Ok(leader);
            },
            (None, true) => {
                let encoded: Vec<u8> = bincode::serialize(&Message::Reset)?;
                socket.send_to(&encoded[..], &broadcast_addr)?;
            },
            (None, false) => {},
            _ => unreachable!(),
        }
        if let Some(candidate) = d.check_vote() {
            let msg = Message::CastVote(CastVote {
                candidate,
            });
            let encoded: Vec<u8> = bincode::serialize(&msg)?;
            socket.send_to(&encoded[..], &broadcast_addr)?;
        }
        // Send an appearance message
        let msg = Message::Appearance(AppearanceMessage {
            priority: d.priority,
            is_master,
        });
        let encoded: Vec<u8> = bincode::serialize(&msg)?;
        socket.send_to(&encoded[..], &broadcast_addr)?;
        std::thread::sleep(delay);
    }
}

fn main() -> Result<()> {
    println!("electing leader");
    let leader = elect_leader(false)?;
    println!("elected {}", leader);
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Ok(())
}
