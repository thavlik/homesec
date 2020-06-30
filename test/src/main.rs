#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate futures;

use anyhow::{Error, Result};
use std::net::{UdpSocket, SocketAddr};
use std::process::Command;
use std::io::{self, Write};
use std::time::{SystemTime, Duration};
use k8s_openapi::api::core::v1;
use kube::{
    api::{Api, ListParams, Meta},
    client::Client,
};

use homesec_bootstrap::{Message, AppearanceMessage};
use std::collections::{HashMap, HashSet};

const BUFFER_SIZE: usize = 8192;

fn build_for_arm() -> Result<()> {
    let output = Command::new("sh")
        .args(&[
            "-c",
            "rm ../target/armv7-unknown-linux-gnueabihf/debug/homesec_bootstrap || true && cargo build --target armv7-unknown-linux-gnueabihf",
        ])
        .current_dir("../bootstrap")
        .output()?;
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        Err(anyhow!("command failed with exit code {}", output.status))
    } else {
        println!("successfully built bootstrap for armv7");
        Ok(())
    }
}

fn get_addresses() -> Result<Vec<String>> {
    Ok(vec![
        String::from("192.168.0.100"),
        String::from("192.168.0.102"),
        String::from("192.168.0.103"),
    ])
}

async fn install_bootstrap(address: &str) -> Result<()> {
    println!("installing to {}", address);
    let spec = include_str!("../../bootstrap/extra/homesec-bootstrap.service");
    let encoded = base64::encode(format!("echo \"{}\" | base64 --decode > /etc/systemd/system/homesec-bootstrap.service", base64::encode(spec)));
    let output = Command::new("ssh")
        .args(&[
            &format!("pi@{}", &address),
            "bash",
            "-c",
            &format!("set -e; echo {} | base64 --decode | sudo bash -", &encoded),
        ])
        .output()?;
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        return Err(anyhow!("command failed with exit code {}", output.status));
    }
    let dest = format!("pi@{}:/tmp/homesec-bootstrap", address);
    let output = Command::new("scp")
        .args(&["../target/armv7-unknown-linux-gnueabihf/debug/homesec_bootstrap", &dest])
        .current_dir("../bootstrap")
        .output()?;
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        return Err(anyhow!("command failed with exit code {}", output.status));
    }
    let encoded = base64::encode("set -e; rm /usr/bin/homesec-bootstrap || true && mv /tmp/homesec-bootstrap /usr/bin/homesec-bootstrap && systemctl restart homesec-bootstrap.service");
    let output = Command::new("ssh")
        .args(&[
            &format!("pi@{}", &address),
            "bash",
            "-c",
            &format!("set -e; echo \"{}\" | base64 --decode | sudo bash -", &encoded),
        ])
        .output()?;
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        return Err(anyhow!("command failed with exit code {}", output.status));
    }
    println!("successfully installed bootstrap to {}", address);
    Ok(())
}

fn uninstall_k3s(address: &str) -> Result<()> {
    let output = Command::new("sh")
        .args(&[
            "-c",
            &format!("set -e; ssh pi@{} sudo /usr/bin/homesec-bootstrap remove", address),
        ])
        .output()?;
    if !output.status.success() {
        if let Some(code) = output.status.code() {
            if code == 127 {
                // command not found
                return Ok(());
            }
        }
        //if std::str::from_utf8(&output.stderr)?.contains("No such file or directory (os error 2)") {
        //    // I believe this error is actually from the remove command
        //    println!("k3s already uninstalled for {}", address);
        //    return Ok(());
        //}
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        return Err(anyhow!("bootstrap remove failed with exit code {}", output.status));
    }
    Ok(())
}

fn ensure_uninstalled(address: &str) -> Result<()> {
    let encoded = base64::encode("set -e;\
if [[ -n \"$(ls /etc | grep k3s-master)\" ]]; then exit 100; fi;\
if [[ -n \"$(ls /etc | grep rancher)\" ]]; then exit 101; fi;\
if [[ -n \"$(ls /var/lib | grep rancher)\" ]]; then exit 102; fi;\
if [[ -n \"$(ls /etc/systemd/system | grep homesec-bootstrap.service)\" ]]; then exit 103; fi;");
    let output = Command::new("sh")
        .args(&[
            "-c",
            &format!("set -e; ssh pi@{} bash -c set -e; echo {} | base64 --decode | bash -", address, encoded),
        ])
        .output()?;
    if !output.status.success() {
        if let Some(code) = output.status.code() {
            if code >= 100 && code <= 103 {
                return Err(anyhow!("failed to uninstall k3s: exit code {}", code));
            }
        }
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        return Err(anyhow!("ensure_uninstalled failed with exit code {}", output.status));
    }
    Ok(())
}

fn wait_for_network_silence(socket: &mut UdpSocket, buf: &mut [u8]) -> Result<()> {
    let start = SystemTime::now();
    let mut last_message = SystemTime::now();
    let mut addr = None;
    let timeout = Duration::from_secs(10);
    let wait_period = Duration::from_secs(5);
    loop {
        if SystemTime::now().duration_since(start).unwrap() > timeout {
            return Err(anyhow!("unexpected network activity from {} after {:?}", addr.unwrap(), timeout));
        }
        if SystemTime::now().duration_since(last_message).unwrap() > wait_period {
            return Ok(());
        }
        match socket.recv_from(buf) {
            Ok((_, source)) => {
                last_message = SystemTime::now();
                addr = Some(source);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(Error::from(e)),
        }
    }
}

async fn prepare(socket: &mut UdpSocket, buf: &mut [u8]) -> Result<Vec<String>> {
    let addresses = get_addresses()?;
    for address in &addresses {
        uninstall_k3s(address)?;
        ensure_uninstalled(address)?;
        println!("uninstalled k3s for {}", address);
    }
    /*let errs = futures::future::join_all(
        addresses.iter()
            .map(|address| {

            }).collect::<Vec<_>>())
        .await
        .iter()
        .enumerate()
        .filter(|(_, r)| r.is_err())
        .map(|(i, r)| (i, format!("{}", &r.as_ref().err().unwrap())))
        .collect::<Vec<_>>();
    if errs.len() > 0 {
        let mut message = String::from("error uninstalling k3s:");
        errs.iter().for_each(|(i, err)| {
            message += &format!("  {}: {}\n", &addresses[*i], err)
        });
        return Err(anyhow!(message));
    }
     */
    println!("waiting for network silence");
    wait_for_network_silence(socket, buf)?;
    build_for_arm()?;
    let errs = futures::future::join_all(
        addresses.iter()
            .map(|address| {
                install_bootstrap(address)
            }).collect::<Vec<_>>())
        .await
        .into_iter()
        .enumerate()
        .filter(|(_, r)| r.is_err())
        .map(|(i, r)| (i, format!("{}", &r.as_ref().err().unwrap())))
        .collect::<Vec<_>>();
    if errs.len() > 0 {
        let mut message = String::from("error installing bootstrap service:");
        errs.iter().for_each(|(i, err)| {
            message += &format!("  {}: {}\n", &addresses[*i], err)
        });
        return Err(anyhow!(message));
    }
    Ok(addresses)
}

fn get_port() -> Result<i32> {
    if let Ok(port) = std::env::var("PORT") {
        return Ok(port.parse::<i32>()?);
    }
    return Ok(43000);
}

fn retrieve_kubeconfig(addr: SocketAddr) -> Result<()> {
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let port = get_port()?;
    let mut socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
    socket.set_nonblocking(true)?;
    socket.set_broadcast(true)?;
    let mut buf = [0; BUFFER_SIZE];
    let addresses = prepare(&mut socket, &mut buf[..]).await?;
    println!("homesec-bootstrap.service updated on {} devices", addresses.len());
    let timeout = Duration::from_secs(240);
    let start = SystemTime::now();
    println!("waiting for master election");
    let mut votes = HashMap::<SocketAddr, HashSet<SocketAddr>>::new();
    let mut appearances = HashSet::<SocketAddr>::new();
    let mut result_count = 0;
    let mut master = None;
    let mut election_results = Vec::new();
    loop {
        let elapsed = SystemTime::now().duration_since(start).unwrap();
        if elapsed > timeout {
            break;
        }
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                let msg: Message = bincode::deserialize(&buf[..n])?;
                match msg {
                    Message::Reset => {
                        return Err(anyhow!("observed unexpected election reset"));
                    },
                    Message::Appearance(msg) => {
                        if appearances.insert(addr) {
                            println!("appearance from {}, is_master={}, hid={}, priority={}", addr, msg.is_master, msg.hid, msg.priority);
                        }
                    },
                    Message::CastVote(vote) => {
                        let cast = match votes.get_mut(&vote.addr) {
                            Some(votes) => votes.insert(addr),
                            None => {
                                let mut v = HashSet::<SocketAddr>::new();
                                v.insert(addr);
                                votes.insert(vote.addr, v);
                                true
                            }
                        };
                        if cast {
                            println!("{} cast vote for {}", addr, vote.addr);
                        }
                    }
                    Message::ConnectionDetails(details) => {
                        if let Some(master) = master {
                            return Err(anyhow!("observed multiple ConnectionDetails from {} and {}", master, addr));
                        } else {
                            println!("observed k3s connection details for master, addr={}, hid={}, token={}", addr, details.hid, &details.token);
                            master = Some(addr);
                        }
                        if result_count == addresses.len() {
                            break;
                        }
                    }
                    Message::ElectionResult(result) => {
                        election_results.push(result.addr);
                        println!("observed election result, source={}, candidate={}, hid={}", addr, result.addr, result.hid);
                        result_count += 1;
                        if result_count == addresses.len() && master.is_some() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(Error::from(e)),
        }
    }
    if appearances.len() != addresses.len() {
        return Err(anyhow!("only {}/{} nodes broadcasted appearance messages", appearances.len(), addresses.len()));
    }
    if result_count != addresses.len() {
        return Err(anyhow!("only {}/{} nodes concluded election", result_count, addresses.len()));
    }
    if master.is_none() {
        return Err(anyhow!("failed to get connection details from master"));
    }
    let master = master.unwrap();
    let vote_count = votes.get(&master).unwrap_or(&HashSet::new()).len();
    if vote_count < addresses.len() {
        return Err(anyhow!("master received only {}/{} votes before observing result", vote_count, addresses.len()));
    }
    if election_results.iter().any(|addr| addr != &master) {
        return Err(anyhow!("nodes disagree on outcome of election"));
    }
    let master = match master {
        SocketAddr::V4(addr) => addr.ip().to_string(),
        SocketAddr::V6(addr) => addr.ip().to_string(),
    };
    let output = Command::new("ssh")
        .args(&[
            &format!("pi@{}", &master),
            "sudo",
            "chmod",
            "644",
            "/etc/rancher/k3s/k3s.yaml",
        ])
        .output()?;
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        return Err(anyhow!("command failed with exit code {}", output.status));
    }
    let kubeconfig = "/tmp/k3s-config";
    let output = Command::new("scp")
        .args(&[&format!("pi@{}:/etc/rancher/k3s/k3s.yaml", &master), kubeconfig])
        .output()?;
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        return Err(anyhow!("command failed with exit code {}", output.status));
    }
    println!("copied kubeconfig from master to {}", kubeconfig);
    std::fs::write(kubeconfig, std::fs::read_to_string(kubeconfig)?.replace("127.0.0.1", &master))?;
    println!("applying kube-system pod security policy");
    let output = Command::new("kubectl")
        .env("KUBECONFIG", kubeconfig)
        .current_dir("../extra")
        .args(&[
            "apply",
            "-f",
            "psp.yaml",
        ])
        .output()?;
    std::io::stdout().write_all(&output.stdout).unwrap();
    if !output.status.success() {
        std::io::stderr().write_all(&output.stderr).unwrap();
        return Err(anyhow!("command failed with exit code {}", output.status));
    }
    std::env::set_var("KUBECONFIG", kubeconfig);
    println!("waiting for nodes");
    wait_for_nodes(&addresses[..]).await?;
    println!("success");
    Ok(())
}

async fn wait_for_nodes(addresses: &[String]) -> Result<()> {
    let client = Client::infer().await.unwrap();
    let nodes = Api::<v1::Node>::all(client.clone());
    let delay = Duration::from_secs(5);
    let start = SystemTime::now();
    let timeout = Duration::from_secs(180);
    let mut last_ready = 0;
    loop {
        let elapsed = SystemTime::now().duration_since(start).unwrap();
        if elapsed > timeout {
            return Err(anyhow!("failed to observe all nodes within {:?}", timeout));
        }
        let nodes = nodes.list(&kube::api::ListParams::default()).await?;
        let ready = nodes.items.iter().filter(|node| {
            if let Some(status) = &node.status {
                if let Some(conditions) = &status.conditions {
                    for condition in conditions {
                        if let Some(reason) = &condition.reason {
                            if reason == "KubeletReady" {
                                return condition.status == "True";
                            }
                        }
                    }
                }
            }
            false
        }).collect::<Vec<_>>();
        if ready.len() != last_ready {
            println!("{}/{} nodes ready", ready.len(), addresses.len());
            last_ready = ready.len();
        }
        if ready.len() == addresses.len() {
            return Ok(());
        }
        std::thread::sleep(delay);
        continue;
    }
}