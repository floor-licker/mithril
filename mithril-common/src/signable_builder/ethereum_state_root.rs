use anyhow::anyhow;
use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    StdResult,
    entities::{Epoch, ProtocolMessage, ProtocolMessagePartKey},
    signable_builder::SignableBuilder,
};

#[cfg(test)]
use mockall::automock;

/// Ethereum State Root data needed for protocol message
#[derive(Debug, Clone)]
pub struct EthereumStateRootData {
    /// The state root hash (as 0x-prefixed hex string)
    pub state_root: String,
    /// The beacon block number
    pub block_number: u64,
    /// The epoch
    pub epoch: u64,
}

/// Ethereum State Root Retriever
#[cfg_attr(test, automock)]
#[async_trait]
pub trait EthereumStateRootRetriever: Send + Sync {
    /// Retrieve the Ethereum state root for a given epoch
    async fn retrieve(&self, epoch: Epoch) -> StdResult<Option<EthereumStateRootData>>;
}

/// A [EthereumStateRootSignableBuilder] builder that creates protocol messages for Ethereum state roots
pub struct EthereumStateRootSignableBuilder {
    ethereum_state_root_retriever: Arc<dyn EthereumStateRootRetriever>,
}

impl EthereumStateRootSignableBuilder {
    /// Constructor
    pub fn new(ethereum_state_root_retriever: Arc<dyn EthereumStateRootRetriever>) -> Self {
        Self {
            ethereum_state_root_retriever,
        }
    }
}

#[async_trait]
impl SignableBuilder<Epoch> for EthereumStateRootSignableBuilder {
    async fn compute_protocol_message(&self, epoch: Epoch) -> StdResult<ProtocolMessage> {
        let state_root_data = self
            .ethereum_state_root_retriever
            .retrieve(epoch)
            .await?
            .ok_or(anyhow!(
                "EthereumStateRootSignableBuilder could not find the state root for epoch: '{epoch}'"
            ))?;

        let mut protocol_message = ProtocolMessage::new();
        protocol_message.set_message_part(
            ProtocolMessagePartKey::EthereumEpoch,
            state_root_data.epoch.to_string(),
        );
        protocol_message.set_message_part(
            ProtocolMessagePartKey::EthereumStateRoot,
            state_root_data.state_root,
        );
        protocol_message.set_message_part(
            ProtocolMessagePartKey::EthereumBeaconBlockNumber,
            state_root_data.block_number.to_string(),
        );

        Ok(protocol_message)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use crate::entities::ProtocolMessagePartKey;

    use super::*;

    #[tokio::test]
    async fn compute_protocol_message_returns_error_when_no_state_root_found() {
        let epoch = Epoch(1);

        let mut ethereum_state_root_retriever = MockEthereumStateRootRetriever::new();
        ethereum_state_root_retriever
            .expect_retrieve()
            .return_once(move |_| Ok(None));
        let ethereum_state_root_signable_builder =
            EthereumStateRootSignableBuilder::new(Arc::new(ethereum_state_root_retriever));

        ethereum_state_root_signable_builder
            .compute_protocol_message(epoch)
            .await
            .expect_err("Should return an error when no ethereum state root found");
    }

    #[tokio::test]
    async fn compute_protocol_message_returns_signable_with_state_root() {
        let epoch = Epoch(1);
        let state_root_data = EthereumStateRootData {
            state_root: "0x1234567890abcdef".to_string(),
            block_number: 100,
            epoch: 1,
        };

        let mut ethereum_state_root_retriever = MockEthereumStateRootRetriever::new();
        ethereum_state_root_retriever
            .expect_retrieve()
            .with(eq(epoch))
            .return_once(move |_| Ok(Some(state_root_data)));
        let ethereum_state_root_signable_builder =
            EthereumStateRootSignableBuilder::new(Arc::new(ethereum_state_root_retriever));

        let signable = ethereum_state_root_signable_builder
            .compute_protocol_message(epoch)
            .await
            .unwrap();

        let mut signable_expected = ProtocolMessage::new();
        signable_expected.set_message_part(
            ProtocolMessagePartKey::EthereumEpoch,
            "1".to_string(),
        );
        signable_expected.set_message_part(
            ProtocolMessagePartKey::EthereumStateRoot,
            "0x1234567890abcdef".to_string(),
        );
        signable_expected.set_message_part(
            ProtocolMessagePartKey::EthereumBeaconBlockNumber,
            "100".to_string(),
        );
        assert_eq!(signable_expected, signable);
    }
}

