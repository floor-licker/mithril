//! Core types for universal chain abstraction

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a blockchain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainId(String);

impl ChainId {
    /// Create a new chain identifier
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the chain ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Information about a chain epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochInfo {
    /// The chain this epoch belongs to
    pub chain_id: ChainId,

    /// Epoch number
    pub epoch_number: u64,

    /// Unix timestamp when this epoch started
    pub start_time: i64,

    /// Unix timestamp when this epoch ends (None if ongoing)
    pub end_time: Option<i64>,
}

/// Unique identifier for a validator
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValidatorId(String);

impl ValidatorId {
    /// Create a new validator identifier
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the validator ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ValidatorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Stake distribution for a given epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeDistribution {
    /// The epoch this distribution is for
    pub epoch: u64,

    /// Map of validator IDs to their stake amounts
    pub validators: HashMap<ValidatorId, u64>,

    /// Total stake across all validators
    pub total_stake: u64,
}

impl StakeDistribution {
    /// Create a new stake distribution
    pub fn new(epoch: u64) -> Self {
        Self {
            epoch,
            validators: HashMap::new(),
            total_stake: 0,
        }
    }

    /// Add a validator's stake
    pub fn add_validator(&mut self, validator_id: ValidatorId, stake: u64) {
        self.validators.insert(validator_id, stake);
        self.total_stake += stake;
    }

    /// Get the number of validators
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }

    /// Get a validator's stake
    pub fn get_stake(&self, validator_id: &ValidatorId) -> Option<u64> {
        self.validators.get(validator_id).copied()
    }
}

/// Type of state commitment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitmentType {
    /// Ethereum-style state root
    StateRoot,

    /// Solana-style accounts hash
    AccountsHash,

    /// Cardano-style immutable file set
    ImmutableFileSet,

    /// Polkadot-style parachain head
    ParachainHead,

    /// Custom commitment type for other chains
    Custom(String),
}

impl fmt::Display for CommitmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StateRoot => write!(f, "StateRoot"),
            Self::AccountsHash => write!(f, "AccountsHash"),
            Self::ImmutableFileSet => write!(f, "ImmutableFileSet"),
            Self::ParachainHead => write!(f, "ParachainHead"),
            Self::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// State commitment representing the chain's state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateCommitment {
    /// The chain this commitment is for
    pub chain_id: ChainId,

    /// The epoch this commitment represents
    pub epoch: u64,

    /// Type of commitment
    pub commitment_type: CommitmentType,

    /// The actual commitment value (hash, root, etc.)
    pub value: Vec<u8>,

    /// Block number or height associated with this commitment
    pub block_number: u64,

    /// Chain-specific metadata
    pub metadata: HashMap<String, String>,
}

impl StateCommitment {
    /// Create a new state commitment
    pub fn new(
        chain_id: ChainId,
        epoch: u64,
        commitment_type: CommitmentType,
        value: Vec<u8>,
        block_number: u64,
    ) -> Self {
        Self {
            chain_id,
            epoch,
            commitment_type,
            value,
            block_number,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the commitment
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get the commitment value as a hex string
    pub fn value_hex(&self) -> String {
        hex::encode(&self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id() {
        let chain_id = ChainId::new("ethereum-mainnet");
        assert_eq!(chain_id.as_str(), "ethereum-mainnet");
        assert_eq!(chain_id.to_string(), "ethereum-mainnet");
    }

    #[test]
    fn test_stake_distribution() {
        let mut distribution = StakeDistribution::new(100);
        distribution.add_validator(ValidatorId::new("val1"), 1000);
        distribution.add_validator(ValidatorId::new("val2"), 2000);

        assert_eq!(distribution.validator_count(), 2);
        assert_eq!(distribution.total_stake, 3000);
        assert_eq!(
            distribution.get_stake(&ValidatorId::new("val1")),
            Some(1000)
        );
    }

    #[test]
    fn test_commitment_type_display() {
        assert_eq!(CommitmentType::StateRoot.to_string(), "StateRoot");
        assert_eq!(
            CommitmentType::Custom("MyType".to_string()).to_string(),
            "Custom(MyType)"
        );
    }
}

