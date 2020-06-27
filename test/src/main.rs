#[macro_use]
extern crate anyhow;

use anyhow::Result;
use std::net::SocketAddr;
use std::process::Command;
use std::io::{self, Write};

fn build_for_arm() -> Result<()> {
    let output = Command::new("sh")
        .args(&[
            "-c",
            "rm ../target/armv7-unknown-linux-gnueabihf/debug/homesec-bootstrap || true && cargo build --target armv7-unknown-linux-gnueabihf",
        ])
        .current_dir("../bootstrap")
        .output()
        .expect("build failed");
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        Err(anyhow!("command failed with exit code {}", output.status))
    } else {
        println!("successfully built bootstrap");
        Ok(())
    }
}

fn get_addresses() -> Result<Vec<String>> {
    Ok(vec![String::from("192.168.0.102")])
}

fn install_bootstrap(addresses: &[String]) -> Result<()> {
    for address in addresses {
        println!("installing to {}", address);
        let dest = format!("pi@{}:/home/pi/homesec-bootstrap", address);
        let output = Command::new("scp")
            .args(&["../target/armv7-unknown-linux-gnueabihf/debug/homesec-bootstrap", &dest])
            .current_dir("../bootstrap")
            .output()
            .expect("build failed");
        if !output.status.success() {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            return Err(anyhow!("command failed with exit code {}", output.status));
        }
        // TODO: return error from script
        let encoded = base64::encode("set -e; rm /usr/bin/homesec-bootstrap || true && mv /home/pi/homesec-bootstrap /usr/bin/homesec-bootstrap && systemctl restart homesec-bootstrap.service");
        //let encoded = base64::encode("exit 1");
        let output = Command::new("ssh")
            .args(&[
                &format!("pi@{}", &address),
                "bash",
                "-c",
                &format!("set -e; echo \"{}\" | base64 --decode | sudo bash -", &encoded),
            ])
            .output()
            .expect("mv failed");
        if !output.status.success() {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            return Err(anyhow!("command failed with exit code {}", output.status));
        }
        println!("successfully installed bootstrap to {}", address);
    }
    Ok(())
}

fn prepare() -> Result<Vec<String>> {
    let addresses = get_addresses()?;
    build_for_arm()?;
    install_bootstrap(&addresses[..])?;
    Ok(addresses)
}

fn main() -> Result<()> {
    let port = 43000;
    let addresses = prepare()?;
    println!("bootstrap.service updated on all devices");
    Ok(())
}
