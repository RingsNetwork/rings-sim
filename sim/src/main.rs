use log::info;
use netsim_embed::*;
use netsim_embed_machine::Namespace;
use sim::cmd;
use sim::Simulator;
use std::thread;
use std::time;

async fn test_network_available() {
    let mut sim = Simulator::new();
    let node = sim.spawn_global_node().await;

    info!("Server Addr {}:{}", node.address().to_string(), node.port);

    info!("Current namespace {}", Namespace::current().unwrap());
    cmd::ifconfig().await;

    node.namespace().enter().unwrap();

    info!("Enter namespace {}", node.namespace());
    cmd::ifconfig().await;

    // Wait for node ready.
    thread::sleep(time::Duration::from_secs(15));

    cmd::ping(&node.address()).await;
    cmd::curl_get(&node.address(), node.port).await;
}

fn main() {
    env_logger::init();
    run(async { test_network_available().await });
}
