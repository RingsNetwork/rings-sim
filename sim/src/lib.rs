use async_process::Command;
use bns_core::ecc::SecretKey;
use netsim_embed::Netsim;
use netsim_embed::Ipv4Range;
use netsim_embed::MachineId;
use netsim_embed::NetworkId;
use std::net::Ipv4Addr;
use netsim_embed::NatConfig;

pub struct Simulator {
    pub driver: Netsim<String, String>
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            driver: Netsim::<String, String>::new()
        }
    }

    pub fn spawn_node(port: usize) -> Command {
        let key = SecretKey::random();
        let mut process = Command::new("cargo");
        process.args(&[
            "run",
            "--manifest-path",
            "bns-node/Cargo.toml",
            "--",
            "run",
            "-k",
            &key.to_string(),
            "-b",
            &format!("0.0.0.0:{}", port)
        ]);
        process
    }

    pub async fn spawn_global_node(&mut self, port: usize) -> (NetworkId, MachineId) {
        let node = Self::spawn_node(port);
        // setup network with global ip
        let net = self.driver.spawn_network(Ipv4Range::global());
        let machine_id = self.driver.spawn_machine(node, None).await;
        // add machine to net
        self.driver.plug(machine_id, net, None).await;
        (net, machine_id)
    }

    pub async fn spawn_nat_node(&mut self, port: usize) -> (NetworkId, NetworkId, MachineId) {
         let node = Self::spawn_node(port);
        // setup network with global ip
        let gnet_id = self.driver.spawn_network(Ipv4Range::global());
        // setup network with local ip
        let lnet_id = self.driver.spawn_network(Ipv4Range::random_local_subnet());
        let machine_id = self.driver.spawn_machine(node, None).await;
        // add machine to net
        self.driver.plug(machine_id, lnet_id, None).await;

        let nat_config = NatConfig::default();
        self.driver.add_nat_route(
            nat_config,
            gnet_id,
            lnet_id
        );
        (gnet_id, lnet_id, machine_id)
    }

    pub fn get_address(&mut self, mid: MachineId) -> Ipv4Addr {
        self.driver.machine(mid).addr()
    }
}
