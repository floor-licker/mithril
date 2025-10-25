//! Beacon chain API client
//!
//! Implements the Ethereum Beacon API specification to query validator information,
//! block data, and network state.

use reqwest::Client;
use std::time::Duration;

use crate::errors::BeaconApiError;
use crate::types::{
    BeaconApiResponse, BeaconApiV2BlockResponse, BeaconBlock, ForkData, GenesisData, ValidatorInfo,
};

/// Client for interacting with Ethereum Beacon Chain API
///
/// This client implements a subset of the Beacon API specification needed for
/// Mithril integration, including validator queries, block retrieval, and
/// network information.
///
/// ## Beacon API Specification
/// https://ethereum.github.io/beacon-APIs/
pub struct BeaconClient {
    endpoint: String,
    client: Client,
}

impl BeaconClient {
    /// Create a new beacon client
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The beacon node API endpoint (e.g., "http://localhost:5052")
    ///
    /// # Example
    ///
    /// ```
    /// use mithril_ethereum_chain::BeaconClient;
    ///
    /// let client = BeaconClient::new("http://localhost:5052");
    /// ```
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: Client::builder()
                .timeout(Duration::from_secs(120)) // Increased for large validator queries
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Get the current slot number
    ///
    /// # Errors
    ///
    /// Returns error if the API request fails or response cannot be parsed
    pub async fn get_current_slot(&self) -> Result<u64, BeaconApiError> {
        // Query head state to get current slot
        let url = format!("{}/eth/v1/beacon/headers/head", self.endpoint);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(BeaconApiError::NodeError(format!(
                "Status: {}",
                response.status()
            )));
        }

        let json: serde_json::Value = response.json().await?;
        let slot_str = json["data"]["header"]["message"]["slot"]
            .as_str()
            .ok_or_else(|| BeaconApiError::DeserializationError("Missing slot field".into()))?;
        
        slot_str.parse().map_err(|e| {
            BeaconApiError::DeserializationError(format!("Invalid slot number: {}", e))
        })
    }

    /// Get validators for a specific epoch
    ///
    /// # Arguments
    ///
    /// * `epoch` - The epoch number to query
    ///
    /// # Returns
    ///
    /// A vector of validator information
    pub async fn get_validators_by_epoch(
        &self,
        epoch: u64,
    ) -> Result<Vec<ValidatorInfo>, BeaconApiError> {
        // Calculate slot for epoch (first slot of epoch)
        let slot = epoch * 32; // Ethereum has 32 slots per epoch
        let state_id = format!("{}", slot);
        
        let url = format!(
            "{}/eth/v1/beacon/states/{}/validators",
            self.endpoint, state_id
        );

        let response = self.client.get(&url).send().await?;

        if response.status().as_u16() == 404 {
            return Err(BeaconApiError::NotFound(format!("Epoch {} not found", epoch)));
        }

        if !response.status().is_success() {
            return Err(BeaconApiError::NodeError(format!(
                "Status: {}",
                response.status()
            )));
        }

        let api_response: BeaconApiResponse<Vec<ValidatorInfo>> = response.json().await?;
        Ok(api_response.data)
    }

    /// Get a beacon block by slot number
    ///
    /// # Arguments
    ///
    /// * `slot` - The slot number
    pub async fn get_block_by_slot(&self, slot: u64) -> Result<BeaconBlock, BeaconApiError> {
        self.get_block_by_slot_str(&slot.to_string()).await
    }

    /// Get a beacon block by slot identifier
    ///
    /// # Arguments
    ///
    /// * `slot_id` - The slot identifier (number, "head", "finalized", "genesis")
    pub async fn get_block_by_slot_str(&self, slot_id: &str) -> Result<BeaconBlock, BeaconApiError> {
        let url = format!("{}/eth/v2/beacon/blocks/{}", self.endpoint, slot_id);

        let response = self.client.get(&url).send().await?;

        if response.status().as_u16() == 404 {
            return Err(BeaconApiError::NotFound(format!(
                "Block at slot {} not found",
                slot_id
            )));
        }

        if !response.status().is_success() {
            return Err(BeaconApiError::NodeError(format!(
                "Status: {}",
                response.status()
            )));
        }

        // v2 API returns version info along with the block data
        let api_response: BeaconApiV2BlockResponse = response.json().await?;
        Ok(api_response.data)
    }

    /// Get genesis information
    pub async fn get_genesis(&self) -> Result<GenesisData, BeaconApiError> {
        let url = format!("{}/eth/v1/beacon/genesis", self.endpoint);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(BeaconApiError::NodeError(format!(
                "Status: {}",
                response.status()
            )));
        }

        let api_response: BeaconApiResponse<GenesisData> = response.json().await?;
        Ok(api_response.data)
    }

    /// Get current fork information
    pub async fn get_fork(&self) -> Result<ForkData, BeaconApiError> {
        let url = format!("{}/eth/v1/beacon/states/head/fork", self.endpoint);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(BeaconApiError::NodeError(format!(
                "Status: {}",
                response.status()
            )));
        }

        let api_response: BeaconApiResponse<ForkData> = response.json().await?;
        Ok(api_response.data)
    }

    /// Get genesis validators root
    pub async fn get_genesis_validators_root(&self) -> Result<String, BeaconApiError> {
        let genesis = self.get_genesis().await?;
        Ok(genesis.genesis_validators_root)
    }

    /// Get genesis time (Unix timestamp)
    pub async fn get_genesis_time(&self) -> Result<i64, BeaconApiError> {
        let genesis = self.get_genesis().await?;
        genesis.genesis_time_i64().map_err(|e| {
            BeaconApiError::DeserializationError(format!("Invalid genesis time: {}", e))
        })
    }

    /// Get current fork version
    pub async fn get_current_fork_version(&self) -> Result<String, BeaconApiError> {
        let fork = self.get_fork().await?;
        Ok(fork.current_version)
    }

    /// Get validator by public key
    ///
    /// # Arguments
    ///
    /// * `pubkey` - The validator's BLS public key (hex string)
    /// * `epoch` - The epoch to query
    pub async fn get_validator_by_pubkey(
        &self,
        pubkey: &str,
        epoch: u64,
    ) -> Result<ValidatorInfo, BeaconApiError> {
        let slot = epoch * 32;
        let state_id = format!("{}", slot);
        
        let url = format!(
            "{}/eth/v1/beacon/states/{}/validators/{}",
            self.endpoint, state_id, pubkey
        );

        let response = self.client.get(&url).send().await?;

        if response.status().as_u16() == 404 {
            return Err(BeaconApiError::NotFound(format!(
                "Validator {} not found",
                pubkey
            )));
        }

        if !response.status().is_success() {
            return Err(BeaconApiError::NodeError(format!(
                "Status: {}",
                response.status()
            )));
        }

        let api_response: BeaconApiResponse<ValidatorInfo> = response.json().await?;
        Ok(api_response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires actual beacon node
    async fn test_beacon_client_creation() {
        let client = BeaconClient::new("http://localhost:5052");
        assert_eq!(client.endpoint, "http://localhost:5052");
    }

    // Note: Integration tests that require a running beacon node should be in a separate
    // integration test file and marked with #[ignore] by default
}

