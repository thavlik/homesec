#[macro_use]
extern crate anyhow;

use anyhow::{Result};
use std::io;
use std::net::{SocketAddr, UdpSocket};
use rand::prelude::*;

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

    // Assign a random priority
    let priority = rand::random();
    println!("Assigned priority {}", priority);

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
        if let Some(leader) = d.check_result() {
            let msg = Message::LeaderElected(LeaderElected {
                addr: leader,
            });
            let encoded: Vec<u8> = bincode::serialize(&msg)?;
            socket.send_to(&encoded[..], &broadcast_addr)?;
            return Ok(leader);
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
            priority,
        });
        let encoded: Vec<u8> = bincode::serialize(&msg)?;
        socket.send_to(&encoded[..], &broadcast_addr)?;
        std::thread::sleep(delay);
    }
}

fn main() -> Result<()> {
    let leader = elect_leader()?;
    println!("elected {}", leader);
    Ok(())
}
