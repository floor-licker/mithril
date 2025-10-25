//! Test utilities and mock data for Ethereum chain integration

use crate::errors::BeaconApiError;
use crate::types::{BeaconBlock, BeaconBlockBody, BeaconBlockMessage, ExecutionPayload, ValidatorInfo, ValidatorDetails, ValidatorStatus};

/// Mock beacon client for testing
pub struct MockBeaconClient {
    /// Current slot number
    pub current_slot: u64,
    /// Genesis timestamp
    pub genesis_time: i64,
    /// Mock validators
    pub validators: Vec<ValidatorInfo>,
}

impl MockBeaconClient {
    /// Create a new mock beacon client with default test data
    pub fn new() -> Self {
        Self {
            current_slot: 5437088,
            genesis_time: 1695902400, // Holesky genesis
            validators: Self::create_mock_validators(100), // 100 validators for testing
        }
    }

    /// Create mock validators with realistic data
    fn create_mock_validators(count: usize) -> Vec<ValidatorInfo> {
        (0..count)
            .map(|i| {
                let effective_balance = if i < 80 {
                    "32000000000" // 32 ETH (active)
                } else {
                    "16000000000" // 16 ETH (some with lower balance)
                };

                ValidatorInfo {
                    index: i.to_string(),
                    balance: effective_balance.to_string(),
                    status: if i < 90 {
                        ValidatorStatus::ActiveOngoing
                    } else {
                        ValidatorStatus::PendingQueued
                    },
                    validator: ValidatorDetails {
                        pubkey: format!("0x{:0>96}", format!("{:x}", i)), // Mock pubkey
                        withdrawal_credentials: format!("0x00{:0>62}", ""),
                        effective_balance: effective_balance.to_string(),
                        slashed: false,
                        activation_eligibility_epoch: "0".to_string(),
                        activation_epoch: if i < 90 { "0" } else { "999999" }.to_string(),
                        exit_epoch: "18446744073709551615".to_string(), // Max u64
                        withdrawable_epoch: "18446744073709551615".to_string(),
                    },
                }
            })
            .collect()
    }

    /// Create a mock beacon block
    pub fn create_mock_block(&self, slot: u64) -> BeaconBlock {
        BeaconBlock {
            message: BeaconBlockMessage {
                slot: slot.to_string(),
                proposer_index: "12345".to_string(),
                parent_root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                state_root: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
                body: BeaconBlockBody {
                    execution_payload: Some(ExecutionPayload {
                        block_number: (slot / 32 * 2).to_string(), // Approximate block number
                        block_hash: format!("0x{:0>64}", format!("{:x}", slot)),
                        state_root: format!("0x{:0>64}", format!("{:x}", slot * 2)),
                        parent_hash: format!("0x{:0>64}", format!("{:x}", slot - 1)),
                        fee_recipient: "0x0000000000000000000000000000000000000000".to_string(),
                        gas_limit: "30000000".to_string(),
                        gas_used: "15000000".to_string(),
                        timestamp: (self.genesis_time + (slot * 12) as i64).to_string(),
                    }),
                },
            },
        }
    }

    /// Get validators by epoch (simulated)
    pub async fn get_validators_by_epoch_mock(&self, _epoch: u64) -> Result<Vec<ValidatorInfo>, BeaconApiError> {
        Ok(self.validators.clone())
    }

    /// Get block by slot (simulated)
    pub async fn get_block_by_slot_mock(&self, slot: u64) -> Result<BeaconBlock, BeaconApiError> {
        if slot > self.current_slot {
            return Err(BeaconApiError::NotFound(format!("Block at slot {} not found", slot)));
        }
        Ok(self.create_mock_block(slot))
    }

    /// Get current slot
    pub async fn get_current_slot_mock(&self) -> Result<u64, BeaconApiError> {
        Ok(self.current_slot)
    }

    /// Get genesis time
    pub async fn get_genesis_time_mock(&self) -> Result<i64, BeaconApiError> {
        Ok(self.genesis_time)
    }
}

impl Default for MockBeaconClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_beacon_client_creation() {
        let client = MockBeaconClient::new();
        assert_eq!(client.validators.len(), 100);
        assert_eq!(client.current_slot, 5437088);
    }

    #[test]
    fn test_mock_validators() {
        let validators = MockBeaconClient::create_mock_validators(10);
        assert_eq!(validators.len(), 10);
        
        // Check first validator is active
        assert_eq!(validators[0].status, ValidatorStatus::ActiveOngoing);
        assert_eq!(validators[0].validator.effective_balance, "32000000000");
        
        // Check last validator is pending (index 9 should still be active in 10 total)
        assert_eq!(validators[9].status, ValidatorStatus::ActiveOngoing);
    }

    #[tokio::test]
    async fn test_mock_block_creation() {
        let client = MockBeaconClient::new();
        let block = client.get_block_by_slot_mock(100).await.unwrap();
        
        assert_eq!(block.message.slot, "100");
        assert!(block.message.body.execution_payload.is_some());
        
        let payload = block.message.body.execution_payload.unwrap();
        assert_eq!(payload.block_number, "6"); // (100 / 32 * 2)
    }

    #[tokio::test]
    async fn test_mock_future_slot_returns_error() {
        let client = MockBeaconClient::new();
        let result = client.get_block_by_slot_mock(99999999).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BeaconApiError::NotFound(_)));
    }
}

