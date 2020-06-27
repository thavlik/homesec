use std::net::SocketAddr;
use std::time::{Duration, SystemTime};
use anyhow::{Result};
use serde::{Serialize, Deserialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::ops::Sub;
use std::alloc::System;

#[derive(Serialize, Deserialize, Debug)]
pub struct AppearanceMessage {
    pub priority: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CastVote {
    pub candidate: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Appearance(AppearanceMessage),
    CastVote(CastVote),
    Reset,
}

#[derive(Clone)]
pub struct Node {
    pub addr: SocketAddr,
    pub priority: i32,
    pub last_seen: SystemTime,
    pub votes: HashSet<SocketAddr>,
}

impl Node {
    fn from_appearance(addr: SocketAddr, msg: &AppearanceMessage) -> Self {
        Self {
            addr,
            priority: msg.priority,
            last_seen: SystemTime::now(),
            votes: HashSet::new(),
        }
    }

    fn process_appearance(&mut self, msg: &AppearanceMessage) -> Result<()> {
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
    pub start_time: SystemTime,
    pub last_vote: SystemTime,
    pub voted: bool,
    pub delay: Duration,
}

impl Election {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            start_time: SystemTime::now(),
            last_vote: SystemTime::now(),
            voted: false,
            delay: Duration::from_secs(10),
        }
    }

    pub fn process_message(&mut self, addr: SocketAddr, msg: &Message) -> Result<()> {
        match msg {
            Message::Appearance(msg) => self.handle_appearance(addr, &msg)?,
            Message::CastVote(vote) => self.cast_vote(vote.candidate, addr)?,
            Message::Reset => self.reset(),
        }
        Ok(())
    }

    fn quorum(&self) -> usize {
        (self.nodes.len() as f64 * 0.666666666666667).ceil() as _
    }

    fn too_early(&self) -> bool {
        SystemTime::now().duration_since(self.start_time).unwrap() < self.delay
    }

    pub fn check_vote(&mut self) -> Option<SocketAddr> {
        if self.voted || self.nodes.is_empty() || self.too_early() {
            None
        } else {
            let mut nodes = self.nodes.clone();
            nodes.sort_by_key(|node| node.priority);
            self.voted = true;
            Some(nodes.last().unwrap().addr)
        }
    }

    pub fn check_result(&mut self) -> Option<SocketAddr> {
        if self.too_early() || SystemTime::now().duration_since(self.last_vote).unwrap() < self.delay {
            // Wait for all the votes to tally
            return None;
        }
        let quorum = self.quorum();
        let mut nodes = self.nodes.iter()
            .filter(|node| node.votes.len() >= quorum)
            .collect::<Vec<_>>();
        if nodes.is_empty() {
            // No quorum has yet been made
            return None;
        }
        let (addr, winning_vote_count) = nodes
            .iter()
            .fold((None, 0), |v, node| {
                if node.votes.len() > v.1 {
                    (Some(node.addr), node.votes.len())
                } else {
                    v
                }
            });
        if winning_vote_count == 0 {
            // Sanity check
            return None;
        }
        if nodes.iter().filter(|node| node.votes.len() == winning_vote_count).count() > 1 {
            // More than one leader was elected. Do the whole thing over again.
            println!("More than one leader was elected. Holding new election...");
            self.reset();
            return None;
        }
        Some(addr.unwrap())
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
                self.last_vote = SystemTime::now();
                println!("Casted vote for {:?}, total_votes={}", candidate, node.votes.len());
                Ok(())
            }
            None => Err(anyhow!("cannot cast vote on unknown candidate node")),
        }
    }

    pub fn reset(&mut self) {
        self.nodes.clear();
        self.start_time = SystemTime::now();
        self.voted = false;
    }
}