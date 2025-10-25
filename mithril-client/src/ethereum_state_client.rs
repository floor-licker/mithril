//! A client to retrieve Ethereum state root data from an Aggregator.
//!
//! In order to do so it defines an [EthereumStateClient] which exposes the following features:
//!  - [get][EthereumStateClient::get]: get an Ethereum state root certificate from its hash
//!  - [list][EthereumStateClient::list]: get the list of available Ethereum state root certificates
//!
//! # Get an Ethereum state root certificate
//!
//! To get an Ethereum state root certificate using the [ClientBuilder][crate::client::ClientBuilder].
//!
//! ```no_run
//! # async fn run() -> mithril_client::MithrilResult<()> {
//! use mithril_client::ClientBuilder;
//!
//! let client = ClientBuilder::aggregator("YOUR_AGGREGATOR_ENDPOINT", "YOUR_GENESIS_VERIFICATION_KEY").build()?;
//! let ethereum_certificate = client.ethereum_state().get("CERTIFICATE_HASH").await?.unwrap();
//!
//! println!("Ethereum certificate hash={}, epoch={:?}", ethereum_certificate["hash"], ethereum_certificate["epoch"]);
//! #    Ok(())
//! # }
//! ```
//!
//! # List available Ethereum state root certificates
//!
//! To list available Ethereum state root certificates using the [ClientBuilder][crate::client::ClientBuilder].
//!
//! ```no_run
//! # async fn run() -> mithril_client::MithrilResult<()> {
//! use mithril_client::ClientBuilder;
//!
//! let client = ClientBuilder::aggregator("YOUR_AGGREGATOR_ENDPOINT", "YOUR_GENESIS_VERIFICATION_KEY").build()?;
//! let ethereum_certificates = client.ethereum_state().list().await?;
//!
//! for certificate in ethereum_certificates {
//!     println!("Ethereum certificate hash={}", certificate["hash"]);
//! }
//! #    Ok(())
//! # }
//! ```

use std::sync::Arc;

use crate::aggregator_client::{AggregatorClient, AggregatorClientError, AggregatorRequest};
use anyhow::Context;

use crate::MithrilResult;

/// HTTP client for Ethereum State Root API from the Aggregator
pub struct EthereumStateClient {
    aggregator_client: Arc<dyn AggregatorClient>,
}

impl EthereumStateClient {
    /// Constructs a new `EthereumStateClient`.
    pub fn new(aggregator_client: Arc<dyn AggregatorClient>) -> Self {
        Self { aggregator_client }
    }

    /// Fetch a list of Ethereum state root certificates
    pub async fn list(&self) -> MithrilResult<Vec<serde_json::Value>> {
        let response = self
            .aggregator_client
            .get_content(AggregatorRequest::ListEthereumCertificates)
            .await
            .with_context(|| "Ethereum State Client cannot get the certificate list")?;
        let items = serde_json::from_str::<Vec<serde_json::Value>>(&response)
            .with_context(|| "Ethereum State Client cannot deserialize certificate list")?;

        Ok(items)
    }

    /// Get the given Ethereum state root certificate. If it cannot be found, a None is returned.
    pub async fn get(&self, hash: &str) -> MithrilResult<Option<serde_json::Value>> {
        match self
            .aggregator_client
            .get_content(AggregatorRequest::GetEthereumCertificate {
                hash: hash.to_string(),
            })
            .await
        {
            Ok(content) => {
                let certificate: serde_json::Value = serde_json::from_str(&content)
                    .with_context(|| "Ethereum State Client cannot deserialize certificate")?;

                Ok(Some(certificate))
            }
            Err(AggregatorClientError::RemoteServerLogical(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregator_client::MockAggregatorClient;

    use super::*;

    fn fake_certificate() -> serde_json::Value {
        serde_json::json!({
            "hash": "cert-hash-123",
            "epoch": 1,
            "beacon": {
                "epoch": 1,
            },
            "signed_entity_type": "EthereumStateRoot",
            "created_at": "2023-01-19T13:43:05.618857482Z",
        })
    }

    fn fake_certificates() -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "hash": "cert-hash-123",
                "epoch": 1,
                "signed_entity_type": "EthereumStateRoot",
                "created_at": "2023-01-19T13:43:05.618857482Z",
            }),
            serde_json::json!({
                "hash": "cert-hash-456",
                "epoch": 2,
                "signed_entity_type": "EthereumStateRoot",
                "created_at": "2023-01-19T13:44:05.618857482Z",
            }),
        ]
    }

    #[tokio::test]
    async fn get_ethereum_certificate_list() {
        let certificates = fake_certificates();
        let mut http_client = MockAggregatorClient::new();
        http_client
            .expect_get_content()
            .return_once(move |_| Ok(serde_json::to_string(&certificates).unwrap()));
        let client = EthereumStateClient::new(Arc::new(http_client));
        let items = client.list().await.unwrap();

        assert_eq!(2, items.len());
        assert_eq!("cert-hash-123", items[0]["hash"].as_str().unwrap());
        assert_eq!("cert-hash-456", items[1]["hash"].as_str().unwrap());
    }

    #[tokio::test]
    async fn get_ethereum_certificate() {
        let certificate = fake_certificate();
        let mut http_client = MockAggregatorClient::new();
        http_client
            .expect_get_content()
            .return_once(move |_| Ok(serde_json::to_string(&certificate).unwrap()));
        let client = EthereumStateClient::new(Arc::new(http_client));
        let cert = client
            .get("cert-hash-123")
            .await
            .unwrap()
            .expect("This test returns a certificate");

        assert_eq!("cert-hash-123", cert["hash"].as_str().unwrap());
    }
}

