use bns_node::cli::Client;
use futures::channel::oneshot;
use log::info;
use netsim_embed::*;
use netsim_embed_core::{wire, DelayBuffer};
use netsim_embed_machine::Namespace;
use sim::cmd;
use sim::Node;
use sim::Simulator;
use std::future::Future;
use std::time;
use tokio::runtime::Runtime;

async fn sleep(duration: time::Duration) {
    info!("sleep {:?}", duration);
    let mut delay = DelayBuffer::new();
    delay.set_delay(duration);

    let (mut plug_a, plug_b) = wire();
    let mut plug_d = delay.spawn(plug_b);

    plug_a.unbounded_send(Vec::from([u8::default()]));
    plug_d.incoming().await;
    info!("sleep {:?} done", duration);
}

async fn wait_ready(nodes: &[&Node]) {
    info!("Waiting for nodes ready {:?}", nodes);

    sleep(time::Duration::from_secs(10)).await;
    sleep(time::Duration::from_secs(10)).await;
    sleep(time::Duration::from_secs(10)).await;
    sleep(time::Duration::from_secs(10)).await;
    sleep(time::Duration::from_secs(10)).await;
    sleep(time::Duration::from_secs(10)).await;

    // TODO: We also should wait by node info, not just duration.
    for n in nodes {
        cmd::ping(&n.addr).await;
        cmd::curl_get(&n.addr, 50000).await;
    }
}

fn tk_run<T, F>(runtime: &Runtime, future: F) -> oneshot::Receiver<T>
where
    F: Future<Output = T> + Send + 'static,
    T: std::fmt::Debug + Send + 'static,
{
    let (tx, rx) = oneshot::channel::<T>();
    runtime.spawn(async {
        tx.send(future.await).unwrap();
    });
    rx
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
    let node1_url = node1.endpoint_url();
    info!("Node1 listen {}", node1.endpoint_url());

    let node2 = sim.spawn_node(net).await;
    info!("Node2 listen {}", node2.endpoint_url());

    let node3 = sim.spawn_node(net).await;
    info!("Node3 listen {}", node3.endpoint_url());

    info!("Enter node1 namespace");
    node1.namespace().await.enter()?;

    wait_ready(&[&node1, &node2, &node3]).await;

    let rx = tk_run(&tkrt, async move {
        info!("Connect to node1");
        let mut cli = Client::new(&node1.endpoint_url()).await?;

        info!("Node1 connect None2 via http: {}", node2.endpoint_url());
        let node2_transport_id = cli
            .connect_peer_via_http(&node2.endpoint_url())
            .await?
            .result;

        anyhow::Ok::<String>(node2_transport_id)
    });

    let node2_transport_id = rx.await??;
    info!("Node2 connected id: {}", node2_transport_id);

    /*
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
    */

    Ok(())
}

fn main() {
    env_logger::init();

    // run(async { test_spawn_node().await.unwrap(); });
    run(async {
        test_handshake().await.unwrap();
    });
}
