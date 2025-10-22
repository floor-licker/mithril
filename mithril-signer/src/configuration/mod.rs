//! Configuration module

mod chain_config;

pub use chain_config::{CardanoConfig, ChainType, EthereumConfig};

// Re-export the main Configuration from configuration.rs for backward compatibility
mod config_impl;
pub use config_impl::{Configuration, DefaultConfiguration};

