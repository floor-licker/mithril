# Mithril Ethereum Chain

Ethereum blockchain integration for Mithril, enabling fast-sync for Ethereum nodes using stake-based threshold signatures.

## Overview

This crate provides the implementation of Mithril's universal chain observer for Ethereum, allowing Ethereum validators to participate in Mithril's certification process.

## Features

- Beacon Chain API client for querying validator information
- Ethereum chain observer implementing the universal trait
- State root certification for the execution layer
- Support for mainnet, Holesky, and Sepolia testnets

## Architecture

```
EthereumChainObserver
    |
    v
BeaconClient --> Ethereum Beacon API
    |
    v
Execution Layer State Root (certified)
```

## Usage

### Basic Setup

```rust
use mithril_ethereum_chain::{BeaconClient, EthereumChainObserver};
use mithril_universal::UniversalChainObserver;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create beacon client pointing to your beacon node
    let beacon_client = BeaconClient::new("http://localhost:5052");
    
    // Create Ethereum observer
    let observer = EthereumChainObserver::new(beacon_client, "mainnet");
    
    // Query current epoch
    let epoch = observer.get_current_epoch().await?;
    println!("Current Ethereum epoch: {}", epoch.epoch_number);
    
    // Get validator set for current epoch
    let stake_dist = observer.get_stake_distribution(epoch.epoch_number).await?;
    println!("Active validators: {}", stake_dist.validator_count());
    
    // Compute state commitment
    let commitment = observer.compute_state_commitment(epoch.epoch_number).await?;
    println!("State root: {}", hex::encode(&commitment.value));
    
    Ok(())
}
```

### Custom Certification Interval

By default, Ethereum states are certified every 675 epochs (approximately 3 days). This can be customized:

```rust
let observer = EthereumChainObserver::new(beacon_client, "mainnet")
    .with_certification_interval(100); // Certify every 100 epochs
```

## Ethereum Specifics

### Epochs and Slots

- Ethereum has 32 slots per epoch
- Each slot is 12 seconds
- One epoch = 6.4 minutes

### State Commitment

The state commitment for Ethereum is the execution layer state root at the last slot of an epoch. This includes:

- `state_root`: The execution layer state trie root
- `block_number`: The execution layer block number
- Metadata: slot number, block hash, beacon root, parent hash

### Validator Set

Validators are queried from the beacon chain API. Only active validators (status `active_ongoing`, `active_exiting`, or `active_slashed`) are included in the stake distribution.

Each validator's stake is their effective balance in Gwei (max 32 ETH = 32,000,000,000 Gwei).

## Requirements

To use this crate, you need access to an Ethereum beacon node with the Beacon API enabled:

- Lighthouse: `--http` flag
- Prysm: `--http-enable` flag
- Teku: `--rest-api-enabled` flag
- Nimbus: `--rest` flag

## Testing

### Unit Tests

```bash
cargo test --package mithril-ethereum-chain
```

### Integration Tests (requires beacon node)

```bash
# Run ignored tests that connect to localhost:5052
cargo test --package mithril-ethereum-chain -- --ignored
```

### With a Local Beacon Node

1. Run a beacon node with API enabled:
```bash
# Example with Lighthouse
lighthouse beacon_node --http
```

2. Run integration tests:
```bash
cargo test --package mithril-ethereum-chain -- --ignored
```

## Beacon API Endpoints Used

This implementation uses the following Beacon API endpoints:

- `GET /eth/v1/beacon/headers/head` - Current slot
- `GET /eth/v1/beacon/states/{state_id}/validators` - Validator set
- `GET /eth/v2/beacon/blocks/{block_id}` - Block data with execution payload
- `GET /eth/v1/beacon/genesis` - Genesis information
- `GET /eth/v1/beacon/states/head/fork` - Fork information

## Limitations

### Current Implementation

- Does not implement validator sampling (all validators are included)
- Does not handle beacon chain reorgs
- Assumes post-merge (execution payload present in all blocks)
- No caching of API responses

### Future Improvements

- Validator sampling for scalability (Ethereum has 1M+ validators)
- Reorg detection and handling
- Response caching
- Support for alternative beacon node implementations
- Metrics and monitoring

## Configuration

### Environment Variables

- `BEACON_API_ENDPOINT`: Override default beacon node endpoint
- `BEACON_API_TIMEOUT_SECONDS`: Override default 30s timeout

### Network Support

Supported networks:
- `mainnet` - Ethereum mainnet
- `holesky` - Holesky testnet
- `sepolia` - Sepolia testnet

## Performance Considerations

### API Call Frequency

The beacon API is called for:
- Every epoch query: 1 API call
- Every stake distribution query: 1 API call (can return 1M+ validators)
- Every state commitment: 1 API call

### Large Validator Sets

Ethereum mainnet has over 1 million validators. Querying the full validator set can:
- Take several seconds
- Return 100+ MB of JSON data
- Require significant memory to process

For production use, implement validator sampling or filtering.

## Examples

See the `tests/integration_test.rs` file for complete examples of:
- Querying current epoch
- Getting stake distribution
- Computing state commitment
- Checking validator status

## License

Apache 2.0 (same as Mithril)

## See Also

- [Mithril Universal](../mithril-universal/) - Chain abstraction layer
- [Ethereum Beacon API Specification](https://ethereum.github.io/beacon-APIs/)
- [Mithril Design Document](../MITHRIL_ANYWHERE_DESIGN.md)

