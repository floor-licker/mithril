//! Integration tests for Holesky testnet
//!
//! These tests require a connection to a Holesky beacon node.
//! 
//! Public Holesky beacon endpoints:
//! - https://ethereum-holesky-beacon-api.publicnode.com
//! - https://holesky-beacon.stakely.io
//! - Or run your own: lighthouse beacon_node --network holesky --http
//!
//! To run these tests:
//! ```
//! export HOLESKY_BEACON_ENDPOINT=https://ethereum-holesky-beacon-api.publicnode.com
//! cargo test --package mithril-ethereum-chain --test holesky_integration_test -- --ignored --nocapture
//! ```

use mithril_ethereum_chain::{BeaconClient, EthereumChainObserver};
use mithril_universal::UniversalChainObserver;
use std::env;

/// Get the Holesky beacon endpoint from environment or use default
fn get_holesky_endpoint() -> String {
    env::var("HOLESKY_BEACON_ENDPOINT")
        .unwrap_or_else(|_| "https://ethereum-holesky-beacon-api.publicnode.com".to_string())
}

#[tokio::test]
#[ignore] // Requires network access to Holesky beacon node
async fn test_connect_to_holesky() {
    let endpoint = get_holesky_endpoint();
    println!("Connecting to Holesky beacon at: {}", endpoint);
    
    let beacon_client = BeaconClient::new(&endpoint);
    let observer = EthereumChainObserver::new(beacon_client, "holesky")
        .with_certification_interval(100); // Shorter interval for testing

    // Test basic connectivity by getting current epoch
    let epoch_result = observer.get_current_epoch().await;
    
    match epoch_result {
        Ok(epoch_info) => {
            println!("[PASS] Successfully connected to Holesky!");
            println!("Current epoch: {}", epoch_info.epoch_number);
            println!("Chain ID: {}", epoch_info.chain_id.as_str());
            println!("Start time: {}", epoch_info.start_time);
            
            assert_eq!(epoch_info.chain_id.as_str(), "ethereum-holesky");
            assert!(epoch_info.epoch_number > 0, "Epoch should be greater than 0");
        }
        Err(e) => {
            panic!("Failed to connect to Holesky beacon node: {:?}\nEndpoint: {}\nTry setting HOLESKY_BEACON_ENDPOINT environment variable", e, endpoint);
        }
    }
}

#[tokio::test]
#[ignore] // Requires network access AND local beacon node (public endpoints timeout on full validator queries)
async fn test_holesky_stake_distribution() {
    // NOTE: This test requires a local or private Holesky beacon node.
    // Public endpoints (like publicnode.com) will timeout (504) when querying
    // all validators (~1.5M validators on Holesky).
    //
    // To run this test:
    // 1. Run a local beacon node: lighthouse beacon_node --network holesky --http
    // 2. Set: export HOLESKY_BEACON_ENDPOINT=http://localhost:5052
    // 3. Run: cargo test test_holesky_stake_distribution -- --ignored
    
    let endpoint = get_holesky_endpoint();
    
    // Skip if using public endpoint
    if endpoint.contains("publicnode") || endpoint.contains("stakely") {
        println!("[WARN] Skipping full validator query test with public endpoint");
        println!("   Public endpoints timeout on queries for all validators");
        println!("   Use a local beacon node to run this test");
        return;
    }
    
    println!("Testing stake distribution from: {}", endpoint);
    
    let beacon_client = BeaconClient::new(&endpoint);
    let observer = EthereumChainObserver::new(beacon_client, "holesky");

    // Get current epoch first
    let epoch_info = observer.get_current_epoch().await
        .expect("Failed to get current epoch");
    
    println!("Querying stake distribution for epoch: {}", epoch_info.epoch_number);

    // Get stake distribution
    let stake_dist_result = observer.get_stake_distribution(epoch_info.epoch_number).await;
    
    match stake_dist_result {
        Ok(stake_dist) => {
            println!("[PASS] Successfully retrieved stake distribution!");
            println!("Number of validators: {}", stake_dist.validators.len());
            println!("Total stake: {} Gwei", stake_dist.total_stake);
            
            assert!(stake_dist.validators.len() > 0, "Should have validators");
            assert!(stake_dist.total_stake > 0, "Total stake should be positive");
            assert_eq!(stake_dist.epoch, epoch_info.epoch_number);
            
            // Print first few validators
            println!("\nFirst 5 validators:");
            for (idx, (validator_id, stake)) in stake_dist.validators.iter().take(5).enumerate() {
                println!("  {}. {} -> {} Gwei", idx + 1, validator_id.as_str(), stake);
            }
        }
        Err(e) => {
            panic!("Failed to get stake distribution: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_holesky_state_commitment() {
    let endpoint = get_holesky_endpoint();
    println!("Testing state commitment from: {}", endpoint);
    
    let beacon_client = BeaconClient::new(&endpoint);
    let observer = EthereumChainObserver::new(beacon_client, "holesky");

    // Get current epoch
    let epoch_info = observer.get_current_epoch().await
        .expect("Failed to get current epoch");
    
    println!("Computing state commitment for epoch: {}", epoch_info.epoch_number);

    // Compute state commitment
    let commitment_result = observer.compute_state_commitment(epoch_info.epoch_number).await;
    
    match commitment_result {
        Ok(commitment) => {
            println!("[PASS] Successfully computed state commitment!");
            println!("Chain ID: {}", commitment.chain_id.as_str());
            println!("Epoch: {}", commitment.epoch);
            println!("Block number: {}", commitment.block_number);
            println!("Commitment type: {:?}", commitment.commitment_type);
            println!("Commitment value (hex): 0x{}", hex::encode(&commitment.value));
            
            assert_eq!(commitment.chain_id.as_str(), "ethereum-holesky");
            assert_eq!(commitment.epoch, epoch_info.epoch_number);
            assert!(commitment.value.len() == 32, "State root should be 32 bytes");
            
            // Print metadata
            if !commitment.metadata.is_empty() {
                println!("\nMetadata:");
                for (key, value) in &commitment.metadata {
                    println!("  {}: {}", key, value);
                }
            }
        }
        Err(e) => {
            panic!("Failed to compute state commitment: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_holesky_validator_activity() {
    let endpoint = get_holesky_endpoint();
    println!("Testing validator activity checks from: {}", endpoint);
    
    let beacon_client = BeaconClient::new(&endpoint);
    let observer = EthereumChainObserver::new(beacon_client, "holesky");

    // Get stake distribution to find a validator
    let epoch_info = observer.get_current_epoch().await
        .expect("Failed to get current epoch");
    
    let stake_dist = observer.get_stake_distribution(epoch_info.epoch_number).await
        .expect("Failed to get stake distribution");
    
    if let Some((validator_id, stake)) = stake_dist.validators.iter().next() {
        println!("Checking activity for validator: {}", validator_id.as_str());
        println!("Validator stake: {} Gwei", stake);
        
        let is_active = observer.is_validator_active(validator_id, epoch_info.epoch_number).await
            .expect("Failed to check validator activity");
        
        println!("[INFO] Validator active: {}", is_active);
        
        // If validator has stake, it should be considered active
        if *stake > 0 {
            assert!(is_active, "Validator with stake should be active");
        }
    } else {
        println!("[WARN] No validators found in stake distribution");
    }
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_holesky_end_to_end() {
    let endpoint = get_holesky_endpoint();
    println!("\n========================================");
    println!("End-to-End Holesky Integration Test");
    println!("========================================\n");
    println!("Endpoint: {}", endpoint);
    
    let beacon_client = BeaconClient::new(&endpoint);
    let observer = EthereumChainObserver::new(beacon_client, "holesky")
        .with_certification_interval(100);

    // Step 1: Get current epoch
    println!("\n1. Getting current epoch...");
    let epoch_info = observer.get_current_epoch().await
        .expect("Failed to get current epoch");
    println!("   ✓ Current epoch: {}", epoch_info.epoch_number);

    // Step 2: Compute state commitment (works with public endpoints)
    println!("\n2. Computing state commitment...");
    let commitment = observer.compute_state_commitment(epoch_info.epoch_number).await
        .expect("Failed to compute state commitment");
    println!("   ✓ State root: 0x{}", hex::encode(&commitment.value[..8]));
    println!("   ✓ Block number: {}", commitment.block_number);

    // Step 3: Check if we can query stake distribution (skip for public endpoints)
    let using_public_endpoint = endpoint.contains("publicnode") || endpoint.contains("stakely");
    if !using_public_endpoint {
        println!("\n3. Getting stake distribution...");
        let stake_dist = observer.get_stake_distribution(epoch_info.epoch_number).await
            .expect("Failed to get stake distribution");
        println!("   ✓ Validators: {}", stake_dist.validators.len());
        println!("   ✓ Total stake: {} Gwei", stake_dist.total_stake);

        // Step 4: Verify a validator
        if let Some((validator_id, _)) = stake_dist.validators.iter().next() {
            println!("\n4. Checking validator activity...");
            let is_active = observer.is_validator_active(validator_id, epoch_info.epoch_number).await
                .expect("Failed to check validator");
            println!("   ✓ Validator active: {}", is_active);
        }
    } else {
        println!("\n3. Skipping stake distribution (public endpoint limitation)");
        println!("   [WARN] Public endpoints timeout on full validator queries");
        println!("   ✓ But state root queries work fine!");
    }

    println!("\n========================================");
    println!("[PASS] All checks passed!");
    println!("========================================\n");
}

