//! # Mithril Universal
//!
//! Universal chain abstraction layer for Mithril, enabling support for multiple
//! proof-of-stake blockchains beyond Cardano.
//!
//! ## Overview
//!
//! This crate provides the core abstractions needed to integrate any proof-of-stake
//! blockchain with Mithril's stake-based threshold signature scheme. The main trait
//! is [`UniversalChainObserver`], which defines the interface any chain must implement.
//!
//! ## Example
//!
//! ```rust,no_run
//! use mithril_universal::{UniversalChainObserver, ChainId, EpochInfo};
//!
//! # struct MyChainObserver;
//! # #[async_trait::async_trait]
//! # impl UniversalChainObserver for MyChainObserver {
//! #     fn chain_id(&self) -> ChainId { ChainId::new("my-chain") }
//! #     async fn get_current_epoch(&self) -> Result<EpochInfo, mithril_universal::ChainObserverError> {
//! #         todo!()
//! #     }
//! #     async fn get_stake_distribution(&self, epoch: u64) -> Result<mithril_universal::StakeDistribution, mithril_universal::ChainObserverError> {
//! #         todo!()
//! #     }
//! #     async fn compute_state_commitment(&self, epoch: u64) -> Result<mithril_universal::StateCommitment, mithril_universal::ChainObserverError> {
//! #         todo!()
//! #     }
//! #     async fn is_validator_active(&self, validator_id: &mithril_universal::ValidatorId, epoch: u64) -> Result<bool, mithril_universal::ChainObserverError> {
//! #         todo!()
//! #     }
//! # }
//! # async fn example() {
//! let observer = MyChainObserver;
//! let epoch = observer.get_current_epoch().await.unwrap();
//! println!("Current epoch: {}", epoch.epoch_number);
//! # }
//! ```

#![warn(missing_docs)]

mod chain_observer;
mod errors;
mod types;

#[cfg(feature = "cardano-adapter")]
pub mod adapters;

pub use chain_observer::UniversalChainObserver;
pub use errors::ChainObserverError;
pub use types::{
    ChainId, CommitmentType, EpochInfo, StateCommitment, StakeDistribution, ValidatorId,
};

