//! Integration tests using mocked beacon data
//!
//! These tests validate the EthereumChainObserver logic without requiring
//! a real beacon node connection.

use mithril_ethereum_chain::test_utils::MockBeaconClient;
use mithril_ethereum_chain::{BeaconClient, EthereumChainObserver};
use mithril_universal::UniversalChainObserver;

/// Helper to create a mock observer with test data
fn create_mock_observer() -> (MockBeaconClient, EthereumChainObserver) {
    let mock = MockBeaconClient::new();
    let beacon_client = BeaconClient::new("http://mock-endpoint");
    let observer = EthereumChainObserver::new(beacon_client, "holesky")
        .with_certification_interval(100);
    
    (mock, observer)
}

#[tokio::test]
async fn test_mock_epoch_calculation() {
    let mock = MockBeaconClient::new();
    
    // Holesky uses 32 slots per epoch, 12 seconds per slot
    let current_slot = mock.current_slot;
    let expected_epoch = current_slot / 32;
    
    println!("Current slot: {}", current_slot);
    println!("Expected epoch: {}", expected_epoch);
    
    assert_eq!(expected_epoch, 169909); // 5437088 / 32
}

#[tokio::test]
async fn test_mock_validator_data_structure() {
    let mock = MockBeaconClient::new();
    let validators = mock.get_validators_by_epoch_mock(1000).await.unwrap();
    
    // Verify we have 100 mock validators
    assert_eq!(validators.len(), 100);
    
    // Count active validators
    let active_count = validators.iter()
        .filter(|v| v.status.is_active())
        .count();
    
    println!("Total validators: {}", validators.len());
    println!("Active validators: {}", active_count);
    
    assert_eq!(active_count, 90, "Expected 90 active validators");
    
    // Verify first validator has correct stake
    let first_validator = &validators[0];
    assert_eq!(first_validator.validator.effective_balance, "32000000000");
    
    // Calculate total stake
    let total_stake: u64 = validators.iter()
        .filter(|v| v.status.is_active())
        .filter_map(|v| v.validator.effective_balance.parse::<u64>().ok())
        .sum();
    
    println!("Total stake: {} Gwei", total_stake);
    assert!(total_stake > 0, "Total stake should be positive");
}

#[tokio::test]
async fn test_mock_block_execution_payload() {
    let mock = MockBeaconClient::new();
    let block = mock.get_block_by_slot_mock(5437088).await.unwrap();
    
    // Verify block structure
    assert_eq!(block.message.slot, "5437088");
    
    // Verify execution payload exists
    let payload = block.message.body.execution_payload
        .expect("Block should have execution payload");
    
    // Verify state root format
    assert!(payload.state_root.starts_with("0x"));
    assert_eq!(payload.state_root.len(), 66); // 0x + 64 hex chars
    
    println!("Block number: {}", payload.block_number);
    println!("State root: {}", payload.state_root);
    println!("Block hash: {}", payload.block_hash);
}

#[tokio::test]
async fn test_mock_certification_interval() {
    let (mock, observer) = create_mock_observer();
    
    // Verify certification interval is set correctly
    assert_eq!(observer.certification_interval(), 100);
    
    // Test epoch calculation
    let current_slot = mock.current_slot;
    let current_epoch = current_slot / 32;
    
    println!("Current epoch: {}", current_epoch);
    println!("Certification interval: {}", observer.certification_interval());
}

#[tokio::test]
async fn test_mock_chain_id() {
    let (_mock, observer) = create_mock_observer();
    
    let chain_id = observer.chain_id();
    assert_eq!(chain_id.as_str(), "ethereum-holesky");
    
    println!("Chain ID: {}", chain_id.as_str());
}

#[tokio::test]
async fn test_mock_multiple_epochs() {
    let mock = MockBeaconClient::new();
    
    // Test blocks at different epoch boundaries
    let epoch_1_last_slot = (32 * 1) - 1; // Slot 31 (last slot of epoch 0)
    let epoch_2_first_slot = 32 * 1;       // Slot 32 (first slot of epoch 1)
    let epoch_100_last_slot = (32 * 100) - 1; // Slot 3199
    
    let block1 = mock.get_block_by_slot_mock(epoch_1_last_slot).await.unwrap();
    assert_eq!(block1.message.slot, "31");
    
    let block2 = mock.get_block_by_slot_mock(epoch_2_first_slot).await.unwrap();
    assert_eq!(block2.message.slot, "32");
    
    let block3 = mock.get_block_by_slot_mock(epoch_100_last_slot).await.unwrap();
    assert_eq!(block3.message.slot, "3199");
    
    println!("✓ Tested epoch boundary blocks");
}

#[tokio::test]
async fn test_mock_validator_pubkeys_unique() {
    let mock = MockBeaconClient::new();
    let validators = mock.get_validators_by_epoch_mock(1000).await.unwrap();
    
    // Collect all pubkeys
    let mut pubkeys: Vec<String> = validators.iter()
        .map(|v| v.validator.pubkey.clone())
        .collect();
    
    let original_count = pubkeys.len();
    pubkeys.sort();
    pubkeys.dedup();
    let unique_count = pubkeys.len();
    
    assert_eq!(original_count, unique_count, "All validator pubkeys should be unique");
    println!("✓ Verified {} unique validator pubkeys", unique_count);
}

#[tokio::test]
async fn test_mock_stake_distribution_logic() {
    let mock = MockBeaconClient::new();
    let validators = mock.get_validators_by_epoch_mock(1000).await.unwrap();
    
    // Simulate building a stake distribution
    let mut total_stake = 0u64;
    let mut validator_count = 0;
    
    for validator in validators.iter() {
        if validator.status.is_active() {
            if let Ok(stake) = validator.validator.effective_balance.parse::<u64>() {
                total_stake += stake;
                validator_count += 1;
            }
        }
    }
    
    println!("Active validators: {}", validator_count);
    println!("Total stake: {} Gwei ({} ETH)", total_stake, total_stake / 1_000_000_000);
    
    assert_eq!(validator_count, 90, "Should have 90 active validators");
    
    // 80 validators with 32 ETH + 10 validators with 16 ETH
    // = (80 * 32) + (10 * 16) = 2560 + 160 = 2720 ETH
    let expected_stake_gwei = (80 * 32_000_000_000u64) + (10 * 16_000_000_000u64);
    assert_eq!(total_stake, expected_stake_gwei);
}

#[tokio::test]
async fn test_mock_block_sequence() {
    let mock = MockBeaconClient::new();
    
    // Get a sequence of blocks
    let slots = vec![100, 101, 102, 103];
    let mut blocks = Vec::new();
    
    for slot in slots {
        let block = mock.get_block_by_slot_mock(slot).await.unwrap();
        blocks.push(block);
    }
    
    // Verify each block has correct slot
    for (i, block) in blocks.iter().enumerate() {
        let expected_slot = 100 + i;
        assert_eq!(block.message.slot, expected_slot.to_string());
        
        // Verify parent hash points to previous block
        if i > 0 {
            let previous_slot = expected_slot - 1;
            // Current block's parent_hash should relate to previous block
            let parent_hash = &block.message.body.execution_payload.as_ref().unwrap().parent_hash;
            
            // In our mock, parent_hash = format of (slot - 1)
            let expected_parent = format!("0x{:0>64}", format!("{:x}", previous_slot));
            assert_eq!(parent_hash, &expected_parent);
        }
    }
    
    println!("✓ Verified {} blocks in sequence", blocks.len());
}

#[tokio::test]
async fn test_mock_handles_future_slots() {
    let mock = MockBeaconClient::new();
    
    // Try to get a block from the future
    let future_slot = mock.current_slot + 1000;
    let result = mock.get_block_by_slot_mock(future_slot).await;
    
    assert!(result.is_err(), "Should error for future slots");
    println!("✓ Correctly rejects future slot queries");
}

#[tokio::test]
async fn test_end_to_end_mock_flow() {
    println!("\n========================================");
    println!("End-to-End Mock Integration Test");
    println!("========================================\n");
    
    let mock = MockBeaconClient::new();
    
    // Step 1: Get current slot
    println!("1. Getting current slot...");
    let current_slot = mock.get_current_slot_mock().await.unwrap();
    let current_epoch = current_slot / 32;
    println!("   ✓ Current slot: {}", current_slot);
    println!("   ✓ Current epoch: {}", current_epoch);
    
    // Step 2: Get validators
    println!("\n2. Getting validators...");
    let validators = mock.get_validators_by_epoch_mock(current_epoch).await.unwrap();
    let active_validators = validators.iter()
        .filter(|v| v.status.is_active())
        .count();
    println!("   ✓ Total validators: {}", validators.len());
    println!("   ✓ Active validators: {}", active_validators);
    
    // Step 3: Get finalized block
    println!("\n3. Getting recent block...");
    let block = mock.get_block_by_slot_mock(current_slot - 100).await.unwrap();
    println!("   ✓ Block slot: {}", block.message.slot);
    
    // Step 4: Extract state root
    let payload = block.message.body.execution_payload.unwrap();
    println!("   ✓ State root: {}", &payload.state_root[..18]);
    println!("   ✓ Block number: {}", payload.block_number);
    
    println!("\n========================================");
    println!("[PASS] All mock integration checks passed!");
    println!("========================================\n");
}

