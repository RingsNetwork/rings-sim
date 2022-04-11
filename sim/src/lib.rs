pub mod cmd;

use netsim_embed::Ipv4Range;
use netsim_embed::MachineId;
use netsim_embed::NatConfig;
use netsim_embed::Netsim;
use netsim_embed::NetworkId;
use netsim_embed_machine::Machine;
use netsim_embed_machine::Namespace;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Mutex;

pub struct Simulator {
    driver: Netsim<String, String>,
    next_port_flag: Mutex<u16>,
}

pub struct Node<'a> {
    machine: &'a Machine<String, String>,

    pub lnet_id: Option<NetworkId>,
    pub gnet_id: Option<NetworkId>,

    pub machine_id: MachineId,
    pub port: u16,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            driver: Netsim::<String, String>::new(),
            next_port_flag: Mutex::new(50000),
        }
    }

    fn next_port(&self) -> u16 {
        let mut port = self.next_port_flag.lock().unwrap();
        *port += 1;
        *port
    }

    pub async fn spawn_global_node<'a>(&'a mut self) -> Node<'a> {
        let port = self.next_port();
        let address = Ipv4Addr::from_str("0.0.0.0").unwrap();

        let node = cmd::build_spawn_node_cmd(&address, port);
        let machine_id = self.driver.spawn_machine(node, None).await;

        // setup network with global ip
        let net = self.driver.spawn_network(Ipv4Range::global());

        // add machine to net
        self.driver.plug(machine_id, net, None).await;

        Node {
            machine: self.driver.machine(machine_id),
            gnet_id: Some(net),
            lnet_id: None,
            machine_id,
            port,
        }
    }

    pub async fn spawn_nat_node<'a>(&'a mut self) -> Node<'a> {
        let port = self.next_port();
        let address = Ipv4Addr::from_str("0.0.0.0").unwrap();

        let node = cmd::build_spawn_node_cmd(&address, port);
        let machine_id = self.driver.spawn_machine(node, None).await;

        // setup network with global ip
        let gnet_id = self.driver.spawn_network(Ipv4Range::global());

        // setup network with local ip
        let lnet_id = self.driver.spawn_network(Ipv4Range::random_local_subnet());

        // add machine to net
        let nat_config = NatConfig::default();
        self.driver.plug(machine_id, lnet_id, None).await;
        self.driver.add_nat_route(nat_config, gnet_id, lnet_id);

        Node {
            machine: self.driver.machine(machine_id),
            gnet_id: Some(gnet_id),
            lnet_id: Some(lnet_id),
            machine_id,
            port,
        }
    }
}

impl<'a> Node<'a> {
    pub fn address(&self) -> Ipv4Addr {
        self.machine.addr()
    }

    pub fn namespace(&self) -> Namespace {
        self.machine.namespace()
    }
}
