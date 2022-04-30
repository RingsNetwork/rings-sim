#[cfg(test)]
pub mod test {
    use anyhow::Result;
    use bns_node::cli::Client;
    use ring_sim::*;

    #[tokio::test]
    async fn test_connect_peer_via_http_in_same_nat() -> Result<()> {
        let nat = create_nat()?;
        let node1 = create_node(&nat, Some("50000"))?;
        let node2 = create_node(&nat, Some("50000"))?;
        println!("{:?}", nat);
        println!("{:?}", node1);
        println!("{:?}", node2);

        let ep1 = format!("http://127.0.0.1:{}", node1.pub_port.unwrap());
        let ep2 = format!("http://127.0.0.1:{}", node2.pub_port.unwrap());
        let node2_lan_url = format!("http://{}:50000", node2.lan_ip);

        let mut cli1 = Client::new(&ep1).await?;
        let mut cli2 = Client::new(&ep2).await?;

        let node1_peers = cli1.list_peers().await?.result;
        let node2_peers = cli2.list_peers().await?.result;
        assert_eq!(node1_peers.len(), 0);
        assert_eq!(node2_peers.len(), 0);

        let transport_id_of_node2_on_node1 =
            cli1.connect_peer_via_http(&node2_lan_url).await?.result;

        let node1_peers = cli1.list_peers().await?.result;
        assert_eq!(node1_peers.len(), 1);
        assert_eq!(node1_peers[0].transport_id, transport_id_of_node2_on_node1);
        assert_eq!(node1_peers[0].address, node2.address()?);

        let node2_peers = cli2.list_peers().await?.result;
        assert_eq!(node2_peers.len(), 1);
        assert_eq!(node2_peers[0].address, node1.address()?);

        Ok(())
    }
}
