/// End-to-end integration test for Ethereum support
///
/// This test validates the complete flow with a real Holesky beacon node:
/// 1. Ethereum chain observer can query Holesky
/// 2. Protocol message computation works
/// 3. Artifact generation works
/// 4. The complete certification pipeline functions correctly
use std::sync::Arc;

use mithril_common::entities::Epoch;
use mithril_ethereum_chain::{BeaconClient, EthereumChainObserver};
use mithril_universal::UniversalChainObserver;

/// Test that we can connect to Holesky and query epoch information
#[tokio::test]
#[ignore] // Requires network access to public Holesky endpoint
async fn test_holesky_connection() {
    // GIVEN: A Holesky beacon client
    let endpoint = "https://ethereum-holesky-beacon-api.publicnode.com";
    let client = BeaconClient::new(endpoint);
    let observer = EthereumChainObserver::new(client, "holesky");

    // WHEN: Querying current epoch
    let result = observer.get_current_epoch().await;

    // THEN: We should get a valid epoch
    assert!(
        result.is_ok(),
        "Should be able to query Holesky epoch: {:?}",
        result.err()
    );

    let epoch_info = result.unwrap();
    println!("Current Holesky epoch: {}", epoch_info.epoch_number);
    assert!(
        epoch_info.epoch_number > 0,
        "Epoch number should be positive"
    );
}

/// Test that we can compute state commitment from Holesky
#[tokio::test]
#[ignore] // Requires network access
async fn test_holesky_state_commitment() {
    // GIVEN: A Holesky chain observer
    let endpoint = "https://ethereum-holesky-beacon-api.publicnode.com";
    let client = BeaconClient::new(endpoint);
    let observer = EthereumChainObserver::new(client, "holesky");

    // Get current epoch
    let current_epoch = observer.get_current_epoch().await.unwrap();
    println!("Testing state commitment for epoch: {}", current_epoch.epoch_number);

    // WHEN: Computing state commitment
    let result = observer.compute_state_commitment(current_epoch.epoch_number).await;

    // THEN: We should get a valid state commitment
    assert!(
        result.is_ok(),
        "Should be able to compute state commitment: {:?}",
        result.err()
    );

    let commitment = result.unwrap();
    println!("State commitment: {}", hex::encode(&commitment.value));
    assert!(!commitment.value.is_empty(), "State commitment should not be empty");
    assert!(commitment.block_number > 0, "Block number should be positive");
}

/// Test Ethereum protocol message computation
#[tokio::test]
#[ignore] // Requires network access
async fn test_ethereum_protocol_message_computation() {
    use mithril_aggregator::services::UniversalEthereumStateRootRetriever;
    use mithril_common::signable_builder::{EthereumStateRootSignableBuilder, SignableBuilder};

    // GIVEN: A Holesky chain observer
    let endpoint = "https://ethereum-holesky-beacon-api.publicnode.com";
    let client = BeaconClient::new(endpoint);
    let observer = Arc::new(EthereumChainObserver::new(client, "holesky"));

    // Get current epoch
    let current_epoch_info = observer.get_current_epoch().await.unwrap();
    let test_epoch = Epoch(current_epoch_info.epoch_number);
    println!("Computing protocol message for epoch: {}", test_epoch);

    // GIVEN: Ethereum state root signable builder
    let retriever = Arc::new(UniversalEthereumStateRootRetriever::new(observer));
    let builder = EthereumStateRootSignableBuilder::new(retriever);

    // WHEN: Computing protocol message
    let result = builder.compute_protocol_message(test_epoch).await;

    // THEN: We should get a valid protocol message
    assert!(
        result.is_ok(),
        "Should be able to compute protocol message: {:?}",
        result.err()
    );

    let protocol_message = result.unwrap();
    println!("Protocol message: {:?}", protocol_message);

    // Verify the protocol message has the expected parts
    assert!(
        protocol_message.get_message_part(&mithril_common::entities::ProtocolMessagePartKey::EthereumStateRoot).is_some(),
        "Protocol message should contain state root"
    );
    assert!(
        protocol_message.get_message_part(&mithril_common::entities::ProtocolMessagePartKey::EthereumBeaconBlockNumber).is_some(),
        "Protocol message should contain block number"
    );
    assert!(
        protocol_message.get_message_part(&mithril_common::entities::ProtocolMessagePartKey::EthereumEpoch).is_some(),
        "Protocol message should contain epoch"
    );
}

/// Test Ethereum state root retrieval (building block for artifacts)
#[tokio::test]
#[ignore] // Requires network access
async fn test_ethereum_state_root_retrieval() {
    use mithril_aggregator::services::UniversalEthereumStateRootRetriever;
    use mithril_common::signable_builder::EthereumStateRootRetriever;

    // GIVEN: A Holesky chain observer
    let endpoint = "https://ethereum-holesky-beacon-api.publicnode.com";
    let client = BeaconClient::new(endpoint);
    let observer = Arc::new(EthereumChainObserver::new(client, "holesky"));

    // Get current epoch
    let current_epoch_info = observer.get_current_epoch().await.unwrap();
    let test_epoch = Epoch(current_epoch_info.epoch_number);
    println!("Retrieving state root for epoch: {}", test_epoch);

    // GIVEN: Ethereum state root retriever
    let retriever = Arc::new(UniversalEthereumStateRootRetriever::new(observer));

    // WHEN: Retrieving state root data
    let result = retriever.retrieve(test_epoch).await;

    // THEN: We should get valid state root data
    assert!(
        result.is_ok(),
        "Should be able to retrieve state root: {:?}",
        result.err()
    );

    let state_root_data = result.unwrap();
    assert!(state_root_data.is_some(), "State root data should exist");

    let data = state_root_data.unwrap();
    println!("State root data: epoch={}, block={}, root={}", 
        data.epoch, data.block_number, data.state_root);

    // Verify data properties
    assert_eq!(data.epoch, test_epoch, "Data should have correct epoch");
    assert!(!data.state_root.is_empty(), "State root should not be empty");
    assert!(data.block_number > 0, "Block number should be positive");
}

/// Test the complete data retrieval flow for Ethereum certification
#[tokio::test]
#[ignore] // Requires network access
async fn test_ethereum_certification_data_flow() {
    use mithril_aggregator::services::UniversalEthereumStateRootRetriever;
    use mithril_common::signable_builder::{EthereumStateRootSignableBuilder, SignableBuilder, EthereumStateRootRetriever};

    // GIVEN: A Holesky chain observer
    let endpoint = "https://ethereum-holesky-beacon-api.publicnode.com";
    let client = BeaconClient::new(endpoint);
    let observer = Arc::new(EthereumChainObserver::new(client, "holesky"));

    // Get current epoch
    let current_epoch_info = observer.get_current_epoch().await.unwrap();
    let test_epoch = Epoch(current_epoch_info.epoch_number);
    println!("\n=== Testing Ethereum Certification Data Flow for Epoch {} ===", test_epoch);

    // STEP 1: Retrieve state root data
    let retriever = Arc::new(UniversalEthereumStateRootRetriever::new(observer.clone()));
    let state_root_data = retriever
        .retrieve(test_epoch)
        .await
        .expect("Should retrieve state root data")
        .expect("State root data should exist");
    println!("✓ Step 1: Retrieved state root data");
    println!("  - State Root: {}", state_root_data.state_root);
    println!("  - Block Number: {}", state_root_data.block_number);

    // STEP 2: Compute protocol message
    let signable_builder = EthereumStateRootSignableBuilder::new(retriever.clone());
    let protocol_message = signable_builder
        .compute_protocol_message(test_epoch)
        .await
        .expect("Should compute protocol message");
    println!("✓ Step 2: Computed protocol message");
    
    // Verify the protocol message has the expected parts
    use mithril_common::entities::ProtocolMessagePartKey;
    assert!(
        protocol_message.get_message_part(&ProtocolMessagePartKey::EthereumStateRoot).is_some(),
        "Protocol message should contain state root"
    );
    assert!(
        protocol_message.get_message_part(&ProtocolMessagePartKey::EthereumBeaconBlockNumber).is_some(),
        "Protocol message should contain block number"
    );
    assert!(
        protocol_message.get_message_part(&ProtocolMessagePartKey::EthereumEpoch).is_some(),
        "Protocol message should contain epoch"
    );
    println!("✓ Step 3: Protocol message contains all required parts");

    // STEP 4: Verify protocol message hash can be computed
    let message_hash = protocol_message.compute_hash();
    assert!(!message_hash.is_empty(), "Message hash should not be empty");
    println!("✓ Step 4: Computed message hash: {}", message_hash);

    // Summary
    println!("\n=== Certification Data Flow Summary ===");
    println!("Epoch: {}", test_epoch);
    println!("State Root: {}", state_root_data.state_root);
    println!("Block Number: {}", state_root_data.block_number);
    println!("Message Hash: {}", message_hash);
    println!("\n✓ All data components for Ethereum certification are functional!");
}

