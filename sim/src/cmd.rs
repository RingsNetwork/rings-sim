use async_process::Command;
use bns_core::ecc::SecretKey;
use log::info;
use std::net::Ipv4Addr;

pub fn build_spawn_node_cmd() -> Command {
    let key = SecretKey::random();

    let mut cmd = Command::new("cargo");
    cmd.args(&[
        "run",
        "--manifest-path",
        "bns-node/Cargo.toml",
        "--",
        "run",
        "-k",
        &key.to_string(),
        "-b",
        &format!("0.0.0.0:50000"),
    ]);

    cmd
}

pub async fn ifconfig() -> Command {
    let mut cmd = Command::new("ifconfig");

    info!("runing ifconfig");
    let output = cmd.output().await.unwrap();

    info!("stdout:\n{}", std::str::from_utf8(&output.stdout).unwrap());
    info!("stderr:\n{}", std::str::from_utf8(&output.stderr).unwrap());

    cmd
}

pub async fn ping(address: &Ipv4Addr) -> Command {
    let mut cmd = Command::new("ping");
    let args = ["-c", "5", &address.to_string()];
    cmd.args(&args);

    info!("runing ping {:?}", args);
    let output = cmd.output().await.unwrap();

    info!("stdout:\n{}", std::str::from_utf8(&output.stdout).unwrap());
    info!("stderr:\n{}", std::str::from_utf8(&output.stderr).unwrap());

    cmd
}

pub async fn curl_get(address: &Ipv4Addr, port: u16) -> Command {
    let mut cmd = Command::new("curl");
    let args = [
        "-v".to_string(),
        "--max-time".to_string(),
        "10".to_string(),
        format!("http://{}:{}/sdp", address.to_string(), port),
    ];
    cmd.args(&args);

    info!("runing curl {:?}", args);
    let output = cmd.output().await.unwrap();

    info!("stdout:\n{}", std::str::from_utf8(&output.stdout).unwrap());
    info!("stderr:\n{}", std::str::from_utf8(&output.stderr).unwrap());

    cmd
}
