/// Integration test for aggregator startup with Ethereum support
///
/// This test validates that:
/// 1. The aggregator can initialize with Ethereum observer enabled
/// 2. The Ethereum configuration is validated correctly
/// 3. Required configuration fields are enforced
mod test_extensions;

use mithril_aggregator::ServeCommandConfiguration;
use mithril_common::{
    entities::{BlockNumber, CardanoTransactionsSigningConfig, ChainPoint, Epoch, SignedEntityTypeDiscriminants, SlotNumber, TimePoint},
    temp_dir,
};
use test_extensions::RuntimeTester;

/// Test that the aggregator can build with Ethereum observer enabled
#[tokio::test]
async fn test_aggregator_builds_with_ethereum_observer() {
    // GIVEN: Configuration with Ethereum enabled
    let configuration = ServeCommandConfiguration {
        signed_entity_types: Some(
            [
                SignedEntityTypeDiscriminants::CardanoImmutableFilesFull.to_string(),
                SignedEntityTypeDiscriminants::EthereumStateRoot.to_string(),
            ]
            .join(","),
        ),
        // Enable Ethereum observer
        enable_ethereum_observer: true,
        ethereum_beacon_endpoint: Some("https://ethereum-holesky-beacon-api.publicnode.com".to_string()),
        ethereum_network: Some("holesky".to_string()),
        ethereum_certification_interval_epochs: Some(10),
        cardano_transactions_signing_config: CardanoTransactionsSigningConfig {
            security_parameter: BlockNumber(0),
            step: BlockNumber(30),
        },
        ..ServeCommandConfiguration::new_sample(temp_dir!())
    };

    // WHEN: Building the runtime tester (which builds dependencies)
    let result = RuntimeTester::build(
        TimePoint {
            epoch: Epoch(1),
            immutable_file_number: 1,
            chain_point: ChainPoint {
                slot_number: SlotNumber(10),
                block_number: BlockNumber(100),
                block_hash: "block_hash-100".to_string(),
            },
        },
        configuration,
    )
    .await;

    // THEN: The runtime should build successfully
    // The successful creation of RuntimeTester proves:
    // 1. Configuration is valid
    // 2. Ethereum observer can be built
    // 3. All dependencies wire up correctly
    // If the RuntimeTester was created successfully, all dependencies are valid
    drop(result);
}

/// Test that Ethereum configuration requires beacon endpoint
#[tokio::test]
#[should_panic(expected = "ethereum_beacon_endpoint")]
async fn test_ethereum_observer_requires_endpoint() {
    // GIVEN: Configuration with Ethereum enabled but missing endpoint
    let configuration = ServeCommandConfiguration {
        signed_entity_types: Some(
            [
                SignedEntityTypeDiscriminants::CardanoImmutableFilesFull.to_string(),
                SignedEntityTypeDiscriminants::EthereumStateRoot.to_string(),
            ]
            .join(","),
        ),
        // Enable Ethereum but missing endpoint
        enable_ethereum_observer: true,
        ethereum_beacon_endpoint: None, // Missing!
        ethereum_network: Some("holesky".to_string()),
        cardano_transactions_signing_config: CardanoTransactionsSigningConfig {
            security_parameter: BlockNumber(0),
            step: BlockNumber(30),
        },
        ..ServeCommandConfiguration::new_sample(temp_dir!())
    };

    // WHEN: Building the runtime tester
    let result = RuntimeTester::build(
        TimePoint {
            epoch: Epoch(1),
            immutable_file_number: 1,
            chain_point: ChainPoint {
                slot_number: SlotNumber(10),
                block_number: BlockNumber(100),
                block_hash: "block_hash-100".to_string(),
            },
        },
        configuration,
    )
    .await;

    // THEN: The runtime should fail to build (this line won't be reached)
    // The test passes if the above build panics
    drop(result);
}

/// Test that unknown Ethereum networks are rejected
#[tokio::test]
#[should_panic(expected = "Unknown Ethereum network")]
async fn test_ethereum_observer_rejects_unknown_network() {
    // GIVEN: Configuration with invalid Ethereum network
    let configuration = ServeCommandConfiguration {
        signed_entity_types: Some(
            [
                SignedEntityTypeDiscriminants::CardanoImmutableFilesFull.to_string(),
                SignedEntityTypeDiscriminants::EthereumStateRoot.to_string(),
            ]
            .join(","),
        ),
        // Invalid network
        enable_ethereum_observer: true,
        ethereum_beacon_endpoint: Some("https://ethereum-holesky-beacon-api.publicnode.com".to_string()),
        ethereum_network: Some("invalid_network".to_string()),
        cardano_transactions_signing_config: CardanoTransactionsSigningConfig {
            security_parameter: BlockNumber(0),
            step: BlockNumber(30),
        },
        ..ServeCommandConfiguration::new_sample(temp_dir!())
    };

    // WHEN: Building the runtime tester
    let result = RuntimeTester::build(
        TimePoint {
            epoch: Epoch(1),
            immutable_file_number: 1,
            chain_point: ChainPoint {
                slot_number: SlotNumber(10),
                block_number: BlockNumber(100),
                block_hash: "block_hash-100".to_string(),
            },
        },
        configuration,
    )
    .await;

    // THEN: The runtime should fail to build (this line won't be reached)
    // The test passes if the above build panics
    drop(result);
}

/// Test that Ethereum observer can be disabled (default behavior)
#[tokio::test]
async fn test_aggregator_works_without_ethereum_observer() {
    // GIVEN: Configuration with Ethereum disabled (default)
    let configuration = ServeCommandConfiguration {
        signed_entity_types: Some(
            SignedEntityTypeDiscriminants::CardanoImmutableFilesFull.to_string(),
        ),
        // Ethereum disabled (default)
        enable_ethereum_observer: false,
        cardano_transactions_signing_config: CardanoTransactionsSigningConfig {
            security_parameter: BlockNumber(0),
            step: BlockNumber(30),
        },
        ..ServeCommandConfiguration::new_sample(temp_dir!())
    };

    // WHEN: Building the runtime tester
    let result = RuntimeTester::build(
        TimePoint {
            epoch: Epoch(1),
            immutable_file_number: 1,
            chain_point: ChainPoint {
                slot_number: SlotNumber(10),
                block_number: BlockNumber(100),
                block_hash: "block_hash-100".to_string(),
            },
        },
        configuration,
    )
    .await;

    // THEN: The runtime should build successfully (backward compatibility)
    // If the RuntimeTester was created successfully, Cardano-only mode works
    drop(result);
}
