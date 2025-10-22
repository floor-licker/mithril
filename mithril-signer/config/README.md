# Mithril Signer Configuration Examples

This directory contains example configuration files for running Mithril signers on different blockchain networks.

## Cardano Networks

- `config.json` - Default Cardano mainnet configuration
- `preview.json` - Cardano Preview testnet configuration

## Ethereum Networks

- `ethereum-mainnet-example.json` - Ethereum Mainnet configuration
- `ethereum-holesky-example.json` - Ethereum Holesky testnet configuration

## Configuration Fields

### Common Fields (All Chains)

- `chain_type`: Type of blockchain (`"cardano"` or `"ethereum"`)
- `aggregator_endpoint`: URL of the Mithril aggregator
- `relay_endpoint`: Optional relay endpoint for additional communication
- `run_interval`: Interval in milliseconds between signature attempts
- `db_directory`: Directory for database storage
- `data_stores_directory`: Directory for additional data stores
- `era_reader_adapter_type`: Type of era reader (`"bootstrap"`, `"cardano-chain"`, etc.)
- `enable_metrics_server`: Whether to enable Prometheus metrics
- `metrics_server_ip`: IP address for metrics server
- `metrics_server_port`: Port for metrics server

### Cardano-Specific Fields

- `cardano_cli_path`: Path to cardano-cli binary
- `cardano_node_socket_path`: Path to Cardano node socket
- `network`: Cardano network name (`"mainnet"`, `"preprod"`, `"preview"`)
- `network_magic`: Network magic number for testnets
- `network_security_parameter`: Number of blocks for finality
- `kes_secret_key_path`: Path to KES secret key
- `operational_certificate_path`: Path to operational certificate

### Ethereum-Specific Fields

- `beacon_endpoint`: URL of Ethereum Beacon node API
- `network`: Ethereum network name (`"mainnet"`, `"holesky"`, `"sepolia"`)
- `validator_pubkey`: BLS public key of the validator (48 bytes hex)
- `validator_seckey_path`: Path to BLS secret key file
- `certification_interval_epochs`: How often to certify (in Ethereum epochs)

## Usage

### Cardano Signer

```bash
mithril-signer -vvv --config /path/to/cardano/config.json
```

### Ethereum Signer

```bash
mithril-signer -vvv --config /path/to/ethereum/config.json
```

## Environment Variables

Configuration values can also be set via environment variables with the prefix `MITHRIL_`:

```bash
export MITHRIL_CHAIN_TYPE=ethereum
export MITHRIL_BEACON_ENDPOINT=http://localhost:5052
export MITHRIL_NETWORK=holesky
export MITHRIL_VALIDATOR_PUBKEY=0x...
```

## Backward Compatibility

Existing Cardano configurations without `chain_type` will default to `"cardano"` for backward compatibility.

## Notes

- Ethereum signers require access to a Beacon node with API enabled
- Certification interval of 675 epochs is approximately 3 days on Ethereum
- Validator keys must be securely stored and have appropriate permissions
- For testnet, use appropriate test validator keys

