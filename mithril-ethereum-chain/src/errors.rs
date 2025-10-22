//! Error types for Ethereum chain integration

use thiserror::Error;

/// Errors that can occur when interacting with the Beacon API
#[derive(Error, Debug)]
pub enum BeaconApiError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),

    /// Failed to deserialize response
    #[error("Failed to deserialize response: {0}")]
    DeserializationError(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Beacon node error
    #[error("Beacon node error: {0}")]
    NodeError(String),

    /// Generic error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<reqwest::Error> for BeaconApiError {
    fn from(err: reqwest::Error) -> Self {
        BeaconApiError::RequestFailed(err.to_string())
    }
}

impl From<serde_json::Error> for BeaconApiError {
    fn from(err: serde_json::Error) -> Self {
        BeaconApiError::DeserializationError(err.to_string())
    }
}

/// Errors specific to Ethereum chain observer
#[derive(Error, Debug)]
pub enum EthereumChainError {
    /// Beacon API error
    #[error("Beacon API error: {0}")]
    BeaconApi(#[from] BeaconApiError),

    /// Failed to compute state commitment
    #[error("Failed to compute state commitment: {0}")]
    StateCommitment(String),

    /// No execution payload available
    #[error("No execution payload in block")]
    NoExecutionPayload,

    /// Invalid epoch
    #[error("Invalid epoch: {0}")]
    InvalidEpoch(String),

    /// Generic error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

