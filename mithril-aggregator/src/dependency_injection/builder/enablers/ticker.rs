use std::sync::Arc;

use anyhow::Context;
use mithril_cardano_node_chain::test::double::FakeChainObserver;
use mithril_cardano_node_internal_database::test::double::DumbImmutableFileObserver;
use mithril_cardano_node_internal_database::{ImmutableFileObserver, ImmutableFileSystemObserver};
use mithril_ticker::{MithrilTickerService, TickerService};

use crate::ExecutionEnvironment;
use crate::dependency_injection::{DependenciesBuilder, Result};
use crate::get_dependency;

impl DependenciesBuilder {
    /// Create [TickerService] instance.
    pub async fn build_ticker_service(&mut self) -> Result<Arc<dyn TickerService>> {
        // For Ethereum-only aggregators, use a FakeChainObserver
        let chain_observer = self.get_chain_observer().await?
            .unwrap_or_else(|| Arc::new(FakeChainObserver::default()));
        let immutable_observer = self.get_immutable_file_observer().await?;

        Ok(Arc::new(MithrilTickerService::new(
            chain_observer,
            immutable_observer,
        )))
    }

    /// [TickerService] service
    pub async fn get_ticker_service(&mut self) -> Result<Arc<dyn TickerService>> {
        get_dependency!(self.ticker_service)
    }

    async fn build_immutable_file_observer(&mut self) -> Result<Arc<dyn ImmutableFileObserver>> {
        // Check if Cardano observer is needed (immutable file observer is Cardano-specific)
        let requires_cardano = self
            .configuration
            .requires_cardano_observer()
            .with_context(|| "Dependencies Builder failed to determine if Cardano observer is required")?;
        
        let immutable_file_observer: Arc<dyn ImmutableFileObserver> =
            match self.configuration.environment() {
                ExecutionEnvironment::Production if requires_cardano => Arc::new(ImmutableFileSystemObserver::new(
                    &self.configuration.db_directory(),
                )),
                _ => Arc::new(DumbImmutableFileObserver::default()),
            };

        Ok(immutable_file_observer)
    }

    /// Return a [ImmutableFileObserver] instance.
    pub async fn get_immutable_file_observer(&mut self) -> Result<Arc<dyn ImmutableFileObserver>> {
        get_dependency!(self.immutable_file_observer)
    }
}
