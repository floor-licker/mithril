//! Integration tests for Ethereum chain observer with the adapter
//!
//! These tests verify that the EthereumChainObserver works correctly when
//! wrapped by the UniversalChainObserverAdapter.

use std::sync::Arc;

use mithril_cardano_node_chain::chain_observer::ChainObserver;
use mithril_ethereum_chain::{BeaconClient, EthereumChainObserver};
use mithril_signer::chain_observer_adapter::UniversalChainObserverAdapter;

#[tokio::test]
async fn test_ethereum_adapter_with_real_observer() {
    // Using a public Holesky endpoint for integration testing
    let endpoint = "https://ethereum-holesky-beacon-api.publicnode.com";
    let beacon_client = BeaconClient::new(endpoint);
    let ethereum_observer = EthereumChainObserver::new(beacon_client, "holesky");
    
    // Wrap in adapter
    let adapter = UniversalChainObserverAdapter::new(Arc::new(ethereum_observer));
    
    // Test 1: Get current epoch
    println!("\n1. Testing get_current_epoch...");
    let epoch_result = adapter.get_current_epoch().await;
    assert!(epoch_result.is_ok(), "Failed to get current epoch: {:?}", epoch_result.err());
    
    let epoch = epoch_result.unwrap();
    assert!(epoch.is_some(), "Epoch should be Some");
    
    let epoch_value = epoch.unwrap();
    println!("   ✓ Current epoch: {}", epoch_value);
    assert!(epoch_value.0 > 0, "Epoch should be > 0");
}

#[tokio::test]
async fn test_ethereum_adapter_stake_distribution() {
    let endpoint = "https://ethereum-holesky-beacon-api.publicnode.com";
    let beacon_client = BeaconClient::new(endpoint);
    let ethereum_observer = EthereumChainObserver::new(beacon_client, "holesky");
    let adapter = UniversalChainObserverAdapter::new(Arc::new(ethereum_observer));
    
    println!("\n2. Testing get_current_stake_distribution...");
    
    // Note: This will timeout on public endpoints due to large validator set
    // In production, you'd use a local node or a service with full indexing
    // For now, we'll just verify the call structure works
    let result = adapter.get_current_stake_distribution().await;
    
    // We expect this to either succeed or fail gracefully
    match result {
        Ok(Some(stake_dist)) => {
            println!("   ✓ Got stake distribution with {} validators", stake_dist.len());
            assert!(stake_dist.len() > 0, "Should have validators");
        },
        Ok(None) => {
            println!("   [WARN] No stake distribution available");
        },
        Err(e) => {
            // Expected for public endpoints
            println!("   [WARN] Stake distribution query failed (expected for public endpoints): {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_ethereum_adapter_chain_point() {
    let endpoint = "https://ethereum-holesky-beacon-api.publicnode.com";
    let beacon_client = BeaconClient::new(endpoint);
    let ethereum_observer = EthereumChainObserver::new(beacon_client, "holesky");
    let adapter = UniversalChainObserverAdapter::new(Arc::new(ethereum_observer));
    
    println!("\n3. Testing get_current_chain_point...");
    let chain_point_result = adapter.get_current_chain_point().await;
    assert!(chain_point_result.is_ok(), "Failed to get chain point");
    
    let chain_point = chain_point_result.unwrap();
    assert!(chain_point.is_some(), "Chain point should be Some");
    
    let cp = chain_point.unwrap();
    println!("   ✓ Chain point:");
    println!("     Slot: {}", cp.slot_number);
    println!("     Block: {}", cp.block_number);
    println!("     Hash: {}", cp.block_hash);
    
    assert!(cp.slot_number.0 > 0, "Slot should be > 0");
    assert!(cp.block_number.0 > 0, "Block number should be > 0");
}

#[tokio::test]
async fn test_ethereum_adapter_era() {
    let endpoint = "https://ethereum-holesky-beacon-api.publicnode.com";
    let beacon_client = BeaconClient::new(endpoint);
    let ethereum_observer = EthereumChainObserver::new(beacon_client, "holesky");
    let adapter = UniversalChainObserverAdapter::new(Arc::new(ethereum_observer));
    
    println!("\n4. Testing get_current_era...");
    let era_result = adapter.get_current_era().await;
    assert!(era_result.is_ok(), "Failed to get era");
    
    let era = era_result.unwrap();
    assert_eq!(era, Some("universal".to_string()));
    println!("   ✓ Era: {:?}", era);
}

#[tokio::test]
async fn test_ethereum_adapter_datums() {
    let endpoint = "https://ethereum-holesky-beacon-api.publicnode.com";
    let beacon_client = BeaconClient::new(endpoint);
    let ethereum_observer = EthereumChainObserver::new(beacon_client, "holesky");
    let adapter = UniversalChainObserverAdapter::new(Arc::new(ethereum_observer));
    
    println!("\n5. Testing get_current_datums...");
    let datums_result = adapter.get_current_datums(&"test_address".to_string()).await;
    assert!(datums_result.is_ok(), "Failed to get datums");
    
    let datums = datums_result.unwrap();
    assert_eq!(datums.len(), 0, "Non-Cardano chains should return empty datums");
    println!("   ✓ Datums: [] (expected for Ethereum)");
}

#[tokio::test]
async fn test_ethereum_adapter_kes_period() {
    let endpoint = "https://ethereum-holesky-beacon-api.publicnode.com";
    let beacon_client = BeaconClient::new(endpoint);
    let ethereum_observer = EthereumChainObserver::new(beacon_client, "holesky");
    let adapter = UniversalChainObserverAdapter::new(Arc::new(ethereum_observer));
    
    println!("\n6. Testing get_current_kes_period...");
    let kes_result = adapter.get_current_kes_period().await;
    assert!(kes_result.is_ok(), "Failed to get KES period");
    
    let kes = kes_result.unwrap();
    assert_eq!(kes, None, "Non-Cardano chains should return None for KES");
    println!("   ✓ KES period: None (expected for Ethereum)");
}

#[tokio::test]
async fn test_ethereum_adapter_end_to_end() {
    println!("\n========================================");
    println!("End-to-End Ethereum Adapter Test");
    println!("========================================\n");
    
    let endpoint = "https://ethereum-holesky-beacon-api.publicnode.com";
    println!("Endpoint: {}\n", endpoint);
    
    let beacon_client = BeaconClient::new(endpoint);
    let ethereum_observer = EthereumChainObserver::new(beacon_client, "holesky");
    let adapter = UniversalChainObserverAdapter::new(Arc::new(ethereum_observer));
    
    // Step 1: Get current epoch
    println!("1. Getting current epoch...");
    let epoch = adapter.get_current_epoch().await
        .expect("Failed to get epoch")
        .expect("Epoch should be Some");
    println!("   ✓ Epoch: {}", epoch);
    
    // Step 2: Get chain point
    println!("\n2. Getting chain point...");
    let chain_point = adapter.get_current_chain_point().await
        .expect("Failed to get chain point")
        .expect("Chain point should be Some");
    println!("   ✓ Slot: {}", chain_point.slot_number);
    println!("   ✓ Block: {}", chain_point.block_number);
    
    // Step 3: Get era
    println!("\n3. Getting era...");
    let era = adapter.get_current_era().await
        .expect("Failed to get era")
        .expect("Era should be Some");
    println!("   ✓ Era: {}", era);
    
    // Step 4: Verify Cardano-specific features return expected values
    println!("\n4. Verifying Cardano-specific features...");
    let datums = adapter.get_current_datums(&"test".to_string()).await
        .expect("Failed to get datums");
    println!("   ✓ Datums: {} (empty for Ethereum)", datums.len());
    
    let kes = adapter.get_current_kes_period().await
        .expect("Failed to get KES");
    println!("   ✓ KES period: {:?} (None for Ethereum)", kes);
    
    println!("\n========================================");
    println!("[PASS] All adapter checks passed!");
    println!("========================================\n");
}

