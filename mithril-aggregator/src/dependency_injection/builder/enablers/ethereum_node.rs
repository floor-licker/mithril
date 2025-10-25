use std::sync::Arc;

use mithril_ethereum_chain::EthereumChainObserver;
use mithril_universal::UniversalChainObserver;

use crate::dependency_injection::{DependenciesBuilder, Result};

impl DependenciesBuilder {
    /// Build an Ethereum chain observer if enabled in configuration
    pub(crate) async fn build_ethereum_chain_observer(
        &mut self,
    ) -> Result<Option<Arc<dyn UniversalChainObserver>>> {
        if !self.configuration.enable_ethereum_observer() {
            return Ok(None);
        }

        let endpoint = self
            .configuration
            .ethereum_beacon_endpoint()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Ethereum observer is enabled but ethereum_beacon_endpoint is not configured"
                )
            })?;

        let network_str = self
            .configuration
            .ethereum_network()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Ethereum observer is enabled but ethereum_network is not configured"
                )
            })?;

        // Validate network string (though EthereumChainObserver accepts any string)
        let normalized_network = network_str.to_lowercase();
        match normalized_network.as_str() {
            "mainnet" | "holesky" | "sepolia" => {},
            other => {
                return Err(anyhow::anyhow!(
                    "Unknown Ethereum network: {}. Expected 'mainnet', 'holesky', or 'sepolia'",
                    other
                )
                .into())
            }
        };

        let beacon_client = mithril_ethereum_chain::BeaconClient::new(&endpoint);
        let observer = EthereumChainObserver::new(beacon_client, &normalized_network);

        Ok(Some(Arc::new(observer)))
    }
}

