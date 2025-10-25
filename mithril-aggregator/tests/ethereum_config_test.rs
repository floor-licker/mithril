/// Integration test for Ethereum configuration loading
/// 
/// This test validates that:
/// 1. The ethereum-holesky.json config file can be loaded
/// 2. All Ethereum-specific fields are parsed correctly
/// 3. The configuration properly enables the Ethereum observer

#[test]
fn test_ethereum_holesky_config_loads() {
    // GIVEN: The ethereum-holesky.json configuration file
    let config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("config")
        .join("ethereum-holesky.json");

    assert!(
        config_path.exists(),
        "Ethereum config file should exist at {:?}",
        config_path
    );

    // WHEN: Loading the configuration
    let config_result = config::Config::builder()
        .add_source(config::File::with_name(
            &config_path.to_string_lossy().to_string(),
        ))
        .build();

    // THEN: Configuration should load successfully
    assert!(
        config_result.is_ok(),
        "Configuration should parse successfully: {:?}",
        config_result.err()
    );

    let config = config_result.unwrap();

    // THEN: Ethereum observer should be enabled
    let enable_ethereum = config
        .get_bool("enable_ethereum_observer")
        .expect("enable_ethereum_observer should be present");
    assert!(
        enable_ethereum,
        "Ethereum observer should be enabled in ethereum-holesky.json"
    );

    // THEN: Ethereum endpoint should be configured
    let ethereum_endpoint = config
        .get_string("ethereum_beacon_endpoint")
        .expect("ethereum_beacon_endpoint should be present");
    assert!(
        !ethereum_endpoint.is_empty(),
        "Ethereum beacon endpoint should not be empty"
    );
    assert!(
        ethereum_endpoint.starts_with("https://"),
        "Ethereum endpoint should use HTTPS: {}",
        ethereum_endpoint
    );

    // THEN: Ethereum network should be configured
    let ethereum_network = config
        .get_string("ethereum_network")
        .expect("ethereum_network should be present");
    assert_eq!(
        ethereum_network, "holesky",
        "Network should be holesky in ethereum-holesky.json"
    );

    // THEN: Signed entity types should include EthereumStateRoot
    let signed_entity_types = config
        .get_string("signed_entity_types")
        .expect("signed_entity_types should be present");
    assert!(
        signed_entity_types.contains("EthereumStateRoot"),
        "signed_entity_types should include EthereumStateRoot: {}",
        signed_entity_types
    );
}

#[test]
fn test_ethereum_config_fields_are_valid() {
    // GIVEN: The ethereum-holesky.json configuration file
    let config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("config")
        .join("ethereum-holesky.json");

    let config = config::Config::builder()
        .add_source(config::File::with_name(
            &config_path.to_string_lossy().to_string(),
        ))
        .build()
        .expect("Config should load");

    // THEN: Certification interval should be reasonable (1-100 epochs)
    if let Ok(interval) = config.get_int("ethereum_certification_interval_epochs") {
        assert!(
            interval > 0 && interval <= 100,
            "Certification interval should be between 1-100 epochs: {}",
            interval
        );
    }

    // THEN: Network should be one of the known networks
    let network = config.get_string("ethereum_network").unwrap();
    assert!(
        matches!(network.as_str(), "mainnet" | "holesky" | "sepolia"),
        "Network should be mainnet, holesky, or sepolia: {}",
        network
    );
}

#[test]
fn test_cardano_and_ethereum_can_coexist() {
    // GIVEN: The ethereum-holesky.json configuration (which has both chains)
    let config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("config")
        .join("ethereum-holesky.json");

    let config = config::Config::builder()
        .add_source(config::File::with_name(
            &config_path.to_string_lossy().to_string(),
        ))
        .build()
        .expect("Config should load");

    // THEN: Cardano configuration should still be present
    assert!(
        config.get_string("network").is_ok(),
        "Cardano network should be configured"
    );
    assert!(
        config.get_string("cardano_cli_path").is_ok(),
        "Cardano CLI path should be configured"
    );
    assert!(
        config.get_string("cardano_node_socket_path").is_ok(),
        "Cardano node socket should be configured"
    );

    // THEN: Ethereum configuration should also be present
    assert!(
        config.get_bool("enable_ethereum_observer").unwrap(),
        "Ethereum observer should be enabled"
    );
    assert!(
        config.get_string("ethereum_network").is_ok(),
        "Ethereum network should be configured"
    );
    assert!(
        config.get_string("ethereum_beacon_endpoint").is_ok(),
        "Ethereum beacon endpoint should be configured"
    );
}

