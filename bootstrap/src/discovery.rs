use std::net::SocketAddr;
use std::time::SystemTime;
use anyhow::{Result};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppearanceMessage {
    pub is_master: bool,
    pub priority: u8,
}

pub struct Node {
    pub addr: SocketAddr,
    pub is_master: bool,
    pub last_seen: SystemTime,
}

impl Node {
    fn from_appearance(addr: SocketAddr, msg: &AppearanceMessage) -> Self {
        Self {
            addr,
            is_master: msg.is_master,
            last_seen: SystemTime::now(),
        }
    }

    fn process_appearance(&mut self, msg: &AppearanceMessage) -> Result<()> {
        self.is_master = msg.is_master;
        self.last_seen = SystemTime::now();
        Ok(())
    }
}

pub struct Discovery {
    pub nodes: Vec<Node>,
}

impl Discovery {
    pub fn new() -> Self {
        Discovery {
            nodes: Vec::new(),
        }
    }

    pub fn handle_appearance(&mut self, addr: std::net::SocketAddr, msg: &AppearanceMessage) -> Result<()> {
        println!("{} {:?}", addr, msg);
        match self.nodes.iter_mut().find(|n| n.addr == addr) {
            Some(node) => node.process_appearance(msg)?,
            None => self.nodes.push(Node::from_appearance(addr, msg)),
        }
        Ok(())
    }
}