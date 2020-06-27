#[macro_use]
extern crate anyhow;
use anyhow::Result;
use std::net::SocketAddr;
use std::process::Command;
use std::io::{self, Write};

fn build_for_arm() -> Result<()> {
    let output = Command::new("cargo")
        .args(&["build", "--target", "armv7-unknown-linux-gnueabihf"])
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
        } else {
            println!("successfully installed bootstrap to {}", address);
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let addresses = get_addresses()?;
    build_for_arm()?;
    install_bootstrap(&addresses[..])?;
    Ok(())
}
