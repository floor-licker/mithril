//! Ethereum-specific types

use serde::{Deserialize, Serialize};

/// Ethereum network
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EthereumNetwork {
    /// Mainnet
    Mainnet,
    /// Holesky testnet
    Holesky,
    /// Sepolia testnet
    Sepolia,
}

impl EthereumNetwork {
    /// Get network name as string
    pub fn name(&self) -> &str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Holesky => "holesky",
            Self::Sepolia => "sepolia",
        }
    }
}

/// Validator status in the beacon chain
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidatorStatus {
    /// Pending initialized
    PendingInitialized,
    /// Pending queued
    PendingQueued,
    /// Active ongoing
    ActiveOngoing,
    /// Active exiting
    ActiveExiting,
    /// Active slashed
    ActiveSlashed,
    /// Exited unslashed
    ExitedUnslashed,
    /// Exited slashed
    ExitedSlashed,
    /// Withdrawal possible
    WithdrawalPossible,
    /// Withdrawal done
    WithdrawalDone,
}

impl ValidatorStatus {
    /// Check if validator is active (can participate in attestation)
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::ActiveOngoing | Self::ActiveExiting | Self::ActiveSlashed
        )
    }
}

/// Information about a validator
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidatorInfo {
    /// Validator index
    pub index: String,
    
    /// Current balance in Gwei
    pub balance: String,
    
    /// Status of the validator
    pub status: ValidatorStatus,
    
    /// Validator details
    pub validator: ValidatorDetails,
}

/// Validator details from beacon chain
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidatorDetails {
    /// BLS public key (48 bytes hex)
    pub pubkey: String,
    
    /// Withdrawal credentials
    pub withdrawal_credentials: String,
    
    /// Effective balance in Gwei (max 32 ETH)
    pub effective_balance: String,
    
    /// Whether validator has been slashed
    pub slashed: bool,
    
    /// Epoch when validator activated
    pub activation_eligibility_epoch: String,
    
    /// Epoch when validator becomes active
    pub activation_epoch: String,
    
    /// Epoch when validator exits
    pub exit_epoch: String,
    
    /// Epoch when validator can withdraw
    pub withdrawable_epoch: String,
}

impl ValidatorDetails {
    /// Parse effective balance from string to u64 (in Gwei)
    pub fn effective_balance_gwei(&self) -> Result<u64, std::num::ParseIntError> {
        self.effective_balance.parse()
    }
}

/// Beacon API response wrapper
#[derive(Debug, Deserialize, Serialize)]
pub struct BeaconApiResponse<T> {
    /// Response data
    pub data: T,
}

/// Beacon API v2 block response (includes version info)
///
///
/// The Ethereum Beacon API has TWO versions of the blocks endpoint:
/// - `/eth/v1/beacon/blocks/{slot}` - Returns just the block data
/// - `/eth/v2/beacon/blocks/{slot}` - Returns block data PLUS metadata (version, finalized, etc.)
///
/// We use v2 because:
/// 1. It's more future-proof (includes fork version info)
/// 2. The metadata fields help with debugging
/// 3. Some beacon nodes return better error messages on v2
///
/// The response structure is:
/// ```json
/// {
///   "version": "deneb",
///   "execution_optimistic": false,
///   "finalized": true,
///   "data": { ...actual block data... }
/// }
/// ```
///
/// We need this wrapper struct to deserialize the outer envelope, then extract the
/// `data` field which contains the actual `BeaconBlock`.
#[derive(Debug, Deserialize, Serialize)]
pub struct BeaconApiV2BlockResponse {
    /// Block version (e.g., "fulu", "deneb", "capella")
    /// This tells us which Ethereum fork the block is from
    #[serde(default)]
    pub version: Option<String>,
    
    /// Whether execution is optimistic
    /// (true if the execution layer hasn't verified the block yet)
    #[serde(default)]
    pub execution_optimistic: Option<bool>,
    
    /// Whether block is finalized
    /// (true if the block has been finalized by the consensus layer)
    #[serde(default)]
    pub finalized: Option<bool>,
    
    /// The actual beacon block data (what we really want)
    pub data: BeaconBlock,
}

/// Beacon block
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeaconBlock {
    /// Beacon block message
    pub message: BeaconBlockMessage,
}

/// Beacon block message
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeaconBlockMessage {
    /// Slot number
    pub slot: String,
    
    /// Proposer index
    pub proposer_index: String,
    
    /// Parent root
    pub parent_root: String,
    
    /// State root
    pub state_root: String,
    
    /// Block body
    pub body: BeaconBlockBody,
}

/// Beacon block body
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeaconBlockBody {
    /// Execution payload (post-merge)
    pub execution_payload: Option<ExecutionPayload>,
}

/// Execution layer payload
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionPayload {
    /// Block number
    pub block_number: String,
    
    /// Block hash
    pub block_hash: String,
    
    /// State root of execution layer
    pub state_root: String,
    
    /// Parent hash
    pub parent_hash: String,
    
    /// Fee recipient
    pub fee_recipient: String,
    
    /// Gas limit
    pub gas_limit: String,
    
    /// Gas used
    pub gas_used: String,
    
    /// Timestamp
    pub timestamp: String,
}

impl ExecutionPayload {
    /// Parse block number from string to u64
    pub fn block_number_u64(&self) -> Result<u64, std::num::ParseIntError> {
        self.block_number.parse()
    }
    
    /// Get state root as bytes
    pub fn state_root_bytes(&self) -> Result<Vec<u8>, hex::FromHexError> {
        // Remove 0x prefix if present
        let hex_str = self.state_root.strip_prefix("0x").unwrap_or(&self.state_root);
        hex::decode(hex_str)
    }
}

/// Genesis information
#[derive(Debug, Deserialize, Serialize)]
pub struct GenesisData {
    /// Genesis time (Unix timestamp)
    pub genesis_time: String,
    
    /// Genesis validators root
    pub genesis_validators_root: String,
    
    /// Genesis fork version
    pub genesis_fork_version: String,
}

impl GenesisData {
    /// Parse genesis time to i64
    pub fn genesis_time_i64(&self) -> Result<i64, std::num::ParseIntError> {
        self.genesis_time.parse()
    }
}

/// Current fork information
#[derive(Debug, Deserialize, Serialize)]
pub struct ForkData {
    /// Previous version
    pub previous_version: String,
    
    /// Current version
    pub current_version: String,
    
    /// Epoch of fork
    pub epoch: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_status_is_active() {
        assert!(ValidatorStatus::ActiveOngoing.is_active());
        assert!(ValidatorStatus::ActiveExiting.is_active());
        assert!(ValidatorStatus::ActiveSlashed.is_active());
        assert!(!ValidatorStatus::PendingInitialized.is_active());
        assert!(!ValidatorStatus::ExitedUnslashed.is_active());
    }

    #[test]
    fn test_execution_payload_parsing() {
        let payload = ExecutionPayload {
            block_number: "12345".to_string(),
            block_hash: "0xabc".to_string(),
            state_root: "0x1234567890abcdef".to_string(),
            parent_hash: "0xdef".to_string(),
            fee_recipient: "0x123".to_string(),
            gas_limit: "30000000".to_string(),
            gas_used: "15000000".to_string(),
            timestamp: "1234567890".to_string(),
        };

        assert_eq!(payload.block_number_u64().unwrap(), 12345);
        assert_eq!(
            payload.state_root_bytes().unwrap(),
            hex::decode("1234567890abcdef").unwrap()
        );
    }
}

