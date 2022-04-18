pub mod cmd;

use futures::lock::Mutex as AsyncMutex;
use log::info;
use netsim_embed::Ipv4Range;
use netsim_embed::MachineId;
use netsim_embed::NatConfig;
use netsim_embed::Netsim;
use netsim_embed::NetworkId;
use netsim_embed_machine::Namespace;
use std::fmt::Debug;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::runtime::Runtime as TokioRuntime;

pub struct Simulator {
    driver: Arc<AsyncMutex<Netsim<String, String>>>,
}

pub struct Node {
    driver: Arc<AsyncMutex<Netsim<String, String>>>,
    machine: MachineId,

    pub net: NetworkId,
    pub addr: Ipv4Addr,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            driver: Arc::new(AsyncMutex::new(Netsim::new())),
        }
    }

    pub async fn spawn_network(&self, range: Option<Ipv4Range>) -> NetworkId {
        let mut driver = self.driver.lock().await;

        let range = range.unwrap_or_else(|| Ipv4Range::global().split(2)[0]);
        let net = driver.spawn_network(range);

        net
    }

    pub async fn spawn_node(&self, net: NetworkId) -> Node {
        let mut driver = self.driver.lock().await;

        let node_cmd = cmd::build_spawn_node_cmd();
        let machine = driver.spawn_machine(node_cmd, None).await;

        let addr = driver.network_mut(net).unique_addr();
        driver.plug(machine, net, Some(addr)).await;

        Node {
            driver: self.driver.clone(),
            machine,
            net,
            addr,
        }
    }
}

impl Node {
    pub async fn enter_namespace(&self) -> anyhow::Result<(Namespace, TokioRuntime)> {
        let mut driver = self.driver.lock().await;
        let ns = driver.machine(self.machine).namespace();

        info!("Enter {:?} namespace {}", self, ns);
        ns.enter()?;
        let tkrt = TokioRuntime::new()?;

        Ok((ns, tkrt))
    }

    pub fn endpoint_url(&self) -> String {
        format!("http://{}:50000", self.addr)
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node(machine_id={:?}, lnet={:?}, laddr={:?})",
            self.machine, self.net, self.addr,
        )
    }
}
