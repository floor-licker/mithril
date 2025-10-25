use anyhow::Context;
use std::sync::Arc;
use tokio::sync::Mutex;

use mithril_cardano_node_chain::{
    chain_observer::{CardanoCliRunner, ChainObserver, ChainObserverBuilder, ChainObserverType},
    chain_reader::{ChainBlockReader, PallasChainReader},
    chain_scanner::{BlockScanner, CardanoBlockScanner},
    test::double::FakeChainObserver,
};
use mithril_cardano_node_internal_database::digesters::{
    CardanoImmutableDigester, ImmutableDigester,
};
use mithril_common::entities::SignedEntityTypeDiscriminants;
use mithril_signed_entity_preloader::{
    CardanoTransactionsPreloader, CardanoTransactionsPreloaderActivation,
};

use crate::ExecutionEnvironment;
use crate::dependency_injection::{DependenciesBuilder, Result};
use crate::get_dependency;
use crate::services::{MithrilStakeDistributionService, StakeDistributionService};
impl DependenciesBuilder {
    /// Build the Cardano chain observer conditionally based on signed_entity_types configuration.
    ///
    /// Originally, the aggregator ALWAYS created a Cardano chain observer on startup, which
    /// required connecting to a Cardano node. This made it impossible to deploy an Ethereum-only
    /// aggregator - it would fail to start without a Cardano node even though it didn't need one.
    ///
    /// Now we check the configured `signed_entity_types` (what the aggregator is supposed to
    /// certify) and only create the Cardano observer if there are actual Cardano types to certify.
    /// If you're only certifying Ethereum stuff, no Cardano observer is created.
    ///
    /// Returns `None` if no Cardano-specific signed entity types are configured
    /// (e.g., Ethereum-only aggregator).
    ///
    /// This maintains backward compatibility:
    /// - Existing Cardano configs work unchanged (observer always created)
    /// - Ethereum-only configs skip Cardano observer (no Cardano node needed)
    /// - Multi-chain configs create both observers
    async fn build_chain_observer(&mut self) -> Result<Option<Arc<dyn ChainObserver>>> {
        // First, ask the config: "Do we actually need Cardano?"
        // This looks at signed_entity_types and returns true if ANY Cardano type is present.
        let requires_cardano = self
            .configuration
            .requires_cardano_observer()
            .with_context(|| "Dependencies Builder failed to determine if Cardano observer is required")?;

        if !requires_cardano {
            // We're Ethereum-only (or something else). Skip Cardano entirely.
            // This means no Cardano node connection required, which is what we want.
            return Ok(None);
        }

        // We need Cardano. Build the observer using the original logic below.
        let chain_observer: Arc<dyn ChainObserver> = match self.configuration.environment() {
            ExecutionEnvironment::Production => {
                let chain_observer_type = &self.configuration.chain_observer_type();
                let cardano_cli_runner = match chain_observer_type {
                    ChainObserverType::CardanoCli => Some(self.get_cardano_cli_runner().await?),
                    _ => None,
                };
                let cardano_node_socket_path = &self.configuration.cardano_node_socket_path();
                let cardano_network = &self
                    .configuration
                    .get_network()
                    .with_context(|| "Dependencies Builder can not get Cardano network while building the chain observer")?;
                let chain_observer_builder = ChainObserverBuilder::new(
                    chain_observer_type,
                    cardano_node_socket_path,
                    cardano_network,
                    cardano_cli_runner.as_deref(),
                );

                chain_observer_builder
                    .build()
                    .with_context(|| "Dependencies Builder can not build chain observer")?
            }
            _ => Arc::new(FakeChainObserver::default()),
        };

        Ok(Some(chain_observer))
    }

    /// Return a [ChainObserver] if Cardano types are configured.
    ///
    /// Returns `None` for Ethereum-only aggregators.
    pub async fn get_chain_observer(&mut self) -> Result<Option<Arc<dyn ChainObserver>>> {
        get_dependency!(self.chain_observer)
    }

    async fn build_cardano_cli_runner(&mut self) -> Result<Box<CardanoCliRunner>> {
        let cli_runner = CardanoCliRunner::new(
            self.configuration.cardano_cli_path(),
            self.configuration.cardano_node_socket_path(),
            self.configuration.get_network().with_context(|| {
                "Dependencies Builder can not get Cardano network while building cardano cli runner"
            })?,
        );

        Ok(Box::new(cli_runner))
    }

    /// Return a [CardanoCliRunner]
    pub async fn get_cardano_cli_runner(&mut self) -> Result<Box<CardanoCliRunner>> {
        get_dependency!(self.cardano_cli_runner)
    }

    async fn build_chain_block_reader(&mut self) -> Result<Arc<Mutex<dyn ChainBlockReader>>> {
        let chain_block_reader = PallasChainReader::new(
            &self.configuration.cardano_node_socket_path(),
            self.configuration.get_network()?,
            self.root_logger(),
        );

        Ok(Arc::new(Mutex::new(chain_block_reader)))
    }

    /// Chain reader
    pub async fn get_chain_block_reader(&mut self) -> Result<Arc<Mutex<dyn ChainBlockReader>>> {
        get_dependency!(self.chain_block_reader)
    }

    async fn build_block_scanner(&mut self) -> Result<Arc<dyn BlockScanner>> {
        let block_scanner = CardanoBlockScanner::new(
            self.get_chain_block_reader().await?,
            self.configuration
                .cardano_transactions_block_streamer_max_roll_forwards_per_poll(),
            self.root_logger(),
        );

        Ok(Arc::new(block_scanner))
    }

    /// Block scanner
    pub async fn get_block_scanner(&mut self) -> Result<Arc<dyn BlockScanner>> {
        get_dependency!(self.block_scanner)
    }

    async fn build_immutable_digester(&mut self) -> Result<Arc<dyn ImmutableDigester>> {
        let immutable_digester_cache = match self.configuration.environment() {
            ExecutionEnvironment::Production => Some(self.get_immutable_cache_provider().await?),
            _ => None,
        };
        let digester = CardanoImmutableDigester::new(
            self.configuration.get_network()?.to_string(),
            immutable_digester_cache,
            self.root_logger(),
        );

        Ok(Arc::new(digester))
    }

    /// Immutable digester.
    pub async fn get_immutable_digester(&mut self) -> Result<Arc<dyn ImmutableDigester>> {
        get_dependency!(self.immutable_digester)
    }

    /// Create a [CardanoTransactionsPreloader] instance.
    pub async fn create_cardano_transactions_preloader(
        &mut self,
    ) -> Result<Arc<CardanoTransactionsPreloader>> {
        let activation = self
            .configuration
            .compute_allowed_signed_entity_types_discriminants()?
            .contains(&SignedEntityTypeDiscriminants::CardanoTransactions);
        
        // CardanoTransactionsPreloader requires Cardano chain observer
        let chain_observer = self.get_chain_observer().await?
            .ok_or_else(|| anyhow::anyhow!("Cardano chain observer is required for CardanoTransactionsPreloader but is not configured"))?;
        
        let cardano_transactions_preloader = CardanoTransactionsPreloader::new(
            self.get_signed_entity_type_lock().await?,
            self.get_transactions_importer().await?,
            self.configuration
                .cardano_transactions_signing_config()
                .security_parameter,
            chain_observer,
            self.root_logger(),
            Arc::new(CardanoTransactionsPreloaderActivation::new(activation)),
        );

        Ok(Arc::new(cardano_transactions_preloader))
    }

    async fn build_stake_distribution_service(
        &mut self,
    ) -> Result<Arc<dyn StakeDistributionService>> {
        use mithril_cardano_node_chain::test::double::FakeChainObserver;
        
        // For Ethereum-only aggregators, use a FakeChainObserver
        // (MithrilStakeDistribution is still needed for protocol parameters)
        let chain_observer = self.get_chain_observer().await?
            .unwrap_or_else(|| Arc::new(FakeChainObserver::default()));
        
        let stake_distribution_service = Arc::new(MithrilStakeDistributionService::new(
            self.get_stake_store().await?,
            chain_observer,
        ));

        Ok(stake_distribution_service)
    }

    /// [StakeDistributionService] service
    pub async fn get_stake_distribution_service(
        &mut self,
    ) -> Result<Arc<dyn StakeDistributionService>> {
        get_dependency!(self.stake_distribution_service)
    }
}

#[cfg(test)]
mod tests {
    use mithril_common::{entities::SignedEntityTypeDiscriminants, temp_dir};

    use crate::ServeCommandConfiguration;

    use super::*;

    #[tokio::test]
    async fn cardano_transactions_preloader_activated_with_cardano_transactions_signed_entity_type_in_configuration()
     {
        assert_cardano_transactions_preloader_activation(
            SignedEntityTypeDiscriminants::CardanoTransactions.to_string(),
            true,
        )
        .await;
        assert_cardano_transactions_preloader_activation(
            SignedEntityTypeDiscriminants::MithrilStakeDistribution.to_string(),
            false,
        )
        .await;
    }

    async fn assert_cardano_transactions_preloader_activation(
        signed_entity_types: String,
        expected_activation: bool,
    ) {
        let configuration = ServeCommandConfiguration {
            signed_entity_types: Some(signed_entity_types),
            ..ServeCommandConfiguration::new_sample(temp_dir!())
        };
        let mut dep_builder = DependenciesBuilder::new_with_stdout_logger(Arc::new(configuration));

        let cardano_transactions_preloader =
            dep_builder.create_cardano_transactions_preloader().await.unwrap();

        let is_activated = cardano_transactions_preloader.is_activated().await.unwrap();
        assert_eq!(
            expected_activation, is_activated,
            "'is_activated' expected {expected_activation}, but was {is_activated}"
        );
    }
}
