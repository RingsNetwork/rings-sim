use anyhow::{anyhow, Result};
use ring_sim::*;
use std::process::{Command, Stdio};

pub fn create_coturn_node(nat: &Nat) -> Result<Node> {
    let co = Command::new("python")
        .args([
            "nind.py",
            "-f",
            "json",
            "create_node",
            "-l",
            &nat.lan,
            "-r",
            &nat.router,
            "--node-image",
            "bnsnet/coturn",
            "sleep",
            "infinity",
        ])
        .stderr(Stdio::inherit())
        .output()?;

    serde_json::from_str(std::str::from_utf8(&co.stdout)?).map_err(|e| anyhow!(e))
}

pub fn get_behavior(node: &Node, args: &[&str]) -> Result<String> {
    let co = Command::new("docker")
        .args(["exec", &node.name, "turnutils_natdiscovery"])
        .args(args)
        .stderr(Stdio::inherit())
        .output()?;

    String::from_utf8(co.stdout).map_err(|e| anyhow!(e))
}

#[test]
fn test_port_restricted_cone_nat() -> Result<()> {
    let nat = create_nat(false)?;
    let node = create_coturn_node(&nat)?;

    let mapping_behavior = get_behavior(&node, &["-m", "172.31.0.200"])?;
    let filter_behavior = get_behavior(&node, &["-f", "172.31.0.200"])?;

    assert_eq!(
        "NAT with Endpoint Independent Mapping!",
        mapping_behavior.rsplitn(4, '\n').nth(2).unwrap()
    );
    assert_eq!(
        "NAT with Address and Port Dependent Filtering!",
        filter_behavior.rsplitn(4, '\n').nth(2).unwrap()
    );

    Ok(())
}

#[test]
fn test_symetric_nat() -> Result<()> {
    let nat = create_nat(true)?;
    let node = create_coturn_node(&nat)?;

    let mapping_behavior = get_behavior(&node, &["-m", "172.31.0.200"])?;
    let filter_behavior = get_behavior(&node, &["-f", "172.31.0.200"])?;

    assert_eq!(
        "NAT with Address and Port Dependent Mapping!",
        mapping_behavior.rsplitn(4, '\n').nth(2).unwrap()
    );
    assert_eq!(
        "NAT with Address and Port Dependent Filtering!",
        filter_behavior.rsplitn(4, '\n').nth(2).unwrap()
    );

    Ok(())
}
