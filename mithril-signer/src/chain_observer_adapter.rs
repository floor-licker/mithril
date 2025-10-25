//! Adapter to make UniversalChainObserver work with mithril-signer's infrastructure
//!
//! The signer was originally built with a Cardano-specific `ChainObserver` trait that has
//! methods like `get_current_datums()` (Cardano transaction metadata) and `get_current_era()`
//! (Shelley/Babbage/etc). These concepts don't exist on Ethereum or other chains.
//!
//! Rather than refactoring the entire signer codebase to use the new `UniversalChainObserver`
//! trait (which would be a massive change touching many files), I use the adapter pattern.
//! This adapter makes a `UniversalChainObserver` (like our Ethereum one) look like the old
//! Cardano-specific trait that the signer expects.
//!
//! - Cardano-specific methods (datums, era) return empty/placeholder values
//! - Universal methods (epoch, stake distribution) are translated between traits
//! - The signer can work with any chain without knowing the difference

use std::sync::Arc;

use async_trait::async_trait;
use mithril_cardano_node_chain::{
    chain_observer::{ChainObserver, ChainObserverError},
    entities::{ChainAddress, TxDatum},
};
use mithril_common::{
    entities::{Epoch, ChainPoint, StakeDistribution as CardanoStakeDistribution},
    crypto_helper::KesPeriod,
};
use mithril_universal::UniversalChainObserver;

/// Adapter that wraps a UniversalChainObserver to implement Cardano's ChainObserver trait
///
/// This allows the signer to work with any blockchain (Ethereum, Cardano, etc.) without
/// changing its core logic. The adapter translates between the universal interface and
/// the Cardano-specific interface the signer was originally built for.
pub struct UniversalChainObserverAdapter {
    inner: Arc<dyn UniversalChainObserver>,
}

impl UniversalChainObserverAdapter {
    /// Create a new adapter
    pub fn new(observer: Arc<dyn UniversalChainObserver>) -> Self {
        Self { inner: observer }
    }
}

#[async_trait]
impl ChainObserver for UniversalChainObserverAdapter {
    async fn get_current_datums(
        &self,
        _address: &ChainAddress,
    ) -> Result<Vec<TxDatum>, ChainObserverError> {
        // Datums are Cardano-specific (transaction metadata)
        // For non-Cardano chains, return empty list
        Ok(Vec::new())
    }

    async fn get_current_era(&self) -> Result<Option<String>, ChainObserverError> {
        // Era is Cardano-specific (Shelley, Babbage, etc.)
        // For non-Cardano chains, we can return a generic era name
        Ok(Some("universal".to_string()))
    }

    async fn get_current_epoch(&self) -> Result<Option<Epoch>, ChainObserverError> {
        let epoch_info = self.inner.get_current_epoch().await
            .map_err(|e| ChainObserverError::General(anyhow::Error::msg(e.to_string())))?;
        Ok(Some(Epoch(epoch_info.epoch_number)))
    }

    async fn get_current_chain_point(&self) -> Result<Option<ChainPoint>, ChainObserverError> {
        // ChainPoint is Cardano-specific (slot number, block hash)
        // For non-Cardano chains, we'll construct a minimal ChainPoint from epoch
        let epoch_info = self.inner.get_current_epoch().await
            .map_err(|e| ChainObserverError::General(anyhow::Error::msg(e.to_string())))?;
        // Use epoch number as slot number approximation
        use mithril_common::entities::{BlockNumber, SlotNumber};
        Ok(Some(ChainPoint {
            slot_number: SlotNumber(epoch_info.epoch_number * 432000), // Approximate slots per epoch
            block_number: BlockNumber(epoch_info.epoch_number * 21600),  // Approximate blocks per epoch
            block_hash: format!("epoch_{}", epoch_info.epoch_number),
        }))
    }

    async fn get_current_stake_distribution(&self) -> Result<Option<CardanoStakeDistribution>, ChainObserverError> {
        let epoch_info = self.inner.get_current_epoch().await
            .map_err(|e| ChainObserverError::General(anyhow::Error::msg(e.to_string())))?;
        let stake_dist = self.inner.get_stake_distribution(epoch_info.epoch_number).await
            .map_err(|e| ChainObserverError::General(anyhow::Error::msg(e.to_string())))?;
        
        // Convert universal stake distribution to Cardano format
        // validators is HashMap<ValidatorId, u64> where u64 is the stake
        Ok(Some(CardanoStakeDistribution::from_iter(
            stake_dist.validators.into_iter().map(|(validator_id, stake)| {
                (validator_id.to_string(), stake)
            })
        )))
    }

    async fn get_current_kes_period(&self) -> Result<Option<KesPeriod>, ChainObserverError> {
        // KES period is Cardano-specific
        // For non-Cardano chains, return None
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mithril_universal::{
        ChainId, ChainObserverError, EpochInfo, StakeDistribution, StateCommitment, ValidatorId,
    };
    use std::collections::HashMap;

    struct MockUniversalObserver;

    #[async_trait]
    impl UniversalChainObserver for MockUniversalObserver {
        fn chain_id(&self) -> ChainId {
            ChainId::new("test-chain")
        }

        async fn get_current_epoch(&self) -> Result<EpochInfo, ChainObserverError> {
            Ok(EpochInfo {
                chain_id: ChainId::new("test-chain"),
                epoch_number: 100,
                start_time: 1000000,
                end_time: Some(1000001),
            })
        }

        async fn get_stake_distribution(
            &self,
            _epoch: u64,
        ) -> Result<StakeDistribution, ChainObserverError> {
            let mut validators = HashMap::new();
            validators.insert(ValidatorId::new("validator1"), 1000);
            Ok(StakeDistribution {
                epoch: 100,
                validators,
                total_stake: 1000,
            })
        }

        async fn compute_state_commitment(
            &self,
            _epoch: u64,
        ) -> Result<StateCommitment, ChainObserverError> {
            use mithril_universal::CommitmentType;
            Ok(StateCommitment {
                chain_id: ChainId::new("test-chain"),
                epoch: 100,
                commitment_type: CommitmentType::StateRoot,
                value: vec![1, 2, 3, 4],
                block_number: 1000,
                metadata: HashMap::new(),
            })
        }

        async fn is_validator_active(
            &self,
            _validator_id: &ValidatorId,
            _epoch: u64,
        ) -> Result<bool, ChainObserverError> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_adapter_get_current_epoch() {
        let mock = Arc::new(MockUniversalObserver);
        let adapter = UniversalChainObserverAdapter::new(mock);

        let epoch = adapter.get_current_epoch().await.unwrap();
        assert_eq!(epoch, Some(Epoch(100)));
    }

    #[tokio::test]
    async fn test_adapter_get_stake_distribution() {
        let mock = Arc::new(MockUniversalObserver);
        let adapter = UniversalChainObserverAdapter::new(mock);

        let stake_dist = adapter.get_current_stake_distribution().await.unwrap();
        assert!(stake_dist.is_some());
        let stake_dist = stake_dist.unwrap();
        assert_eq!(stake_dist.len(), 1);
    }

    #[tokio::test]
    async fn test_adapter_get_datums() {
        let mock = Arc::new(MockUniversalObserver);
        let adapter = UniversalChainObserverAdapter::new(mock);
        
        let address: ChainAddress = "test_address".to_string();
        let datums = adapter.get_current_datums(&address).await.unwrap();
        assert_eq!(datums.len(), 0);
    }
}

