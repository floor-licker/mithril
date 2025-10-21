# Mithril Universal

Universal chain abstraction layer for Mithril, enabling support for multiple proof-of-stake blockchains.

## Overview

`mithril-universal` provides the core abstractions needed to integrate any proof-of-stake blockchain with Mithril's stake-based threshold signature scheme. Instead of being limited to Cardano, Mithril can now support Ethereum, Solana, Polkadot, and any other PoS chain.

## Features

- **Chain-agnostic traits**: Define once, implement for any blockchain
- **Type-safe abstractions**: Rust's type system ensures correct usage
- **Extensible design**: Easy to add support for new chains
- **Backward compatible**: Existing Cardano functionality unchanged

## Core Trait: UniversalChainObserver

The main abstraction is the `UniversalChainObserver` trait, which any blockchain must implement:

```rust
#[async_trait]
pub trait UniversalChainObserver: Send + Sync {
    fn chain_id(&self) -> ChainId;
    async fn get_current_epoch(&self) -> Result<EpochInfo, ChainObserverError>;
    async fn get_stake_distribution(&self, epoch: u64) -> Result<StakeDistribution, ChainObserverError>;
    async fn compute_state_commitment(&self, epoch: u64) -> Result<StateCommitment, ChainObserverError>;
    async fn is_validator_active(&self, validator_id: &ValidatorId, epoch: u64) -> Result<bool, ChainObserverError>;
}
```

## Example: Mock Implementation

```rust
use async_trait::async_trait;
use mithril_universal::{
    UniversalChainObserver, ChainId, EpochInfo,
    StakeDistribution, StateCommitment, ValidatorId,
    ChainObserverError, CommitmentType,
};

struct MyChainObserver {
    chain_id: ChainId,
}

#[async_trait]
impl UniversalChainObserver for MyChainObserver {
    fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }

    async fn get_current_epoch(&self) -> Result<EpochInfo, ChainObserverError> {
        // Query your chain's current epoch
        Ok(EpochInfo {
            chain_id: self.chain_id.clone(),
            epoch_number: 100,
            start_time: 1234567890,
            end_time: None,
        })
    }

    async fn get_stake_distribution(
        &self,
        epoch: u64,
    ) -> Result<StakeDistribution, ChainObserverError> {
        // Query validator set and stakes
        let mut distribution = StakeDistribution::new(epoch);
        distribution.add_validator(ValidatorId::new("validator1"), 1000);
        Ok(distribution)
    }

    async fn compute_state_commitment(
        &self,
        epoch: u64,
    ) -> Result<StateCommitment, ChainObserverError> {
        // Compute state commitment (hash, root, etc.)
        Ok(StateCommitment::new(
            self.chain_id.clone(),
            epoch,
            CommitmentType::StateRoot,
            vec![1, 2, 3, 4],
            12345,
        ))
    }

    async fn is_validator_active(
        &self,
        validator_id: &ValidatorId,
        epoch: u64,
    ) -> Result<bool, ChainObserverError> {
        // Check if validator is active
        Ok(true)
    }
}
```

## Cardano Adapter

The crate includes an adapter that wraps Mithril's existing Cardano observer:

```rust
use mithril_universal::adapters::CardanoChainObserverAdapter;
use std::sync::Arc;

// With feature = "cardano-adapter"
let adapter = CardanoChainObserverAdapter::new(
    cardano_observer,
    "mainnet"
);

let epoch = adapter.get_current_epoch().await?;
```

This proves the abstraction works without modifying existing Cardano code.

## Key Types

### ChainId
Unique identifier for a blockchain (e.g., "ethereum-mainnet", "cardano-preprod")

### EpochInfo
Information about a chain epoch including number and timing

### StakeDistribution
Map of validators to their stake amounts for a given epoch

### StateCommitment
Commitment to chain state (state root, accounts hash, etc.)

### CommitmentType
Enum of different commitment types:
- `StateRoot` - Ethereum-style state root
- `AccountsHash` - Solana-style accounts hash
- `ImmutableFileSet` - Cardano-style immutable files
- `ParachainHead` - Polkadot-style parachain head
- `Custom(String)` - For other chains

## Design Principles

1. **Minimal assumptions**: Only assume what's common to all PoS chains
2. **Extensibility**: Easy to add chain-specific features
3. **Type safety**: Use Rust's type system to prevent mistakes
4. **Documentation**: Every public item has examples
5. **Testing**: Comprehensive tests with mock implementations

## Roadmap

### Phase 1: Foundation (Current)
- ✅ Core trait definitions
- ✅ Type system
- ✅ Error handling
- ✅ Cardano adapter
- ✅ Documentation and tests

### Phase 2: Ethereum Integration (Next)
- [ ] Ethereum beacon chain client
- [ ] Ethereum chain observer implementation
- [ ] State root certification
- [ ] Validator set queries

### Phase 3: Additional Chains
- [ ] Polkadot integration
- [ ] Cosmos integration
- [ ] Additional chain support

## Contributing

To add support for a new blockchain:

1. Implement `UniversalChainObserver` for your chain
2. Handle chain-specific epoch concepts
3. Map validator identifiers to ValidatorId
4. Compute appropriate state commitments
5. Add tests for your implementation

See the integration tests for examples of mock implementations.

## Architecture

```
┌─────────────────────────────────┐
│   Mithril STM (unchanged)       │  Core cryptography
└─────────────────────────────────┘
             ↑
┌─────────────────────────────────┐
│   mithril-universal             │  Chain abstraction
│   - UniversalChainObserver      │
│   - Common types                │
└─────────────────────────────────┘
             ↑
    ┌────────┴────────┐
┌───────┐      ┌──────────┐
│Cardano│      │ Ethereum │  ...
│Adapter│      │ Observer │
└───────┘      └──────────┘
```

## License

Apache 2.0 (same as Mithril)

## See Also

- [Mithril Documentation](https://mithril.network/doc)
- [Design Document](../MITHRIL_ANYWHERE_DESIGN.md)
- [Implementation Plan](../IMPLEMENTATION_PLAN.md)

