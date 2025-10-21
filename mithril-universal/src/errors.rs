//! Error types for chain observers

use thiserror::Error;

/// Errors that can occur when interacting with a chain observer
#[derive(Error, Debug)]
pub enum ChainObserverError {
    /// Failed to connect to the chain
    #[error("Failed to connect to chain: {0}")]
    ConnectionError(String),

    /// Failed to query epoch information
    #[error("Failed to query epoch data: {0}")]
    EpochQueryError(String),

    /// Failed to retrieve stake distribution
    #[error("Failed to get stake distribution: {0}")]
    StakeDistributionError(String),

    /// Failed to compute state commitment
    #[error("Failed to compute state commitment: {0}")]
    StateCommitmentError(String),

    /// Invalid data received from the chain
    #[error("Invalid data received: {0}")]
    InvalidData(String),

    /// Validator not found or inactive
    #[error("Validator not found or inactive: {0}")]
    ValidatorNotFound(String),

    /// Chain-specific error
    #[error("Chain-specific error: {0}")]
    ChainSpecific(String),

    /// Generic error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result type for chain observer operations
#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, ChainObserverError>;

