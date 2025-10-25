use async_trait::async_trait;
use std::sync::Arc;

use mithril_common::{
    StdResult,
    entities::Epoch,
    signable_builder::{EthereumStateRootData, EthereumStateRootRetriever},
};
use mithril_universal::UniversalChainObserver;

/// Implementation of [EthereumStateRootRetriever] that uses a [UniversalChainObserver]
pub struct UniversalEthereumStateRootRetriever {
    chain_observer: Arc<dyn UniversalChainObserver>,
}

impl UniversalEthereumStateRootRetriever {
    /// Create a new instance of [UniversalEthereumStateRootRetriever]
    pub fn new(chain_observer: Arc<dyn UniversalChainObserver>) -> Self {
        Self { chain_observer }
    }
}

#[async_trait]
impl EthereumStateRootRetriever for UniversalEthereumStateRootRetriever {
    async fn retrieve(&self, epoch: Epoch) -> StdResult<Option<EthereumStateRootData>> {
        // Get the current epoch to determine if we have data for the requested epoch
        let current_epoch_info = self.chain_observer.get_current_epoch().await
            .map_err(|e| anyhow::anyhow!("Failed to get current epoch: {}", e))?;
        
        // If the requested epoch is in the future, return None
        if epoch.0 > current_epoch_info.epoch_number {
            return Ok(None);
        }

        // Compute the state commitment for this epoch
        let state_commitment = self.chain_observer.compute_state_commitment(epoch.0).await
            .map_err(|e| anyhow::anyhow!("Failed to compute state commitment: {}", e))?;

        // Convert the value (Vec<u8>) to hex string
        let state_root_hex = format!("0x{}", hex::encode(&state_commitment.value));

        Ok(Some(EthereumStateRootData {
            state_root: state_root_hex,
            block_number: state_commitment.block_number,
            epoch: epoch.0,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mithril_universal::{ChainId, ChainObserverError, CommitmentType, EpochInfo, StakeDistribution, StateCommitment, ValidatorId};
    use std::collections::HashMap;

    struct MockChainObserver {
        current_epoch: u64,
        state_root: Vec<u8>,
        block_number: u64,
    }

    #[async_trait]
    impl UniversalChainObserver for MockChainObserver {
        fn chain_id(&self) -> ChainId {
            ChainId::new("test")
        }

        async fn get_current_epoch(&self) -> Result<EpochInfo, ChainObserverError> {
            Ok(EpochInfo {
                chain_id: ChainId::new("test"),
                epoch_number: self.current_epoch,
                start_time: 0,
                end_time: None,
            })
        }

        async fn get_stake_distribution(
            &self,
            _epoch: u64,
        ) -> Result<StakeDistribution, ChainObserverError> {
            unimplemented!()
        }

        async fn compute_state_commitment(&self, _epoch: u64) -> Result<StateCommitment, ChainObserverError> {
            Ok(StateCommitment {
                chain_id: ChainId::new("test"),
                epoch: _epoch,
                commitment_type: CommitmentType::StateRoot,
                value: self.state_root.clone(),
                block_number: self.block_number,
                metadata: HashMap::new(),
            })
        }

        async fn is_validator_active(&self, _validator_id: &ValidatorId, _epoch: u64) -> Result<bool, ChainObserverError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn retrieve_returns_none_for_future_epoch() {
        let mock_observer = Arc::new(MockChainObserver {
            current_epoch: 10,
            state_root: vec![0x12, 0x34],
            block_number: 100,
        });
        let retriever = UniversalEthereumStateRootRetriever::new(mock_observer);

        let result = retriever.retrieve(Epoch(11)).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn retrieve_returns_state_root_data_for_current_epoch() {
        let mock_observer = Arc::new(MockChainObserver {
            current_epoch: 10,
            state_root: vec![0xab, 0xcd, 0xef],
            block_number: 1000,
        });
        let retriever = UniversalEthereumStateRootRetriever::new(mock_observer);

        let result = retriever.retrieve(Epoch(10)).await.unwrap();
        assert!(result.is_some());

        let data = result.unwrap();
        assert_eq!(data.state_root, "0xabcdef");
        assert_eq!(data.block_number, 1000);
        assert_eq!(data.epoch, 10);
    }

    #[tokio::test]
    async fn retrieve_returns_state_root_data_for_past_epoch() {
        let mock_observer = Arc::new(MockChainObserver {
            current_epoch: 10,
            state_root: vec![0x99, 0x88],
            block_number: 500,
        });
        let retriever = UniversalEthereumStateRootRetriever::new(mock_observer);

        let result = retriever.retrieve(Epoch(5)).await.unwrap();
        assert!(result.is_some());

        let data = result.unwrap();
        assert_eq!(data.state_root, "0x9988");
        assert_eq!(data.block_number, 500);
        assert_eq!(data.epoch, 5);
    }
}
