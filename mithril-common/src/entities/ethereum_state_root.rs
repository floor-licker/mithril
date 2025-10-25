use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::entities::Epoch;

/// Ethereum State Root artifact
///
/// This artifact contains the certified Ethereum execution layer state root
/// at a specific epoch boundary, allowing trustless verification of Ethereum state.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct EthereumStateRoot {
    /// Epoch at which this state root was captured
    pub epoch: Epoch,

    /// State root hash (0x-prefixed hex string)
    pub state_root: String,

    /// Beacon block number at which this state root was captured
    pub block_number: u64,

    /// Hash of the artifact (computed from all fields)
    pub hash: String,

    /// Date and time of creation
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl EthereumStateRoot {
    /// Create a new Ethereum State Root artifact
    pub fn new(epoch: Epoch, state_root: String, block_number: u64) -> Self {
        let mut instance = Self {
            epoch,
            state_root,
            block_number,
            hash: String::new(),
            created_at: chrono::Utc::now(),
        };
        instance.hash = instance.compute_hash();
        instance
    }

    /// Compute the hash of this artifact
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}", self.epoch.0).as_bytes());
        hasher.update(self.state_root.as_bytes());
        hasher.update(self.block_number.to_string().as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash_is_deterministic() {
        let artifact1 = EthereumStateRoot::new(
            Epoch(100),
            "0x1234567890abcdef".to_string(),
            12345,
        );
        let artifact2 = EthereumStateRoot::new(
            Epoch(100),
            "0x1234567890abcdef".to_string(),
            12345,
        );

        assert_eq!(artifact1.compute_hash(), artifact2.compute_hash());
    }

    #[test]
    fn test_hash_changes_when_epoch_changes() {
        let artifact1 = EthereumStateRoot::new(
            Epoch(100),
            "0x1234567890abcdef".to_string(),
            12345,
        );
        let artifact2 = EthereumStateRoot::new(
            Epoch(101),
            "0x1234567890abcdef".to_string(),
            12345,
        );

        assert_ne!(artifact1.hash, artifact2.hash);
    }

    #[test]
    fn test_hash_changes_when_state_root_changes() {
        let artifact1 = EthereumStateRoot::new(
            Epoch(100),
            "0x1234567890abcdef".to_string(),
            12345,
        );
        let artifact2 = EthereumStateRoot::new(
            Epoch(100),
            "0xfedcba0987654321".to_string(),
            12345,
        );

        assert_ne!(artifact1.hash, artifact2.hash);
    }

    #[test]
    fn test_hash_changes_when_block_number_changes() {
        let artifact1 = EthereumStateRoot::new(
            Epoch(100),
            "0x1234567890abcdef".to_string(),
            12345,
        );
        let artifact2 = EthereumStateRoot::new(
            Epoch(100),
            "0x1234567890abcdef".to_string(),
            54321,
        );

        assert_ne!(artifact1.hash, artifact2.hash);
    }
}

