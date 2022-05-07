use anyhow::{anyhow, Result};
use rings_core::ecc::SecretKey;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Nat {
    pub lan: String,
    pub router: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Node {
    pub name: String,
    pub router: String,
    pub lan: String,
    pub key: String,
    pub lan_ip: String,
    pub pub_port: Option<u16>,
}

impl Node {
    pub fn address(&self) -> Result<String> {
        let key = SecretKey::try_from(self.key.as_str())?;
        Ok(key.address().to_string())
    }
}

pub fn create_nat(symmetric: bool) -> Result<Nat> {
    let mut c = Command::new("python");
    c.args(["nind.py", "-f", "json", "create_nat"]);

    if symmetric {
        c.args(["--symmetric"]);
    }

    let co = c.stderr(Stdio::inherit()).output()?;

    serde_json::from_str(std::str::from_utf8(&co.stdout)?).map_err(|e| anyhow!(e))
}

pub fn create_node(nat: &Nat, publish: Option<&str>) -> Result<Node> {
    let mut c = Command::new("python");
    c.args([
        "nind.py",
        "-f",
        "json",
        "create_node",
        "-l",
        &nat.lan,
        "-r",
        &nat.router,
    ]);

    if let Some(port) = publish {
        c.args(["-p", port]);
    }

    let co = c.stderr(Stdio::inherit()).output()?;

    serde_json::from_str(std::str::from_utf8(&co.stdout)?).map_err(|e| anyhow!(e))
}
