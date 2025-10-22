//! Ethereum chain observer implementation

use async_trait::async_trait;
use std::collections::HashMap;

use mithril_universal::{
    ChainId, ChainObserverError, CommitmentType, EpochInfo, StateCommitment, StakeDistribution,
    UniversalChainObserver, ValidatorId,
};

use crate::{BeaconClient, EthereumChainError};

/// Ethereum chain observer implementing the universal trait
///
/// This observer uses the Beacon Chain API to query validator information
/// and execution layer state roots for Mithril certification.
///
/// ## Ethereum Epochs vs Mithril Certification
///
/// Ethereum epochs are very short (6.4 minutes, 32 slots). For Mithril purposes,
/// we may want to certify less frequently. This can be configured via the
/// certification interval.
pub struct EthereumChainObserver {
    beacon_client: BeaconClient,
    chain_id: ChainId,
    certification_interval_epochs: u64,
}

impl EthereumChainObserver {
    /// Create a new Ethereum chain observer
    ///
    /// # Arguments
    ///
    /// * `beacon_client` - Client for querying the beacon chain
    /// * `network` - Network name (e.g., "mainnet", "holesky")
    ///
    /// # Example
    ///
    /// ```
    /// use mithril_ethereum_chain::{BeaconClient, EthereumChainObserver};
    ///
    /// let beacon_client = BeaconClient::new("http://localhost:5052");
    /// let observer = EthereumChainObserver::new(beacon_client, "mainnet");
    /// ```
    pub fn new(beacon_client: BeaconClient, network: &str) -> Self {
        Self {
            beacon_client,
            chain_id: ChainId::new(format!("ethereum-{}", network)),
            // Default: certify every 675 epochs (approximately 3 days)
            certification_interval_epochs: 675,
        }
    }

    /// Set the certification interval
    ///
    /// By default, we certify every 675 Ethereum epochs (about 3 days).
    /// This can be adjusted based on requirements.
    ///
    /// # Arguments
    ///
    /// * `interval` - Number of Ethereum epochs between certifications
    pub fn with_certification_interval(mut self, interval: u64) -> Self {
        self.certification_interval_epochs = interval;
        self
    }

    /// Calculate the epoch to certify based on current epoch
    ///
    /// This ensures we only certify at interval boundaries and account for
    /// finality delays.
    #[allow(dead_code)]
    fn calculate_certification_epoch(&self, current_epoch: u64) -> u64 {
        // Wait 2 epochs for finality
        let finalized_epoch = current_epoch.saturating_sub(2);
        
        // Round down to nearest certification interval
        (finalized_epoch / self.certification_interval_epochs)
            * self.certification_interval_epochs
    }
}

#[async_trait]
impl UniversalChainObserver for EthereumChainObserver {
    fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }

    async fn get_current_epoch(&self) -> Result<EpochInfo, ChainObserverError> {
        let current_slot = self
            .beacon_client
            .get_current_slot()
            .await
            .map_err(|e| ChainObserverError::EpochQueryError(e.to_string()))?;

        // Ethereum has 32 slots per epoch, 12 seconds per slot
        let epoch_number = current_slot / 32;
        let genesis_time = self
            .beacon_client
            .get_genesis_time()
            .await
            .map_err(|e| ChainObserverError::EpochQueryError(e.to_string()))?;

        // Calculate epoch start time
        // Each epoch is 32 slots * 12 seconds = 384 seconds = 6.4 minutes
        let epoch_duration_seconds = 32 * 12;
        let epoch_start_time = genesis_time + (epoch_number as i64 * epoch_duration_seconds);

        Ok(EpochInfo {
            chain_id: self.chain_id.clone(),
            epoch_number,
            start_time: epoch_start_time,
            end_time: None, // Ethereum epochs are ongoing
        })
    }

    async fn get_stake_distribution(
        &self,
        epoch: u64,
    ) -> Result<StakeDistribution, ChainObserverError> {
        let validators = self
            .beacon_client
            .get_validators_by_epoch(epoch)
            .await
            .map_err(|e| ChainObserverError::StakeDistributionError(e.to_string()))?;

        let mut distribution = StakeDistribution::new(epoch);

        for validator in validators {
            // Only include active validators
            if !validator.status.is_active() {
                continue;
            }

            // Parse effective balance (in Gwei)
            let stake = validator
                .validator
                .effective_balance_gwei()
                .map_err(|e| {
                    ChainObserverError::StakeDistributionError(format!(
                        "Invalid stake amount: {}",
                        e
                    ))
                })?;

            // Use validator public key as ID
            let validator_id = ValidatorId::new(validator.validator.pubkey);
            distribution.add_validator(validator_id, stake);
        }

        Ok(distribution)
    }

    async fn compute_state_commitment(
        &self,
        epoch: u64,
    ) -> Result<StateCommitment, ChainObserverError> {
        // For Ethereum, we certify the execution layer state root at the last slot of the epoch
        let last_slot_of_epoch = (epoch + 1) * 32 - 1;

        let beacon_block = self
            .beacon_client
            .get_block_by_slot(last_slot_of_epoch)
            .await
            .map_err(|e| ChainObserverError::StateCommitmentError(e.to_string()))?;

        // Extract execution payload
        let execution_payload = beacon_block
            .message
            .body
            .execution_payload
            .ok_or_else(|| {
                let err = EthereumChainError::NoExecutionPayload;
                ChainObserverError::StateCommitmentError(err.to_string())
            })?;

        // Get state root as bytes
        let state_root_bytes = execution_payload
            .state_root_bytes()
            .map_err(|e| ChainObserverError::StateCommitmentError(e.to_string()))?;

        let block_number = execution_payload
            .block_number_u64()
            .map_err(|e| ChainObserverError::StateCommitmentError(e.to_string()))?;

        // Build commitment with metadata
        let mut metadata = HashMap::new();
        metadata.insert("slot".to_string(), last_slot_of_epoch.to_string());
        metadata.insert("block_hash".to_string(), execution_payload.block_hash);
        metadata.insert("beacon_root".to_string(), beacon_block.message.state_root);
        metadata.insert("parent_hash".to_string(), execution_payload.parent_hash);

        Ok(StateCommitment {
            chain_id: self.chain_id.clone(),
            epoch,
            commitment_type: CommitmentType::StateRoot,
            value: state_root_bytes,
            block_number,
            metadata,
        })
    }

    async fn is_validator_active(
        &self,
        validator_id: &ValidatorId,
        epoch: u64,
    ) -> Result<bool, ChainObserverError> {
        let validator = self
            .beacon_client
            .get_validator_by_pubkey(validator_id.as_str(), epoch)
            .await
            .map_err(|e| {
                // If validator not found, they're not active
                if matches!(e, crate::BeaconApiError::NotFound(_)) {
                    return ChainObserverError::ValidatorNotFound(validator_id.as_str().to_string());
                }
                ChainObserverError::EpochQueryError(e.to_string())
            })?;

        Ok(validator.status.is_active())
    }

    fn get_metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("chain_type".to_string(), "ethereum".to_string());
        metadata.insert("chain_id".to_string(), self.chain_id.to_string());
        metadata.insert(
            "certification_interval_epochs".to_string(),
            self.certification_interval_epochs.to_string(),
        );
        metadata.insert("slots_per_epoch".to_string(), "32".to_string());
        metadata.insert("slot_duration_seconds".to_string(), "12".to_string());
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethereum_observer_creation() {
        let beacon_client = BeaconClient::new("http://localhost:5052");
        let observer = EthereumChainObserver::new(beacon_client, "mainnet");
        
        assert_eq!(observer.chain_id().as_str(), "ethereum-mainnet");
        assert_eq!(observer.certification_interval_epochs, 675);
    }

    #[test]
    fn test_certification_interval_configuration() {
        let beacon_client = BeaconClient::new("http://localhost:5052");
        let observer = EthereumChainObserver::new(beacon_client, "holesky")
            .with_certification_interval(100);
        
        assert_eq!(observer.certification_interval_epochs, 100);
    }

    #[test]
    fn test_calculate_certification_epoch() {
        let beacon_client = BeaconClient::new("http://localhost:5052");
        let observer = EthereumChainObserver::new(beacon_client, "mainnet")
            .with_certification_interval(100);

        // Current epoch 702, minus 2 for finality = 700, rounds to 700
        assert_eq!(observer.calculate_certification_epoch(702), 700);
        
        // Current epoch 750, minus 2 = 748, rounds down to 700
        assert_eq!(observer.calculate_certification_epoch(750), 700);
        
        // Current epoch 802, minus 2 = 800, rounds to 800
        assert_eq!(observer.calculate_certification_epoch(802), 800);
    }

    // Integration tests requiring a real beacon node should be in tests/ directory
}

