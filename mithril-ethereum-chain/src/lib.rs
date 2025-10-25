//! # Mithril Ethereum Chain
//!
//! Ethereum blockchain integration for Mithril, enabling fast-sync for Ethereum nodes
//! using stake-based threshold signatures.
//!
//! This crate provides:
//! - Beacon chain API client for querying validator sets and state
//! - Ethereum chain observer implementing the universal trait
//! - State root certification for execution layer
//!
//! ## Example
//!
//! ```rust,no_run
//! use mithril_ethereum_chain::{BeaconClient, EthereumChainObserver};
//! use mithril_universal::UniversalChainObserver;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let beacon_client = BeaconClient::new("http://localhost:5052");
//! let observer = EthereumChainObserver::new(
//!     beacon_client,
//!     "mainnet"
//! );
//!
//! let epoch = observer.get_current_epoch().await?;
//! println!("Current Ethereum epoch: {}", epoch.epoch_number);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

mod beacon_client;
mod chain_observer;
mod errors;
pub mod types;

// Test utilities - only included in test/dev builds
#[cfg(any(test, debug_assertions))]
pub mod test_utils;

pub use beacon_client::BeaconClient;
pub use chain_observer::EthereumChainObserver;
pub use errors::{BeaconApiError, EthereumChainError};
pub use types::{BeaconBlock, ExecutionPayload, ValidatorInfo, ValidatorStatus};

