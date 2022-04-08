use netsim_embed::*;
use netsim_embed_machine::Namespace;
use sim::Simulator;

fn main() {
    run(async {
        env_logger::init();
        let mut sim = Simulator::new();
        let (nid, mid) = sim.spawn_global_node(4242).await;

        let server_addr = sim.get_address(mid);

        println!("Server Addr {}:50000", server_addr.to_string());

        // let mut cmd = Command::new("curl");
        // cmd.args(&["http://{}:4242/sdp", server_addr.to_string(), "4242"]);
        // let output = cmd.output().await.unwrap();
        // println!("response: {}", std::str::from_utf8(&output.stdout).unwrap());
        //println!("error: {}", std::str::from_utf8(&output.stderr).unwrap());
    });
}
