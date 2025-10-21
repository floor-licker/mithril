//! Universal chain observer trait definition

use async_trait::async_trait;

use crate::{
    ChainId, ChainObserverError, EpochInfo, StateCommitment, StakeDistribution, ValidatorId,
};

/// Universal chain observer that works across different blockchains
///
/// This trait defines the interface that any blockchain must implement to integrate
/// with Mithril's stake-based threshold signature scheme. It abstracts away the
/// chain-specific details while providing a consistent API for:
///
/// - Querying epoch information
/// - Retrieving validator stake distributions
/// - Computing state commitments that can be signed
/// - Checking validator status
///
/// # Example Implementation
///
/// ```rust,no_run
/// use mithril_universal::{
///     UniversalChainObserver, ChainId, EpochInfo, StakeDistribution,
///     StateCommitment, ValidatorId, ChainObserverError,
/// };
/// use async_trait::async_trait;
///
/// struct MyChainObserver {
///     chain_id: ChainId,
/// }
///
/// #[async_trait]
/// impl UniversalChainObserver for MyChainObserver {
///     fn chain_id(&self) -> ChainId {
///         self.chain_id.clone()
///     }
///
///     async fn get_current_epoch(&self) -> Result<EpochInfo, ChainObserverError> {
///         // Query your chain's current epoch
///         todo!()
///     }
///
///     async fn get_stake_distribution(
///         &self,
///         epoch: u64,
///     ) -> Result<StakeDistribution, ChainObserverError> {
///         // Query validator set and their stakes for the given epoch
///         todo!()
///     }
///
///     async fn compute_state_commitment(
///         &self,
///         epoch: u64,
///     ) -> Result<StateCommitment, ChainObserverError> {
///         // Compute the state commitment (hash, root, etc.) for the epoch
///         todo!()
///     }
///
///     async fn is_validator_active(
///         &self,
///         validator_id: &ValidatorId,
///         epoch: u64,
///     ) -> Result<bool, ChainObserverError> {
///         // Check if the validator is active in the given epoch
///         todo!()
///     }
/// }
/// ```
#[async_trait]
pub trait UniversalChainObserver: Send + Sync {
    /// Get the unique identifier for this chain
    ///
    /// This should be a stable identifier that uniquely identifies the blockchain
    /// and network (e.g., "cardano-mainnet", "ethereum-mainnet", "ethereum-holesky")
    fn chain_id(&self) -> ChainId;

    /// Get information about the current epoch
    ///
    /// Returns details about the current epoch including its number and timing.
    /// This is used to determine when to trigger certificate generation.
    async fn get_current_epoch(&self) -> Result<EpochInfo, ChainObserverError>;

    /// Get the stake distribution for a given epoch
    ///
    /// Returns a mapping of validator identifiers to their stake amounts.
    /// This is used by Mithril to weight signatures according to validator stake.
    ///
    /// # Arguments
    ///
    /// * `epoch` - The epoch number to query
    async fn get_stake_distribution(
        &self,
        epoch: u64,
    ) -> Result<StakeDistribution, ChainObserverError>;

    /// Compute the state commitment for a given epoch
    ///
    /// This is the core data that will be signed by validators. It should be a
    /// deterministic commitment to the chain's state at a specific point in time.
    ///
    /// For different chains, this might be:
    /// - Ethereum: execution layer state root at epoch boundary
    /// - Cardano: digest of immutable file set
    /// - Solana: accounts hash at epoch end
    /// - Polkadot: parachain state root
    ///
    /// # Arguments
    ///
    /// * `epoch` - The epoch number to compute the commitment for
    async fn compute_state_commitment(
        &self,
        epoch: u64,
    ) -> Result<StateCommitment, ChainObserverError>;

    /// Check if a validator is active in a given epoch
    ///
    /// Returns true if the validator is eligible to sign in the given epoch.
    /// This typically means the validator is registered, has stake, and has not
    /// been slashed or exited.
    ///
    /// # Arguments
    ///
    /// * `validator_id` - The validator to check
    /// * `epoch` - The epoch to check in
    async fn is_validator_active(
        &self,
        validator_id: &ValidatorId,
        epoch: u64,
    ) -> Result<bool, ChainObserverError>;

    /// Get metadata about this chain observer implementation
    ///
    /// Returns information about the chain observer implementation, such as
    /// version, capabilities, and configuration. This is optional and returns
    /// an empty map by default.
    fn get_metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }
}

