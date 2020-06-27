use std::net::SocketAddr;
use std::time::SystemTime;
use anyhow::{Result};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct AppearanceMessage {
    pub is_master: bool,
    pub priority: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Appearance(AppearanceMessage),
    StartElection,
}

pub struct Node {
    pub addr: SocketAddr,
    pub is_master: bool,
    pub priority: i32,
    pub last_seen: SystemTime,
    pub votes: i32,
}

impl Node {
    fn from_appearance(addr: SocketAddr, msg: &AppearanceMessage) -> Self {
        Self {
            addr,
            is_master: msg.is_master,
            priority: msg.priority,
            last_seen: SystemTime::now(),
            votes: 0,
        }
    }

    fn process_appearance(&mut self, msg: &AppearanceMessage) -> Result<()> {
        self.is_master = msg.is_master;
        self.priority = msg.priority;
        self.last_seen = SystemTime::now();
        Ok(())
    }

    fn cast_vote(&mut self) {
        self.votes += 1
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

    fn quorum(&self) -> usize {
        (self.nodes.len() as f64 * 0.666666666666667).ceil() as _
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