use std::io;
use std::net::UdpSocket;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct AppearanceMessage {
    is_master: bool,
}

fn main() -> io::Result<()> {
    let port = 43000;
    let broadcast_addr = format!("192.168.0.255:{}", port);
    let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    socket.set_nonblocking(true)?;
    socket.set_broadcast(true)?;
    let mut buf = [0; 128];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                let msg: AppearanceMessage = bincode::deserialize(&buf[..n]).expect("deserialize");
                println!("{} bytes from {}, {:?}", n, addr, msg);
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            }
            Err(e) => panic!("encountered IO error: {}", e),
        }
        let msg = AppearanceMessage{
            is_master: false,
        };
        let encoded: Vec<u8> = bincode::serialize(&msg).expect("serialize");
        socket.send_to(&encoded[..], &broadcast_addr)?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Ok(())
}
