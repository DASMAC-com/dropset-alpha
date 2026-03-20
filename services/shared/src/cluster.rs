
    /// Resolves which Solana cluster the RPC is connected to by matching
    /// the genesis hash. Refuses to operate against mainnet-beta.
    ///
    /// Returns the detected [`ClusterType`] for logging/display.
    pub async fn resolve_cluster(&self) -> anyhow::Result<ClusterType> {
        let genesis = self
            .rpc
            .client
            .get_genesis_hash()
            .await
            .context("Failed to fetch genesis hash")?;

        let cluster = [
            ClusterType::MainnetBeta,
            ClusterType::Testnet,
            ClusterType::Devnet,
        ]
        .into_iter()
        .find(|c| c.get_genesis_hash().is_some_and(|h| h == genesis))
        .unwrap_or(ClusterType::Development);

        anyhow::ensure!(
            cluster != ClusterType::MainnetBeta,
            "Refusing to operate against mainnet-beta. \
             The faucet is only for testnet/devnet/localnet."
        );

        Ok(cluster)
    }