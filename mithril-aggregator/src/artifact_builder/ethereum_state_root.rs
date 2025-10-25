use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;

use mithril_common::{
    StdResult,
    entities::{Certificate, Epoch, EthereumStateRoot},
    signable_builder::EthereumStateRootRetriever,
};

use crate::ArtifactBuilder;

/// An [EthereumStateRoot] artifact builder
pub struct EthereumStateRootArtifactBuilder {
    ethereum_state_root_retriever: Arc<dyn EthereumStateRootRetriever>,
}

impl EthereumStateRootArtifactBuilder {
    /// EthereumStateRoot artifact builder factory
    pub fn new(ethereum_state_root_retriever: Arc<dyn EthereumStateRootRetriever>) -> Self {
        Self {
            ethereum_state_root_retriever,
        }
    }
}

#[async_trait]
impl ArtifactBuilder<Epoch, EthereumStateRoot> for EthereumStateRootArtifactBuilder {
    async fn compute_artifact(
        &self,
        epoch: Epoch,
        _certificate: &Certificate,
    ) -> StdResult<EthereumStateRoot> {
        let state_root_data = self
            .ethereum_state_root_retriever
            .retrieve(epoch)
            .await?
            .ok_or_else(|| anyhow!("No Ethereum state root found for epoch '{}'", epoch))?;

        Ok(EthereumStateRoot::new(
            epoch,
            state_root_data.state_root,
            state_root_data.block_number,
        ))
    }
}

#[cfg(test)]
mod tests {
    use mithril_common::{
        signable_builder::EthereumStateRootData,
        test::double::fake_data,
    };
    use mockall::{mock, predicate::eq};

    use super::*;

    mock! {
        pub EthereumStateRootRetrieverImpl {}

        #[async_trait]
        impl EthereumStateRootRetriever for EthereumStateRootRetrieverImpl {
            async fn retrieve(&self, epoch: Epoch) -> StdResult<Option<EthereumStateRootData>>;
        }
    }

    #[tokio::test]
    async fn compute_artifact_returns_valid_artifact() {
        let epoch = Epoch(100);
        let certificate = fake_data::certificate("whatever".to_string());
        let state_root_data = EthereumStateRootData {
            state_root: "0x1234567890abcdef".to_string(),
            block_number: 12345,
            epoch: 100,
        };
        let mut mock_retriever = MockEthereumStateRootRetrieverImpl::new();
        mock_retriever
            .expect_retrieve()
            .with(eq(epoch))
            .return_once(move |_| Ok(Some(state_root_data)));
        let builder = EthereumStateRootArtifactBuilder::new(Arc::new(mock_retriever));

        let ethereum_state_root =
            builder.compute_artifact(epoch, &certificate).await.unwrap();

        let expected = EthereumStateRoot::new(epoch, "0x1234567890abcdef".to_string(), 12345);
        assert_eq!(ethereum_state_root.epoch, expected.epoch);
        assert_eq!(ethereum_state_root.state_root, expected.state_root);
        assert_eq!(ethereum_state_root.block_number, expected.block_number);
    }

    #[tokio::test]
    async fn compute_artifact_returns_error_if_no_state_root_found_for_epoch() {
        let epoch = Epoch(100);
        let certificate = fake_data::certificate("whatever".to_string());
        let mut mock_retriever = MockEthereumStateRootRetrieverImpl::new();
        mock_retriever
            .expect_retrieve()
            .with(eq(epoch))
            .return_once(move |_| Ok(None));
        let builder = EthereumStateRootArtifactBuilder::new(Arc::new(mock_retriever));

        builder
            .compute_artifact(epoch, &certificate)
            .await
            .expect_err("Should return error");
    }
}

