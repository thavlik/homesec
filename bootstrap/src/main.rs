#[macro_use]
extern crate anyhow;

use anyhow::{Result};
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
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

fn elect_leader(socket: &mut UdpSocket, broadcast_addr: &str, hid: Uuid, is_master: bool, delay: Duration) -> Result<(SocketAddr, Uuid)> {
    let mut d = Election::new();
    let mut buf = [0; 128];

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

fn main() -> Result<()> {
    let hid = get_hid()?;
    let mut is_master = false;
    println!("hid={}", hid);
    println!("electing leader");
    let delay = Duration::from_millis(1000);
    let port = get_port()?;
    let broadcast_addr = get_broadcast_address(port)?;
    let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    socket.set_nonblocking(true)?;
    socket.set_broadcast(true)?;
    let (leader_addr, leader_hid) = elect_leader(&mut socket, &broadcast_addr, hid, is_master, delay)?;
    if leader_hid == hid {
        println!("elected master, broadcasting appearance");
        is_master = true;
        loop {
            let msg = Message::Appearance(AppearanceMessage {
                priority: -1,
                hid,
                is_master,
            });
            let encoded: Vec<u8> = bincode::serialize(&msg)?;
            socket.send_to(&encoded[..], &broadcast_addr)?;
            println!("sent");
            std::thread::sleep(delay);
        }
    } else {
        println!("elected {}, hid={}", leader_addr, leader_hid);
    }
    Ok(())
}
