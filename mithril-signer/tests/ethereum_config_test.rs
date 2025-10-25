use config::Config;
use mithril_signer::Configuration;
use mithril_signer::configuration::ChainType;
use std::path::PathBuf;

#[test]
fn test_ethereum_config_loading() {
    // Create a raw config with Ethereum settings
    let raw_config = Config::builder()
        .set_default("chain_type", "ethereum").unwrap()
        .set_default("beacon_endpoint", "http://localhost:5052").unwrap()
        .set_default("network", "holesky").unwrap()
        .set_default("validator_pubkey", "0x1234567890abcdef").unwrap()
        .set_default("validator_seckey_path", "/keys/validator.key").unwrap()
        .set_default("certification_interval_epochs", 675).unwrap()
        // Set required Cardano fields with defaults (for backward compatibility)
        .set_default("aggregator_endpoint", "http://localhost:8000").unwrap()
        .set_default("db_directory", "/tmp/db").unwrap()
        .set_default("data_stores_directory", "/tmp/stores").unwrap()
        .set_default("network_security_parameter", 2160).unwrap()
        .set_default("preload_security_parameter", 3000).unwrap()
        .set_default("run_interval", 10000).unwrap()
        .set_default("cardano_cli_path", "/usr/bin/cardano-cli").unwrap()
        .set_default("cardano_node_socket_path", "/tmp/node.socket").unwrap()
        .set_default("network", "holesky").unwrap()
        .set_default("era_reader_adapter_type", "bootstrap").unwrap()
        .set_default("enable_metrics_server", false).unwrap()
        .set_default("metrics_server_ip", "0.0.0.0").unwrap()
        .set_default("metrics_server_port", 9090).unwrap()
        .set_default("disable_digests_cache", false).unwrap()
        .set_default("reset_digests_cache", false).unwrap()
        .set_default("allow_unparsable_block", false).unwrap()
        .set_default("enable_transaction_pruning", true).unwrap()
        .set_default("transactions_import_block_chunk_size", 1000).unwrap()
        .set_default("preloading_refresh_interval_in_seconds", 60).unwrap()
        .set_default("cardano_transactions_block_streamer_max_roll_forwards_per_poll", 10000).unwrap()
        .set_default("signature_publisher_config.retry_attempts", 1).unwrap()
        .set_default("signature_publisher_config.retry_delay_ms", 1000).unwrap()
        .set_default("signature_publisher_config.delayer_delay_ms", 1000).unwrap()
        .set_default("signature_publisher_config.skip_delayer", false).unwrap()
        .build()
        .expect("Should build config");

    let mut config: Configuration = raw_config
        .clone()
        .try_deserialize()
        .expect("Should deserialize");

    // Populate chain-specific configuration
    config
        .populate_chain_config(&raw_config)
        .expect("Should populate chain config");

    // Verify the chain type
    assert_eq!(config.chain_type, ChainType::Ethereum);

    // Verify Ethereum configuration was populated
    assert!(config.ethereum_config.is_some());
    
    let eth_config = config.ethereum_config.unwrap();
    assert_eq!(eth_config.beacon_endpoint, "http://localhost:5052");
    assert_eq!(eth_config.network, "holesky");
    assert_eq!(eth_config.validator_pubkey, "0x1234567890abcdef");
    assert_eq!(eth_config.validator_seckey_path, PathBuf::from("/keys/validator.key"));
    assert_eq!(eth_config.certification_interval_epochs, 675);
}

#[test]
fn test_ethereum_config_missing_required_field() {
    // Create a config without required fields
    let raw_config = Config::builder()
        .set_default("chain_type", "ethereum").unwrap()
        // Missing beacon_endpoint, network, etc.
        .set_default("aggregator_endpoint", "http://localhost:8000").unwrap()
        .set_default("db_directory", "/tmp/db").unwrap()
        .set_default("data_stores_directory", "/tmp/stores").unwrap()
        .set_default("network_security_parameter", 2160).unwrap()
        .set_default("preload_security_parameter", 3000).unwrap()
        .set_default("run_interval", 10000).unwrap()
        .set_default("cardano_cli_path", "/usr/bin/cardano-cli").unwrap()
        .set_default("cardano_node_socket_path", "/tmp/node.socket").unwrap()
        .set_default("network", "mainnet").unwrap()
        .set_default("era_reader_adapter_type", "bootstrap").unwrap()
        .set_default("enable_metrics_server", false).unwrap()
        .set_default("metrics_server_ip", "0.0.0.0").unwrap()
        .set_default("metrics_server_port", 9090).unwrap()
        .set_default("disable_digests_cache", false).unwrap()
        .set_default("reset_digests_cache", false).unwrap()
        .set_default("allow_unparsable_block", false).unwrap()
        .set_default("enable_transaction_pruning", true).unwrap()
        .set_default("transactions_import_block_chunk_size", 1000).unwrap()
        .set_default("preloading_refresh_interval_in_seconds", 60).unwrap()
        .set_default("cardano_transactions_block_streamer_max_roll_forwards_per_poll", 10000).unwrap()
        .set_default("signature_publisher_config.retry_attempts", 1).unwrap()
        .set_default("signature_publisher_config.retry_delay_ms", 1000).unwrap()
        .set_default("signature_publisher_config.delayer_delay_ms", 1000).unwrap()
        .set_default("signature_publisher_config.skip_delayer", false).unwrap()
        .build()
        .expect("Should build config");

    let mut config: Configuration = raw_config
        .clone()
        .try_deserialize()
        .expect("Should deserialize");

    // Populate chain-specific configuration (should fail)
    let result = config.populate_chain_config(&raw_config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("beacon_endpoint"));
}

#[test]
fn test_cardano_config_backward_compatibility() {
    // Create a Cardano config (no explicit chain_type, defaults to Cardano)
    let raw_config = Config::builder()
        // No chain_type set, should default to Cardano
        .set_default("aggregator_endpoint", "http://localhost:8000").unwrap()
        .set_default("db_directory", "/tmp/db").unwrap()
        .set_default("data_stores_directory", "/tmp/stores").unwrap()
        .set_default("network_security_parameter", 2160).unwrap()
        .set_default("preload_security_parameter", 3000).unwrap()
        .set_default("run_interval", 10000).unwrap()
        .set_default("cardano_cli_path", "/usr/bin/cardano-cli").unwrap()
        .set_default("cardano_node_socket_path", "/tmp/node.socket").unwrap()
        .set_default("network", "mainnet").unwrap()
        .set_default("era_reader_adapter_type", "bootstrap").unwrap()
        .set_default("enable_metrics_server", false).unwrap()
        .set_default("metrics_server_ip", "0.0.0.0").unwrap()
        .set_default("metrics_server_port", 9090).unwrap()
        .set_default("disable_digests_cache", false).unwrap()
        .set_default("reset_digests_cache", false).unwrap()
        .set_default("allow_unparsable_block", false).unwrap()
        .set_default("enable_transaction_pruning", true).unwrap()
        .set_default("transactions_import_block_chunk_size", 1000).unwrap()
        .set_default("preloading_refresh_interval_in_seconds", 60).unwrap()
        .set_default("cardano_transactions_block_streamer_max_roll_forwards_per_poll", 10000).unwrap()
        .set_default("signature_publisher_config.retry_attempts", 1).unwrap()
        .set_default("signature_publisher_config.retry_delay_ms", 1000).unwrap()
        .set_default("signature_publisher_config.delayer_delay_ms", 1000).unwrap()
        .set_default("signature_publisher_config.skip_delayer", false).unwrap()
        .build()
        .expect("Should build config");

    let mut config: Configuration = raw_config
        .clone()
        .try_deserialize()
        .expect("Should deserialize");

    config
        .populate_chain_config(&raw_config)
        .expect("Should populate chain config");

    // Verify the chain type defaults to Cardano
    assert_eq!(config.chain_type, ChainType::Cardano);

    // Verify Ethereum configuration was NOT populated
    assert!(config.ethereum_config.is_none());
}

