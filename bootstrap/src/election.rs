use std::net::SocketAddr;
use std::time::SystemTime;
use anyhow::{Result};
use serde::{Serialize, Deserialize};
use std::cmp::Ordering;
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug)]
pub struct AppearanceMessage {
    pub is_master: bool,
    pub priority: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CastVote {
    pub voter: SocketAddr,
    pub candidate: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Appearance(AppearanceMessage),
    CastVote(CastVote),
    Reset,
}

pub struct Node {
    pub addr: SocketAddr,
    pub is_master: bool,
    pub priority: i32,
    pub last_seen: SystemTime,
    pub votes: HashSet<SocketAddr>,
}

impl Node {
    fn from_appearance(addr: SocketAddr, msg: &AppearanceMessage) -> Self {
        Self {
            addr,
            is_master: msg.is_master,
            priority: msg.priority,
            last_seen: SystemTime::now(),
            votes: HashSet::new(),
        }
    }

    fn process_appearance(&mut self, msg: &AppearanceMessage) -> Result<()> {
        self.is_master = msg.is_master;
        self.priority = msg.priority;
        self.last_seen = SystemTime::now();
        Ok(())
    }

    fn cast_vote(&mut self, voter: SocketAddr) {
        self.votes.insert(voter);
    }
}

pub struct Election {
    pub nodes: Vec<Node>,
}

impl Election {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    fn quorum(&self) -> usize {
        (self.nodes.len() as f64 * 0.666666666666667).ceil() as _
    }

    pub fn check_result(&self) -> Option<SocketAddr> {
        let quorum = self.quorum();
        let mut nodes = self.nodes.iter()
            .filter(|node| node.votes.len() >= quorum)
            .collect::<Vec<_>>();
        nodes.sort_by(|a, b| {
            if a.votes.len() < b.votes.len() {
                Ordering::Less
            } else if a.votes == b.votes {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        });
        if let Some(node) = nodes.iter().last() {
            Some(node.addr)
        } else {
            None
        }
    }

    pub fn handle_appearance(&mut self, addr: SocketAddr, msg: &AppearanceMessage) -> Result<()> {
        println!("{} {:?}", addr, msg);
        match self.nodes.iter_mut().find(|n| n.addr == addr) {
            Some(node) => node.process_appearance(msg)?,
            None => self.nodes.push(Node::from_appearance(addr, msg)),
        }
        Ok(())
    }

    pub fn cast_vote(&mut self, candidate: SocketAddr, voter: SocketAddr) -> Result<()> {
        match self.nodes.iter_mut().find(|n| n.addr == candidate) {
            Some(node) => {
                node.cast_vote(voter);
                println!("Casted vote for {:?}, total_votes={}", candidate, node.votes.len());
                Ok(())
            }
            None => Err(anyhow!("cannot cast vote on unknown candidate node")),
        }
    }

    pub fn reset(&mut self) {
        self.nodes.clear();
    }
}