//! Integration tests for mithril-universal

use async_trait::async_trait;
use mithril_universal::{
    ChainId, ChainObserverError, CommitmentType, EpochInfo, StateCommitment, StakeDistribution,
    UniversalChainObserver, ValidatorId,
};

/// Mock chain observer for testing
struct MockChainObserver {
    chain_id: ChainId,
    current_epoch: u64,
}

impl MockChainObserver {
    fn new(chain_id: &str, epoch: u64) -> Self {
        Self {
            chain_id: ChainId::new(chain_id),
            current_epoch: epoch,
        }
    }
}

#[async_trait]
impl UniversalChainObserver for MockChainObserver {
    fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }

    async fn get_current_epoch(&self) -> Result<EpochInfo, ChainObserverError> {
        Ok(EpochInfo {
            chain_id: self.chain_id.clone(),
            epoch_number: self.current_epoch,
            start_time: 1234567890,
            end_time: Some(1234567890 + 86400),
        })
    }

    async fn get_stake_distribution(
        &self,
        epoch: u64,
    ) -> Result<StakeDistribution, ChainObserverError> {
        let mut distribution = StakeDistribution::new(epoch);
        distribution.add_validator(ValidatorId::new("validator1"), 1000);
        distribution.add_validator(ValidatorId::new("validator2"), 2000);
        distribution.add_validator(ValidatorId::new("validator3"), 1500);
        Ok(distribution)
    }

    async fn compute_state_commitment(
        &self,
        epoch: u64,
    ) -> Result<StateCommitment, ChainObserverError> {
        let commitment = StateCommitment::new(
            self.chain_id.clone(),
            epoch,
            CommitmentType::StateRoot,
            vec![1, 2, 3, 4, 5, 6, 7, 8],
            12345,
        )
        .with_metadata("test_key".to_string(), "test_value".to_string());

        Ok(commitment)
    }

    async fn is_validator_active(
        &self,
        validator_id: &ValidatorId,
        epoch: u64,
    ) -> Result<bool, ChainObserverError> {
        let distribution = self.get_stake_distribution(epoch).await?;
        Ok(distribution.get_stake(validator_id).is_some())
    }
}

#[tokio::test]
async fn test_mock_observer_epoch() {
    let observer = MockChainObserver::new("test-chain", 100);

    let epoch = observer.get_current_epoch().await.unwrap();
    assert_eq!(epoch.epoch_number, 100);
    assert_eq!(epoch.chain_id.as_str(), "test-chain");
}

#[tokio::test]
async fn test_mock_observer_stake_distribution() {
    let observer = MockChainObserver::new("test-chain", 100);

    let distribution = observer.get_stake_distribution(100).await.unwrap();
    assert_eq!(distribution.validator_count(), 3);
    assert_eq!(distribution.total_stake, 4500);
    assert_eq!(
        distribution.get_stake(&ValidatorId::new("validator1")),
        Some(1000)
    );
}

#[tokio::test]
async fn test_mock_observer_state_commitment() {
    let observer = MockChainObserver::new("test-chain", 100);

    let commitment = observer.compute_state_commitment(100).await.unwrap();
    assert_eq!(commitment.epoch, 100);
    assert_eq!(commitment.commitment_type, CommitmentType::StateRoot);
    assert_eq!(commitment.block_number, 12345);
    assert_eq!(commitment.value, vec![1, 2, 3, 4, 5, 6, 7, 8]);
}

#[tokio::test]
async fn test_mock_observer_validator_active() {
    let observer = MockChainObserver::new("test-chain", 100);

    let active = observer
        .is_validator_active(&ValidatorId::new("validator1"), 100)
        .await
        .unwrap();
    assert!(active);

    let inactive = observer
        .is_validator_active(&ValidatorId::new("nonexistent"), 100)
        .await
        .unwrap();
    assert!(!inactive);
}

#[tokio::test]
async fn test_multiple_chains() {
    let eth_observer = MockChainObserver::new("ethereum-mainnet", 200);
    let cardano_observer = MockChainObserver::new("cardano-mainnet", 400);

    let eth_epoch = eth_observer.get_current_epoch().await.unwrap();
    let cardano_epoch = cardano_observer.get_current_epoch().await.unwrap();

    assert_eq!(eth_epoch.chain_id.as_str(), "ethereum-mainnet");
    assert_eq!(cardano_epoch.chain_id.as_str(), "cardano-mainnet");
    assert_eq!(eth_epoch.epoch_number, 200);
    assert_eq!(cardano_epoch.epoch_number, 400);
}

