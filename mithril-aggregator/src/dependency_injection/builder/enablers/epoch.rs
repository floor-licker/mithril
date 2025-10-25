use std::sync::Arc;
use tokio::sync::RwLock;

use mithril_cardano_node_chain::test::double::FakeChainObserver;

use crate::dependency_injection::{DependenciesBuilder, EpochServiceWrapper, Result};
use crate::get_dependency;
use crate::services::{EpochServiceDependencies, MithrilEpochService};
impl DependenciesBuilder {
    async fn build_epoch_service(&mut self) -> Result<EpochServiceWrapper> {
        let verification_key_store = self.get_verification_key_store().await?;
        let epoch_settings_storer = self.get_epoch_settings_store().await?;
        
        // For Ethereum-only aggregators, use a FakeChainObserver (maintains backward compatibility)
        // For Cardano or multi-chain, use the real Cardano observer
        let chain_observer = self.get_chain_observer().await?
            .unwrap_or_else(|| Arc::new(FakeChainObserver::default()));
        
        let era_checker = self.get_era_checker().await?;
        let stake_store = self.get_stake_store().await?;
        let epoch_settings = self.configuration.get_epoch_settings_configuration();
        let allowed_discriminants = self
            .configuration
            .compute_allowed_signed_entity_types_discriminants()?;

        let epoch_service = Arc::new(RwLock::new(MithrilEpochService::new(
            epoch_settings,
            EpochServiceDependencies::new(
                epoch_settings_storer,
                verification_key_store,
                chain_observer,
                era_checker,
                stake_store,
            ),
            allowed_discriminants,
            self.root_logger(),
        )));

        Ok(epoch_service)
    }

    /// [EpochService][crate::services::EpochService] service
    pub async fn get_epoch_service(&mut self) -> Result<EpochServiceWrapper> {
        get_dependency!(self.epoch_service)
    }
}
