use bns_node::cli::Client;
use log::info;
use netsim_embed::*;
use netsim_embed_machine::Namespace;
use sim::cmd;
use sim::Node;
use sim::Simulator;
use std::thread;
use std::time;
use tokio::runtime::Runtime;

async fn wait_ready(nodes: &[&Node]) {
    info!("Waiting for nodes ready {:?}", nodes);

    // TODO: There is no sleep in async_global_executor.
    // And we also should wait by node info, not just duration.

    thread::sleep(time::Duration::from_secs(20));
    for n in nodes {
        cmd::ping(&n.addr).await;
        cmd::curl_get(&n.addr, 50000).await;
    }
}

async fn test_spawn_node() -> anyhow::Result<()> {
    let sim = Simulator::new();
    let net = sim.spawn_network(None).await;

    let node1 = sim.spawn_node(net).await;
    info!("Node1 listen {}", node1.endpoint_url());

    let node2 = sim.spawn_node(net).await;
    info!("Node2 listen {}", node2.endpoint_url());

    info!("Current namespace {}", Namespace::current()?);
    cmd::ifconfig().await;

    let node1_ns = node1.namespace().await;
    info!("Enter node1 namespace {}", node1_ns);
    node1_ns.enter()?;
    cmd::ifconfig().await;

    wait_ready(&[&node1, &node2]).await;

    Ok(())
}

async fn test_handshake() -> anyhow::Result<()> {
    let tkrt = Runtime::new().unwrap();
    let sim = Simulator::new();
    let net = sim.spawn_network(None).await;

    let node1 = sim.spawn_node(net).await;
    info!("Node1 listen {}", node1.endpoint_url());

    let node2 = sim.spawn_node(net).await;
    info!("Node2 listen {}", node2.endpoint_url());

    let node3 = sim.spawn_node(net).await;
    info!("Node3 listen {}", node3.endpoint_url());

    info!("Enter node1 namespace");
    node1.namespace().await.enter()?;

    wait_ready(&[&node1, &node2, &node3]).await;

    let mut node1_cli = tkrt.block_on(Client::new(&node1.endpoint_url()))?;

    info!("Node1 connect None2 via http: {}", node2.endpoint_url());
    let node2_transport_id = tkrt
        .block_on(node1_cli.connect_peer_via_http(&node2.endpoint_url()))?
        .result;

    info!("Node1 connect None3 via http: {}", node3.endpoint_url());
    let node3_transport_id = tkrt
        .block_on(node1_cli.connect_peer_via_http(&node3.endpoint_url()))?
        .result;

    let peers = tkrt.block_on(node1_cli.list_peers(true))?.result;
    info!("Node1 list all peers: {:?}", peers);

    Ok(())
}

fn main() {
    env_logger::init();

    run(async {
        test_spawn_node().await.unwrap();
    });
    run(async {
        test_handshake().await.unwrap();
    });
}
