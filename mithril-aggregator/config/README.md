# Mithril Aggregator Configuration Examples

This directory contains example configuration files for the Mithril Aggregator.

## Configuration Files

### `dev.json`
Configuration for local development with a devnet Cardano node.
- Network: devnet (magic: 42)
- Snapshot storage: local filesystem
- Run interval: 30 seconds

### `preview.json`
Configuration for Cardano preview testnet.
- Network: preview
- Snapshot storage: Google Cloud Storage
- Run interval: 60 seconds

### `ethereum-holesky.json`
**Multi-chain configuration** enabling both Cardano and Ethereum support.
- Cardano: preview network
- Ethereum: Holesky testnet
- Certified entity types: Cardano Immutable Files + Ethereum State Roots
- Run interval: 60 seconds

## Multi-Chain Configuration

To enable Ethereum support alongside Cardano, add these fields to your configuration:

```json
{
  "signed_entity_types": "CardanoImmutableFilesFull,EthereumStateRoot",
  "enable_ethereum_observer": true,
  "ethereum_beacon_endpoint": "https://ethereum-holesky-beacon-api.publicnode.com",
  "ethereum_network": "holesky",
  "ethereum_certification_interval_epochs": 10
}
```

### Configuration Fields

| Field | Required | Description | Example |
|-------|----------|-------------|---------|
| `signed_entity_types` | Yes | Comma-separated list of entity types to certify | `"CardanoImmutableFilesFull,EthereumStateRoot"` |
| `enable_ethereum_observer` | Yes | Enable Ethereum chain observer | `true` |
| `ethereum_beacon_endpoint` | Yes (if enabled) | Ethereum Beacon Chain API endpoint | `"https://ethereum-holesky-beacon-api.publicnode.com"` |
| `ethereum_network` | Yes (if enabled) | Ethereum network identifier | `"holesky"`, `"mainnet"`, or `"sepolia"` |
| `ethereum_certification_interval_epochs` | No | How often to certify (in Ethereum epochs) | `10` (default) |

### Supported Ethereum Networks

- **mainnet**: Ethereum mainnet (production)
- **holesky**: Ethereum Holesky testnet (recommended for testing)
- **sepolia**: Ethereum Sepolia testnet

### Public Beacon Endpoints

For testing purposes, you can use these public Beacon API endpoints:

- **Holesky**: `https://ethereum-holesky-beacon-api.publicnode.com`
- **Mainnet**: `https://ethereum-beacon-api.publicnode.com`
- **Sepolia**: `https://ethereum-sepolia-beacon-api.publicnode.com`

**Note**: For production use, run your own Beacon node for reliability and performance.

## Signed Entity Types

Available entity types for the `signed_entity_types` configuration:

### Cardano
- `MithrilStakeDistribution` - Always included by default
- `CardanoStakeDistribution` - Cardano stake distribution
- `CardanoImmutableFilesFull` - Full Cardano immutable file snapshots
- `CardanoTransactions` - Cardano transaction history
- `CardanoDatabase` - Cardano database snapshots

### Ethereum
- `EthereumStateRoot` - Ethereum execution layer state roots

## Running the Aggregator

### With Ethereum Support
```bash
mithril-aggregator serve --config config/ethereum-holesky.json
```

### Cardano Only (Legacy)
```bash
mithril-aggregator serve --config config/preview.json
```

## Architecture

When Ethereum support is enabled:
- The aggregator runs **both chain observers concurrently**
- Each blockchain maintains its own **independent certificate chain**
- Certificates are served on **chain-specific API endpoints**:
  - `/cardano/certificates` - Cardano certificates
  - `/ethereum/certificates` - Ethereum certificates
  - `/certificates` - Legacy endpoint (defaults to Cardano)

## Troubleshooting

### Connection Issues
If the aggregator cannot connect to the Ethereum beacon node:
1. Verify the endpoint is accessible: `curl https://ethereum-holesky-beacon-api.publicnode.com/eth/v1/node/health`
2. Check network connectivity and firewall rules
3. Try an alternative public endpoint

### Configuration Errors
If you see "Unknown Ethereum network" errors:
- Ensure `ethereum_network` is one of: `mainnet`, `holesky`, `sepolia` (lowercase)

### Missing Dependencies
If the aggregator fails to start:
- Verify `enable_ethereum_observer` is set to `true`
- Verify `ethereum_beacon_endpoint` and `ethereum_network` are configured
- Check that `EthereumStateRoot` is in the `signed_entity_types` list

