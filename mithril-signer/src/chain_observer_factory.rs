//! Factory for creating chain-specific observers

use std::sync::Arc;

use anyhow::Context;
use mithril_cardano_node_chain::chain_observer::{
    CardanoCliRunner, ChainObserver, ChainObserverBuilder, ChainObserverType,
};
use mithril_common::StdResult;
use slog::Logger;

use crate::configuration::{ChainType, Configuration};

/// Build a chain observer based on configuration
pub fn build_chain_observer(
    config: &Configuration,
    logger: Logger,
) -> StdResult<Arc<dyn ChainObserver>> {
    match config.chain_type {
        ChainType::Cardano => build_cardano_observer(config, logger),
        ChainType::Ethereum => build_ethereum_observer(config, logger),
    }
}

/// Build a Cardano chain observer
fn build_cardano_observer(
    config: &Configuration,
    _logger: Logger,
) -> StdResult<Arc<dyn ChainObserver>> {
    let chain_observer_type = ChainObserverType::Pallas;
    let cardano_cli_path = &config.cardano_cli_path;
    let cardano_node_socket_path = &config.cardano_node_socket_path;
    let cardano_network = &config.get_network().with_context(|| {
        "Dependencies Builder can not get Cardano network while building the chain observer"
    })?;
    let cardano_cli_runner = &CardanoCliRunner::new(
        cardano_cli_path.to_owned(),
        cardano_node_socket_path.to_owned(),
        cardano_network.to_owned(),
    );

    let chain_observer_builder = ChainObserverBuilder::new(
        &chain_observer_type,
        cardano_node_socket_path,
        cardano_network,
        Some(cardano_cli_runner),
    );

    chain_observer_builder
        .build()
        .with_context(|| "Dependencies Builder can not build chain observer")
}

/// Build an Ethereum chain observer
fn build_ethereum_observer(
    _config: &Configuration,
    _logger: Logger,
) -> StdResult<Arc<dyn ChainObserver>> {
    // For now, return an error. This will be implemented when we integrate
    // mithril-ethereum-chain with the signer.
    anyhow::bail!("Ethereum observer not yet implemented in signer")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Configuration;
    use mithril_common::entities::PartyId;

    #[test]
    fn test_build_cardano_observer() {
        let config = Configuration::new_sample::<PartyId>("test".to_string());
        let logger = slog::Logger::root(slog::Discard, slog::o!());

        // This should create a Cardano observer (though it may fail due to missing dependencies)
        let result = build_chain_observer(&config, logger);
        // We expect this to fail in tests since we don't have a real Cardano node
        match result {
            Ok(_) => assert!(true, "Successfully created observer"),
            Err(e) => assert!(
                e.to_string().contains("ChainObserver")
                    || e.to_string().contains("socket")
                    || e.to_string().contains("connection"),
                "Expected connection error, got: {}",
                e
            ),
        }
    }

    #[test]
    fn test_build_ethereum_observer_not_implemented() {
        let mut config = Configuration::new_sample::<PartyId>("test".to_string());
        config.chain_type = ChainType::Ethereum;
        let logger = slog::Logger::root(slog::Discard, slog::o!());

        let result = build_chain_observer(&config, logger);
        match result {
            Ok(_) => panic!("Expected error for Ethereum observer"),
            Err(e) => assert!(
                e.to_string().contains("Ethereum observer not yet implemented"),
                "Expected 'not yet implemented' error, got: {}",
                e
            ),
        }
    }
}

