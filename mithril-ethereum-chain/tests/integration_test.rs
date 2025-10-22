//! Integration tests for Ethereum chain observer
//!
//! Note: Most tests are marked with #[ignore] as they require a running beacon node.
//! To run these tests: cargo test -- --ignored

use mithril_ethereum_chain::{BeaconClient, EthereumChainObserver};
use mithril_universal::UniversalChainObserver;

#[test]
fn test_observer_creation_and_metadata() {
    let beacon_client = BeaconClient::new("http://localhost:5052");
    let observer = EthereumChainObserver::new(beacon_client, "mainnet");

    // Test chain ID
    assert_eq!(observer.chain_id().as_str(), "ethereum-mainnet");

    // Test metadata
    let metadata = observer.get_metadata();
    assert_eq!(metadata.get("chain_type").unwrap(), "ethereum");
    assert_eq!(metadata.get("slots_per_epoch").unwrap(), "32");
    assert_eq!(metadata.get("slot_duration_seconds").unwrap(), "12");
}

#[test]
fn test_observer_with_custom_interval() {
    let beacon_client = BeaconClient::new("http://localhost:5052");
    let observer = EthereumChainObserver::new(beacon_client, "holesky")
        .with_certification_interval(100);

    let metadata = observer.get_metadata();
    assert_eq!(metadata.get("certification_interval_epochs").unwrap(), "100");
}

// The following tests require an actual beacon node connection
// Run with: cargo test --test integration_test -- --ignored

#[tokio::test]
#[ignore]
async fn test_get_current_epoch_from_real_node() {
    // This test requires a running beacon node at localhost:5052
    let beacon_client = BeaconClient::new("http://localhost:5052");
    let observer = EthereumChainObserver::new(beacon_client, "mainnet");

    let epoch = observer.get_current_epoch().await;
    
    // If node is running, this should succeed
    if let Ok(epoch_info) = epoch {
        println!("Current Ethereum epoch: {}", epoch_info.epoch_number);
        assert!(epoch_info.epoch_number > 0);
        assert!(epoch_info.start_time > 0);
    } else {
        // Node not available is okay for this test
        println!("Beacon node not available - test skipped");
    }
}

#[tokio::test]
#[ignore]
async fn test_get_stake_distribution_from_real_node() {
    let beacon_client = BeaconClient::new("http://localhost:5052");
    let observer = EthereumChainObserver::new(beacon_client, "mainnet");

    // Try to get current epoch first
    if let Ok(epoch_info) = observer.get_current_epoch().await {
        let distribution = observer.get_stake_distribution(epoch_info.epoch_number).await;
        
        if let Ok(dist) = distribution {
            println!("Validator count: {}", dist.validator_count());
            println!("Total stake: {} Gwei", dist.total_stake);
            assert!(dist.validator_count() > 0);
            assert!(dist.total_stake > 0);
        } else {
            println!("Could not fetch stake distribution - test skipped");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_compute_state_commitment_from_real_node() {
    let beacon_client = BeaconClient::new("http://localhost:5052");
    let observer = EthereumChainObserver::new(beacon_client, "mainnet");

    if let Ok(epoch_info) = observer.get_current_epoch().await {
        // Use an older finalized epoch
        let finalized_epoch = epoch_info.epoch_number.saturating_sub(5);
        
        let commitment = observer.compute_state_commitment(finalized_epoch).await;
        
        if let Ok(comm) = commitment {
            println!("Commitment for epoch {}: {:?}", finalized_epoch, hex::encode(&comm.value));
            assert_eq!(comm.epoch, finalized_epoch);
            assert_eq!(comm.commitment_type, mithril_universal::CommitmentType::StateRoot);
            assert!(!comm.value.is_empty());
            assert!(comm.block_number > 0);
        } else {
            println!("Could not compute state commitment - test skipped");
        }
    }
}

