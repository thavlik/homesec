#[macro_use]
extern crate anyhow;

use anyhow::{Result, Error};
use std::io;
use std::net::UdpSocket;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct AppearanceMessage {
    is_master: bool,
    priority: u8,
}

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

fn handle_appearance(addr: std::net::SocketAddr, msg: &AppearanceMessage) -> Result<()> {
    println!("{} {:?}", addr, msg);
    Ok(())
}

fn main() -> Result<()> {
    let port = get_port()?;
    let broadcast_addr = get_broadcast_address(port)?;
    let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    socket.set_nonblocking(true)?;
    socket.set_broadcast(true)?;
    let mut buf = [0; 128];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                let msg: AppearanceMessage = bincode::deserialize(&buf[..n]).expect("deserialize");
                handle_appearance(addr, &msg)?;
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            }
            Err(e) => panic!("socket IO error: {}", e),
        }
        let msg = AppearanceMessage{
            is_master: false,
            priority: 0,
        };
        let encoded: Vec<u8> = bincode::serialize(&msg).expect("serialize");
        socket.send_to(&encoded[..], &broadcast_addr)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Ok(())
}
