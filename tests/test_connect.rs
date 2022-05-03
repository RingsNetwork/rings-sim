#[cfg(test)]
pub mod test {
    use anyhow::{Ok, Result};
    use bns_node::cli::Client;
    use ring_sim::*;

    async fn test_connect_peer_via_http(node1: &Node, node2: &Node) -> Result<()> {
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

    async fn test_connect_peer_via_sdp(node1: &Node, node2: &Node) -> Result<()> {
        let ep1 = format!("http://127.0.0.1:{}", node1.pub_port.unwrap());
        let ep2 = format!("http://127.0.0.1:{}", node2.pub_port.unwrap());

        let mut cli1 = Client::new(&ep1).await?;
        let mut cli2 = Client::new(&ep2).await?;

        let node1_peers = cli1.list_peers().await?.result;
        let node2_peers = cli2.list_peers().await?.result;
        assert_eq!(node1_peers.len(), 0);
        assert_eq!(node2_peers.len(), 0);

        let trans_and_ice1 = cli1.create_offer().await?.result;
        let trans_and_ice2 = cli2.connect_peer_via_ice(&trans_and_ice1.ice).await?.result;
        let peer = cli1
            .accept_answer(&trans_and_ice1.transport_id, &trans_and_ice2.ice)
            .await?
            .result;

        assert_eq!(peer.transport_id, trans_and_ice1.transport_id);

        let node1_peers = cli1.list_peers().await?.result;
        assert_eq!(node1_peers.len(), 1);
        assert_eq!(node1_peers[0].transport_id, trans_and_ice1.transport_id);
        assert_eq!(node1_peers[0].address, node2.address()?);

        let node2_peers = cli2.list_peers().await?.result;
        assert_eq!(node2_peers.len(), 1);
        assert_eq!(node2_peers[0].transport_id, trans_and_ice2.transport_id);
        assert_eq!(node2_peers[0].address, node1.address()?);

        Ok(())
    }

    #[tokio::test]
    async fn test_connect_peer_via_http_in_same_nat() -> Result<()> {
        let nat = create_nat(false)?;
        let node1 = create_node(&nat, Some("50000"))?;
        let node2 = create_node(&nat, Some("50000"))?;

        test_connect_peer_via_http(&node1, &node2).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_connect_peer_via_sdp_in_same_nat() -> Result<()> {
        let nat = create_nat(false)?;
        let node1 = create_node(&nat, Some("50000"))?;
        let node2 = create_node(&nat, Some("50000"))?;

        test_connect_peer_via_sdp(&node1, &node2).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_connect_peer_via_sdp_in_different_nat() -> Result<()> {
        let nat1 = create_nat(false)?;
        let nat2 = create_nat(false)?;
        let node1 = create_node(&nat1, Some("50000"))?;
        let node2 = create_node(&nat2, Some("50000"))?;

        test_connect_peer_via_sdp(&node1, &node2).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_connect_peer_via_sdp_in_different_symmetric_nat() -> Result<()> {
        let nat1 = create_nat(true)?;
        let nat2 = create_nat(true)?;
        let node1 = create_node(&nat1, Some("50000"))?;
        let node2 = create_node(&nat2, Some("50000"))?;

        test_connect_peer_via_sdp(&node1, &node2).await?;

        Ok(())
    }
}
