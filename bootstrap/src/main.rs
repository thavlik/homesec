use std::io;
use std::net::UdpSocket;

struct AppearanceMessage {
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut socket = UdpSocket::bind("0.0.0.0:43000")?;
    socket.set_nonblocking(true)?;
    socket.set_broadcast(true)?;
    loop {
        let mut buf = [0; 10];
        match socket.recv_from(&mut buf) {
            Ok(n) => {
                println!("got message");
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            }
            Err(e) => panic!("encountered IO error: {}", e),
        }
        socket.send_to(&[0; 10], "192.168.0.255:43000")?;
        println!("sent packet");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Ok(())
}
