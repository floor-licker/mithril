//! Chain-specific configuration types

use config::{Map, Value};
use mithril_doc::{Documenter, StructDoc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Type of blockchain the signer operates on
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    /// Cardano blockchain
    Cardano,
    
    /// Ethereum blockchain
    Ethereum,
}

impl Default for ChainType {
    fn default() -> Self {
        // Default to Cardano for backward compatibility
        Self::Cardano
    }
}

impl std::fmt::Display for ChainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cardano => write!(f, "cardano"),
            Self::Ethereum => write!(f, "ethereum"),
        }
    }
}

/// Cardano-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Documenter)]
pub struct CardanoConfig {
    /// Cardano CLI tool path
    #[example = "`cardano-cli`"]
    pub cardano_cli_path: PathBuf,

    /// Path of the socket opened by the Cardano node
    #[example = "`/ipc/node.socket`"]
    pub cardano_node_socket_path: PathBuf,

    /// Cardano network
    #[example = "`mainnet` or `preprod` or `devnet`"]
    pub network: String,

    /// Cardano Network Magic number (useful for TestNet & DevNet)
    #[example = "`1097911063` or `42`"]
    pub network_magic: Option<u64>,

    /// Directory to snapshot (Cardano DB directory)
    pub db_directory: PathBuf,

    /// File path to the KES secret key of the pool
    pub kes_secret_key_path: Option<PathBuf>,

    /// File path to the operational certificate of the pool
    pub operational_certificate_path: Option<PathBuf>,
}

/// Ethereum-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Documenter)]
pub struct EthereumConfig {
    /// Ethereum Beacon node API endpoint
    #[example = "`http://localhost:5052`"]
    pub beacon_endpoint: String,

    /// Ethereum network
    #[example = "`mainnet` or `holesky` or `sepolia`"]
    pub network: String,

    /// Validator BLS public key (48 bytes hex, with or without 0x prefix)
    #[example = "`0x1234567890abcdef...`"]
    pub validator_pubkey: String,

    /// Path to validator BLS secret key file
    #[example = "`/keys/validator.key`"]
    pub validator_seckey_path: PathBuf,

    /// Certification interval in Ethereum epochs (default: 675 ~= 3 days)
    #[serde(default = "default_certification_interval")]
    pub certification_interval_epochs: u64,
}

fn default_certification_interval() -> u64 {
    675 // Approximately 3 days
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_type_default() {
        assert_eq!(ChainType::default(), ChainType::Cardano);
    }

    #[test]
    fn test_chain_type_display() {
        assert_eq!(ChainType::Cardano.to_string(), "cardano");
        assert_eq!(ChainType::Ethereum.to_string(), "ethereum");
    }

    #[test]
    fn test_chain_type_serde() {
        let cardano_json = r#""cardano""#;
        let ethereum_json = r#""ethereum""#;

        let cardano: ChainType = serde_json::from_str(cardano_json).unwrap();
        let ethereum: ChainType = serde_json::from_str(ethereum_json).unwrap();

        assert_eq!(cardano, ChainType::Cardano);
        assert_eq!(ethereum, ChainType::Ethereum);

        assert_eq!(serde_json::to_string(&ChainType::Cardano).unwrap(), cardano_json);
        assert_eq!(serde_json::to_string(&ChainType::Ethereum).unwrap(), ethereum_json);
    }

    #[test]
    fn test_ethereum_config_defaults() {
        let config_json = r#"{
            "beacon_endpoint": "http://localhost:5052",
            "network": "mainnet",
            "validator_pubkey": "0x1234",
            "validator_seckey_path": "/keys/validator.key"
        }"#;

        let config: EthereumConfig = serde_json::from_str(config_json).unwrap();
        assert_eq!(config.certification_interval_epochs, 675);
    }
}

