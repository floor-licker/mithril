//! Integration tests for multi-chain database functionality
//!
//! These tests verify that the database correctly stores and retrieves
//! certificates with different chain types.

use std::sync::Arc;

use mithril_aggregator::database::record::CertificateRecord;
use mithril_aggregator::database::repository::CertificateRepository;
use mithril_aggregator::database::test_helper::main_db_connection;
use mithril_common::entities::{Certificate, Epoch, SignedEntityType};
use mithril_common::test::crypto_helper::setup_certificate_chain;

// Helper to create test certificates with specific chain types
fn create_test_certificates_with_chain_types() -> Vec<CertificateRecord> {
    // Create Cardano certificates
    let cardano_cert_1 = CertificateRecord::dummy_genesis("cardano-1", Epoch(1));
    let cardano_cert_2 = CertificateRecord::dummy_db_snapshot("cardano-2", "cardano-1", Epoch(1), 2);
    
    // Create Ethereum certificates
    let mut ethereum_cert_1 = CertificateRecord::dummy_genesis("ethereum-1", Epoch(1));
    ethereum_cert_1.chain_type = "ethereum".to_string();
    ethereum_cert_1.signed_entity_type = SignedEntityType::EthereumStateRoot(Epoch(1));
    
    let mut ethereum_cert_2 = CertificateRecord::dummy("ethereum-2", "ethereum-1", Epoch(1), SignedEntityType::EthereumStateRoot(Epoch(1)));
    ethereum_cert_2.chain_type = "ethereum".to_string();
    
    vec![cardano_cert_1, cardano_cert_2, ethereum_cert_1, ethereum_cert_2]
}

#[tokio::test]
async fn test_chain_type_is_persisted() {
    println!("\n======================================");
    println!("Testing chain_type Persistence");
    println!("======================================\n");
    
    let connection = Arc::new(main_db_connection().unwrap());
    let repository = CertificateRepository::new(connection.clone());
    
    // Insert certificates with different chain types
    let certificates = create_test_certificates_with_chain_types();
    
    println!("1. Inserting {} certificates...", certificates.len());
    repository.create_many_certificates(certificates.clone()).await
        .expect("Failed to insert certificates");
    
    // Retrieve all certificates
    println!("\n2. Retrieving all certificates...");
    let all_certs: Vec<CertificateRecord> = repository
        .get_latest_certificates(usize::MAX)
        .await
        .expect("Failed to retrieve certificates");
    
    println!("   ✓ Retrieved {} certificates", all_certs.len());
    assert_eq!(all_certs.len(), 4);
    
    // Verify chain types
    println!("\n3. Verifying chain types...");
    let cardano_count = all_certs.iter()
        .filter(|c| c.chain_type == "cardano")
        .count();
    let ethereum_count = all_certs.iter()
        .filter(|c| c.chain_type == "ethereum")
        .count();
    
    println!("   ✓ Cardano certificates: {}", cardano_count);
    println!("   ✓ Ethereum certificates: {}", ethereum_count);
    
    assert_eq!(cardano_count, 2, "Should have 2 Cardano certificates");
    assert_eq!(ethereum_count, 2, "Should have 2 Ethereum certificates");
    
    println!("\n======================================");
    println!("[PASS] Chain type persistence test passed!");
    println!("======================================\n");
}

#[tokio::test]
async fn test_query_certificates_by_chain_type() {
    println!("\n======================================");
    println!("Testing chain_type Filtering");
    println!("======================================\n");
    
    let connection = Arc::new(main_db_connection().unwrap());
    let repository = CertificateRepository::new(connection.clone());
    
    // Insert certificates
    let certificates = create_test_certificates_with_chain_types();
    repository.create_many_certificates(certificates).await
        .expect("Failed to insert certificates");
    
    // Query Cardano certificates
    println!("1. Querying Cardano certificates...");
    let cardano_result = connection.fetch_collect(
        "SELECT * FROM certificate WHERE chain_type = 'cardano' ORDER BY created_at DESC"
    ).unwrap();
    
    let cardano_certs: Vec<CertificateRecord> = cardano_result.into_iter()
        .map(|row| CertificateRecord::from_row(&row).unwrap())
        .collect();
    
    println!("   ✓ Found {} Cardano certificates", cardano_certs.len());
    assert_eq!(cardano_certs.len(), 2);
    assert!(cardano_certs.iter().all(|c| c.chain_type == "cardano"));
    
    // Query Ethereum certificates
    println!("\n2. Querying Ethereum certificates...");
    let ethereum_result = connection.fetch_collect(
        "SELECT * FROM certificate WHERE chain_type = 'ethereum' ORDER BY created_at DESC"
    ).unwrap();
    
    let ethereum_certs: Vec<CertificateRecord> = ethereum_result.into_iter()
        .map(|row| CertificateRecord::from_row(&row).unwrap())
        .collect();
    
    println!("   ✓ Found {} Ethereum certificates", ethereum_certs.len());
    assert_eq!(ethereum_certs.len(), 2);
    assert!(ethereum_certs.iter().all(|c| c.chain_type == "ethereum"));
    
    println!("\n======================================");
    println!("[PASS] Chain type filtering test passed!");
    println!("======================================\n");
}

#[tokio::test]
async fn test_backward_compatibility_default_cardano() {
    println!("\n======================================");
    println!("Testing Backward Compatibility");
    println!("======================================\n");
    
    let connection = Arc::new(main_db_connection().unwrap());
    
    // Insert a certificate directly with SQL (simulating old code)
    // that doesn't specify chain_type - it should default to 'cardano'
    println!("1. Inserting certificate without explicit chain_type...");
    connection.execute(
        "INSERT INTO certificate (
            certificate_id, parent_certificate_id, message, 
            signature, aggregate_verification_key, epoch, 
            network, beacon, signed_entity_beacon, 
            protocol_version, protocol_parameters, protocol_message, 
            signers, initiated_at, sealed_at
        ) VALUES (
            'test-default', NULL, 'test-message',
            'test-sig', 'test-avk', 1,
            'testnet', 0, 1,
            '1.0.0', '{}', '{}',
            '[]', datetime('now'), datetime('now')
        )"
    ).expect("Failed to insert test certificate");
    
    // Retrieve the certificate
    println!("\n2. Retrieving certificate...");
    let result = connection.fetch_one(
        "SELECT chain_type FROM certificate WHERE certificate_id = 'test-default'"
    ).unwrap();
    
    let chain_type: String = result.read("chain_type");
    
    println!("   ✓ chain_type: {}", chain_type);
    assert_eq!(chain_type, "cardano", "Default chain_type should be 'cardano'");
    
    println!("\n======================================");
    println!("[PASS] Backward compatibility test passed!");
    println!("======================================\n");
}

#[tokio::test]
async fn test_certificate_record_sets_chain_type_from_signed_entity() {
    println!("\n======================================");
    println!("Testing CertificateRecord Chain Type Derivation");
    println!("======================================\n");
    
    // Create a Cardano certificate chain
    let cardano_chain = setup_certificate_chain(2, 1);
    let cardano_cert = cardano_chain.certificates_chained[0].clone();
    let cardano_record = CertificateRecord::try_from(cardano_cert.clone())
        .expect("Failed to convert Cardano certificate");
    
    println!("1. Cardano certificate:");
    println!("   Type: {:?}", cardano_cert.signed_entity_type());
    println!("   chain_type: {}", cardano_record.chain_type);
    assert_eq!(cardano_record.chain_type, "cardano");
    
    // Create an Ethereum certificate record directly
    let ethereum_record = CertificateRecord::dummy(
        "eth-test",
        "eth-parent",
        Epoch(1),
        SignedEntityType::EthereumStateRoot(Epoch(1))
    );
    
    println!("\n2. Ethereum certificate:");
    println!("   Type: {:?}", ethereum_record.signed_entity_type);
    println!("   chain_type: {}", ethereum_record.chain_type);
    assert_eq!(ethereum_record.chain_type, "ethereum");
    
    println!("\n======================================");
    println!("[PASS] Chain type derivation test passed!");
    println!("======================================\n");
}

#[tokio::test]
async fn test_mixed_chain_certificates_dont_interfere() {
    println!("\n======================================");
    println!("Testing Mixed Chain Non-Interference");
    println!("======================================\n");
    
    let connection = Arc::new(main_db_connection().unwrap());
    let repository = CertificateRepository::new(connection.clone());
    
    // Create certificates for both chains at the same epoch
    let epoch = Epoch(10);
    
    let mut cardano_cert = CertificateRecord::dummy_genesis("cardano-epoch-10", epoch);
    cardano_cert.chain_type = "cardano".to_string();
    
    let mut ethereum_cert = CertificateRecord::dummy_genesis("ethereum-epoch-10", epoch);
    ethereum_cert.chain_type = "ethereum".to_string();
    ethereum_cert.signed_entity_type = SignedEntityType::EthereumStateRoot(epoch);
    
    println!("1. Inserting certificates for both chains at epoch {}...", epoch);
    repository.create_many_certificates(vec![cardano_cert.clone(), ethereum_cert.clone()]).await
        .expect("Failed to insert certificates");
    
    // Retrieve all certificates for this epoch
    let all_epoch_certs: Vec<CertificateRecord> = connection.fetch_collect(
        &format!("SELECT * FROM certificate WHERE epoch = {} ORDER BY chain_type", epoch.0)
    ).unwrap().into_iter()
        .map(|row| CertificateRecord::from_row(&row).unwrap())
        .collect();
    
    println!("\n2. Retrieved {} certificates for epoch {}", all_epoch_certs.len(), epoch);
    assert_eq!(all_epoch_certs.len(), 2);
    
    // Verify they are separate
    let cardano_certs: Vec<_> = all_epoch_certs.iter()
        .filter(|c| c.chain_type == "cardano")
        .collect();
    let ethereum_certs: Vec<_> = all_epoch_certs.iter()
        .filter(|c| c.chain_type == "ethereum")
        .collect();
    
    println!("   ✓ Cardano certificates: {}", cardano_certs.len());
    println!("   ✓ Ethereum certificates: {}", ethereum_certs.len());
    
    assert_eq!(cardano_certs.len(), 1);
    assert_eq!(ethereum_certs.len(), 1);
    assert_eq!(cardano_certs[0].certificate_id, "cardano-epoch-10");
    assert_eq!(ethereum_certs[0].certificate_id, "ethereum-epoch-10");
    
    println!("\n======================================");
    println!("[PASS] Non-interference test passed!");
    println!("======================================\n");
}

