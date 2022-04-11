use log::info;
use netsim_embed::*;
use netsim_embed_machine::Namespace;
use sim::cmd;
use sim::Node;
use sim::Simulator;
use std::thread;
use std::time;

async fn wait_ready(nodes: &[&Node]) {
    info!("Waiting for nodes ready {:?}", nodes);

    // TODO: There is no sleep in async_global_executor.
    // And we also should wait by node info, not just duration.

    thread::sleep(time::Duration::from_secs(15));
}

async fn test_spawn_global_node() {
    let sim = Simulator::new();
    let node = sim.spawn_global_node().await;
    let node_addr = node.address().await;
    let node_ns = node.namespace().await;

    info!("Node listen {}:{}", node_addr.to_string(), node.port);

    info!("Current namespace {}", Namespace::current().unwrap());
    cmd::ifconfig().await;

    node_ns.enter().unwrap();

    info!("Enter namespace {}", node_ns);
    cmd::ifconfig().await;

    wait_ready(&[&node]).await;

    cmd::ping(&node_addr).await;
    cmd::curl_get(&node_addr, node.port).await;
}

async fn test_spawn_nat_node() {
    let sim = Simulator::new();
    let node = sim.spawn_nat_node().await;
    let node_addr = node.address().await;
    let node_ns = node.namespace().await;

    info!("Node listen {}:{}", node_addr.to_string(), node.port);

    info!("Current namespace {}", Namespace::current().unwrap());
    cmd::ifconfig().await;

    node_ns.enter().unwrap();

    info!("Enter namespace {}", node_ns);
    cmd::ifconfig().await;

    wait_ready(&[&node]).await;

    cmd::ping(&node_addr).await;
    cmd::curl_get(&node_addr, node.port).await;
}

async fn test_handshake() {
    let sim = Simulator::new();

    let node1 = sim.spawn_nat_node().await;
    let node1_addr = node1.address().await;
    info!("Node1 listen {}:{}", node1_addr.to_string(), node1.port);

    let node2 = sim.spawn_nat_node().await;
    let node2_addr = node2.address().await;
    info!("Node2 listen {}:{}", node2_addr.to_string(), node2.port);

    wait_ready(&[&node1, &node2]).await;
}

fn main() {
    env_logger::init();
    run(async { test_spawn_global_node().await });
    run(async { test_spawn_nat_node().await });
    run(async { test_handshake().await });
}
