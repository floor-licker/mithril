//! Adapters for existing chain implementations
//!
//! This module provides adapters that wrap existing chain-specific implementations
//! to work with the universal chain observer trait.

#[cfg(feature = "cardano-adapter")]
mod cardano;

#[cfg(feature = "cardano-adapter")]
pub use cardano::CardanoChainObserverAdapter;

