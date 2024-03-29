use std::net::SocketAddr;
use std::time::{Duration, SystemTime};
use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::ops::Sub;
use std::alloc::System;
use rand::prelude::*;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct AppearanceMessage {
    pub priority: i32,
    pub is_master: bool,
    pub hid: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionDetails {
    pub hid: Uuid,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CastVote {
    pub addr: SocketAddr,
    pub hid: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ElectionResult {
    pub addr: SocketAddr,
    pub hid: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Appearance(AppearanceMessage),
    CastVote(CastVote),
    Reset,
    ElectionResult(ElectionResult),
    ConnectionDetails(ConnectionDetails),
}

#[derive(Clone)]
pub struct Node {
    pub addr: SocketAddr,
    pub hid: Uuid,
    pub is_master: bool,
    pub priority: i32,
    pub last_seen: SystemTime,
    pub votes: HashSet<SocketAddr>,
}

impl Node {
    fn from_appearance(addr: SocketAddr, msg: &AppearanceMessage) -> Self {
        Self {
            addr,
            hid: msg.hid,
            is_master: msg.is_master,
            priority: msg.priority,
            last_seen: SystemTime::now(),
            votes: HashSet::new(),
        }
    }

    fn process_appearance(&mut self, msg: &AppearanceMessage) -> Result<()> {
        if self.hid != msg.hid {
            return Err(anyhow!("detected address change for {}", msg.hid));
        }
        self.priority = msg.priority;
        self.is_master = msg.is_master;
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
    pub priority: i32,
}

fn gen_priority() -> i32 {
    rand::random::<u16>() as _
}

impl Election {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            start_time: SystemTime::now(),
            last_vote: SystemTime::now(),
            voted: false,
            delay: Duration::from_secs(10),
            priority: gen_priority(),
        }
    }

    pub fn process_message(&mut self, source: SocketAddr, msg: &Message) -> Result<()> {
        match msg {
            Message::Appearance(msg) => self.handle_appearance(source, &msg)?,
            Message::CastVote(CastVote { addr, hid }) => self.cast_vote(*addr, *hid, source)?,
            Message::Reset => {
                println!("resetting election");
                self.reset()
            }
            Message::ElectionResult(ElectionResult { addr, hid }) => {
                println!("{}, hid={} elected master by {}", addr, hid, source);
            },
            Message::ConnectionDetails(ConnectionDetails{ hid, token }) => {
                println!("received connection details for {}", hid);
                if let Some(node) = self.nodes.iter_mut().find(|node| node.hid == *hid) {
                    node.is_master = true;
                } else {
                    self.nodes.push(Node::from_appearance(source, &AppearanceMessage{
                        is_master: true,
                        hid: *hid,
                        priority: -1,
                    }));
                }
            },
        }
        Ok(())
    }

    fn quorum(&self) -> usize {
        (self.nodes.len() as f64 * 0.666666666666667).ceil() as _
    }

    fn too_early(&self) -> bool {
        SystemTime::now().duration_since(self.start_time).unwrap() < self.delay
    }

    pub fn check_vote(&mut self) -> Option<(SocketAddr, Uuid)> {
        if self.voted {
            None
        } else if let Some(node) = self.nodes.iter().find(|node| node.is_master) {
            // Always prefer existing master
            self.voted = true;
            Some((node.addr, node.hid))
        } else if self.nodes.is_empty() || self.too_early() {
            None
        } else {
            let mut nodes = self.nodes.clone();
            nodes.sort_by_key(|node| node.priority);
            self.voted = true;
            let node = nodes.last().unwrap();
            Some((node.addr, node.hid))
        }
    }

    pub fn check_result(&mut self) -> (Option<(SocketAddr, Uuid)>, bool) {
        if let Some(node) = self.nodes.iter().find(|node| node.is_master) {
            // always prefer an existing master
            return (Some((node.addr, node.hid)), false);
        }
        if self.too_early() || SystemTime::now().duration_since(self.last_vote).unwrap() < self.delay {
            // wait for sufficient appearance messages
            return (None, false);
        }
        let quorum = self.quorum();
        let nodes = self.nodes.iter()
            .filter(|node| node.votes.len() >= quorum)
            .collect::<Vec<_>>();
        if nodes.is_empty() {
            // No quorum has yet been made
            return (None, false);
        }
        let acc = (nodes[0].addr, nodes[0].hid, nodes[0].votes.len());
        let (addr, hid, winning_vote_count) = nodes[1..]
            .iter()
            .fold(acc, |acc, node| {
                if node.votes.len() > acc.2 {
                    (node.addr, node.hid, node.votes.len())
                } else {
                    acc
                }
            });
        if nodes.iter().filter(|node| {
            node.votes.len() == winning_vote_count
        }).count() > 1 {
            println!("More than one master was elected. Holding new election...");
            self.reset();
            return (None, true);
        }
        (Some((addr, hid)), false)
    }

    pub fn handle_appearance(&mut self, addr: SocketAddr, msg: &AppearanceMessage) -> Result<()> {
        println!("{} {:?}", addr, msg);
        match self.nodes.iter_mut().find(|n| n.addr == addr) {
            Some(node) => node.process_appearance(msg)?,
            None => self.nodes.push(Node::from_appearance(addr, msg)),
        }
        Ok(())
    }

    pub fn cast_vote(&mut self, addr: SocketAddr, hid: Uuid, voter: SocketAddr) -> Result<()> {
        match self.nodes.iter_mut().find(|n| n.addr == addr && n.hid == hid) {
            Some(node) => {
                node.cast_vote(voter);
                self.last_vote = SystemTime::now();
                println!("{} casted vote for {}, total_votes={}", voter, addr, node.votes.len());
                Ok(())
            }
            None => Err(anyhow!("cannot cast vote on unknown candidate node")),
        }
    }

    pub fn reset(&mut self) {
        self.nodes.clear();
        self.start_time = SystemTime::now();
        self.voted = false;
        self.priority = gen_priority();
        println!("assigned priority {}", self.priority);
    }
}
