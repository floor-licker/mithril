//! Cardano adapter for the universal chain observer
//!
//! This adapter wraps Mithril's existing Cardano chain observer to implement
//! the universal interface, proving that the abstraction works without modifying
//! existing Cardano functionality.

use async_trait::async_trait;
use std::sync::Arc;

use mithril_cardano_node_chain::ChainObserver as CardanoChainObserver;

use crate::{
    ChainId, ChainObserverError, CommitmentType, EpochInfo, StateCommitment, StakeDistribution,
    UniversalChainObserver, ValidatorId,
};

/// Adapter that wraps Cardano's existing ChainObserver to implement UniversalChainObserver
///
/// This demonstrates that the universal interface works with existing implementations
/// without requiring changes to the Cardano-specific code.
///
/// # Example
///
/// ```rust,no_run
/// use mithril_universal::adapters::CardanoChainObserverAdapter;
/// use mithril_cardano_node_chain::ChainObserver;
/// use std::sync::Arc;
///
/// # async fn example(cardano_observer: Arc<dyn ChainObserver>) {
/// let adapter = CardanoChainObserverAdapter::new(
///     cardano_observer,
///     "mainnet"
/// );
///
/// let epoch = adapter.get_current_epoch().await.unwrap();
/// println!("Cardano epoch: {}", epoch.epoch_number);
/// # }
/// ```
pub struct CardanoChainObserverAdapter {
    cardano_observer: Arc<dyn CardanoChainObserver>,
    chain_id: ChainId,
}

impl CardanoChainObserverAdapter {
    /// Create a new Cardano adapter
    ///
    /// # Arguments
    ///
    /// * `cardano_observer` - The existing Cardano chain observer to wrap
    /// * `network` - The Cardano network name (e.g., "mainnet", "preprod", "preview")
    pub fn new(cardano_observer: Arc<dyn CardanoChainObserver>, network: &str) -> Self {
        Self {
            cardano_observer,
            chain_id: ChainId::new(format!("cardano-{}", network)),
        }
    }
}

#[async_trait]
impl UniversalChainObserver for CardanoChainObserverAdapter {
    fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }

    async fn get_current_epoch(&self) -> Result<EpochInfo, ChainObserverError> {
        let epoch = self
            .cardano_observer
            .get_current_epoch()
            .await
            .map_err(|e| ChainObserverError::EpochQueryError(e.to_string()))?
            .ok_or_else(|| ChainObserverError::EpochQueryError("No epoch found".to_string()))?;

        // Note: Cardano epochs are approximately 5 days
        // We could calculate more precise timing based on genesis time + epoch length
        Ok(EpochInfo {
            chain_id: self.chain_id.clone(),
            epoch_number: epoch,
            start_time: 0, // TODO: Calculate actual epoch start time
            end_time: None,
        })
    }

    async fn get_stake_distribution(
        &self,
        epoch: u64,
    ) -> Result<StakeDistribution, ChainObserverError> {
        // Get current stake distribution from Cardano observer
        // Note: In real implementation, we'd need to query historical stake for specific epoch
        let stake_dist = self
            .cardano_observer
            .get_current_stake_distribution()
            .await
            .map_err(|e| ChainObserverError::StakeDistributionError(e.to_string()))?
            .ok_or_else(|| {
                ChainObserverError::StakeDistributionError(
                    "No stake distribution available".to_string(),
                )
            })?;

        let mut distribution = StakeDistribution::new(epoch);
        for (party_id, stake) in stake_dist {
            distribution.add_validator(ValidatorId::new(party_id), stake);
        }

        Ok(distribution)
    }

    async fn compute_state_commitment(
        &self,
        epoch: u64,
    ) -> Result<StateCommitment, ChainObserverError> {
        // For Cardano, the state commitment would be based on the immutable file set
        // This is a simplified version - the actual implementation would:
        // 1. Get the chain point for the epoch
        // 2. Compute the digest of immutable files up to that point
        // 3. Return that as the commitment

        let chain_point = self
            .cardano_observer
            .get_current_chain_point()
            .await
            .map_err(|e| ChainObserverError::StateCommitmentError(e.to_string()))?
            .ok_or_else(|| {
                ChainObserverError::StateCommitmentError("No chain point available".to_string())
            })?;

        // TODO: Compute actual immutable file digest
        // For now, we use the chain point as a placeholder
        let commitment_value = format!("{:?}", chain_point).into_bytes();

        Ok(StateCommitment::new(
            self.chain_id.clone(),
            epoch,
            CommitmentType::ImmutableFileSet,
            commitment_value,
            chain_point.block_number,
        ))
    }

    async fn is_validator_active(
        &self,
        validator_id: &ValidatorId,
        epoch: u64,
    ) -> Result<bool, ChainObserverError> {
        // Check if validator exists in stake distribution for this epoch
        let stake_dist = self.get_stake_distribution(epoch).await?;
        Ok(stake_dist.get_stake(validator_id).is_some())
    }

    fn get_metadata(&self) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("adapter_type".to_string(), "cardano".to_string());
        metadata.insert("chain_id".to_string(), self.chain_id.to_string());
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Real tests would use the FakeChainObserver from mithril-cardano-node-chain
    // For now, these are just placeholders to show the structure

    #[tokio::test]
    #[ignore] // Requires actual Cardano observer instance
    async fn test_cardano_adapter_epoch() {
        // This would test with a mock or fake Cardano observer
    }

    #[tokio::test]
    #[ignore]
    async fn test_cardano_adapter_stake_distribution() {
        // This would test stake distribution query
    }
}

