pub mod cmd;

use futures::lock::Mutex as AsyncMutex;
use netsim_embed::Ipv4Range;
use netsim_embed::MachineId;
use netsim_embed::NatConfig;
use netsim_embed::Netsim;
use netsim_embed::NetworkId;
use netsim_embed_machine::Namespace;
use std::fmt::Debug;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub struct Simulator {
    driver: Arc<AsyncMutex<Netsim<String, String>>>,
    next_port_flag: Mutex<u16>,
}

pub struct Node {
    driver: Arc<AsyncMutex<Netsim<String, String>>>,
    machine_id: MachineId,

    pub lnet_id: Option<NetworkId>,
    pub gnet_id: Option<NetworkId>,

    pub port: u16,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            driver: Arc::new(AsyncMutex::new(Netsim::new())),
            next_port_flag: Mutex::new(50000),
        }
    }

    fn next_port(&self) -> u16 {
        let mut port = self.next_port_flag.lock().unwrap();
        *port += 1;
        *port
    }

    pub async fn spawn_global_node(&self) -> Node {
        let mut driver = self.driver.lock().await;

        let port = self.next_port();
        let address = Ipv4Addr::from_str("0.0.0.0").unwrap();

        let node_cmd = cmd::build_spawn_node_cmd(&address, port);
        let machine_id = driver.spawn_machine(node_cmd, None).await;

        // setup network with global ip
        let net = driver.spawn_network(Ipv4Range::global());

        // add machine to net
        driver.plug(machine_id, net, None).await;

        Node {
            driver: self.driver.clone(),
            gnet_id: Some(net),
            lnet_id: None,
            machine_id,
            port,
        }
    }

    pub async fn spawn_nat_node(&self) -> Node {
        let mut driver = self.driver.lock().await;

        let port = self.next_port();
        let address = Ipv4Addr::from_str("0.0.0.0").unwrap();

        let node_cmd = cmd::build_spawn_node_cmd(&address, port);
        let machine_id = driver.spawn_machine(node_cmd, None).await;

        // setup network with global ip
        let gnet_id = driver.spawn_network(Ipv4Range::global());

        // setup network with local ip
        let lnet_id = driver.spawn_network(Ipv4Range::random_local_subnet());

        // add machine to net
        let nat_config = NatConfig::default();
        driver.plug(machine_id, lnet_id, None).await;
        driver.add_nat_route(nat_config, gnet_id, lnet_id);

        Node {
            driver: self.driver.clone(),
            gnet_id: Some(gnet_id),
            lnet_id: Some(lnet_id),
            machine_id,
            port,
        }
    }
}

impl Node {
    pub async fn address(&self) -> Ipv4Addr {
        let mut driver = self.driver.lock().await;
        driver.machine(self.machine_id).addr()
    }

    pub async fn namespace(&self) -> Namespace {
        let mut driver = self.driver.lock().await;
        driver.machine(self.machine_id).namespace()
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node(machine_id={:?})", self.machine_id)
    }
}
