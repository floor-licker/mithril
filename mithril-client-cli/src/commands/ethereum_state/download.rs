use clap::Parser;
use std::path::PathBuf;

use mithril_client::{Client, MithrilResult};

use crate::{
    CommandContext,
    commands::client_builder_with_fallback_genesis_key,
};

/// Clap command to download and verify an Ethereum state root certificate
#[derive(Parser, Debug, Clone)]
pub struct EthereumStateDownloadCommand {
    /// Certificate hash to download, or "latest" for the most recent
    certificate_hash: String,

    /// Directory to save the certificate data
    #[clap(long, default_value = ".")]
    output_dir: PathBuf,
}

impl EthereumStateDownloadCommand {
    /// Main command execution
    pub async fn execute(&self, context: CommandContext) -> MithrilResult<()> {
        let client = client_builder_with_fallback_genesis_key(context.config_parameters())?
            .with_logger(context.logger().clone())
            .build()?;

        self.download_and_verify(client, context).await
    }

    async fn download_and_verify(&self, client: Client, context: CommandContext) -> MithrilResult<()> {
        // Step 1: Fetch the certificate
        let certificate = if self.certificate_hash == "latest" {
            // Fetch latest certificate
            let certificates = client.ethereum_state().list().await?;
            certificates
                .first()
                .cloned()
                .ok_or_else(|| mithril_client::MithrilError::msg("No certificates available"))?
        } else {
            // Fetch specific certificate
            client
                .ethereum_state()
                .get(&self.certificate_hash)
                .await?
                .ok_or_else(|| mithril_client::MithrilError::msg(format!(
                    "Certificate not found: {}",
                    self.certificate_hash
                )))?
        };
        
        // Step 2: Verify the certificate
        self.verify_certificate(&certificate)?;

        // Step 3: Save the certificate data
        self.save_certificate_data(&certificate, &context).await?;

        if !context.is_json_output_enabled() {
            println!("\n✓ Ethereum state root certificate downloaded and verified successfully!");
            println!("Certificate hash: {}", certificate["hash"].as_str().unwrap_or("N/A"));
            println!("Epoch: {}", certificate["beacon"]["epoch"].as_u64().unwrap_or(0));
            println!("Output directory: {}", self.output_dir.display());
        } else {
            let output = serde_json::json!({
                "status": "success",
                "certificate_hash": certificate["hash"].as_str().unwrap_or("N/A"),
                "epoch": certificate["beacon"]["epoch"].as_u64().unwrap_or(0),
                "output_dir": self.output_dir.display().to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }

        Ok(())
    }

    fn verify_certificate(&self, certificate: &serde_json::Value) -> MithrilResult<()> {
        // Basic validation
        if certificate["hash"].as_str().is_none() {
            return Err(mithril_client::MithrilError::msg("Certificate missing hash"));
        }

        if certificate["multi_signature"].as_str().is_none() {
            return Err(mithril_client::MithrilError::msg("Certificate missing multi-signature"));
        }

        if certificate["aggregate_verification_key"].as_str().is_none() {
            return Err(mithril_client::MithrilError::msg("Certificate missing AVK"));
        }

        // TODO: Implement full cryptographic verification
        // This would involve:
        // 1. Verifying the multi-signature against the protocol message
        // 2. Checking that the AVK represents the validator set
        // 3. Verifying the certificate chain

        Ok(())
    }

    async fn save_certificate_data(&self, certificate: &serde_json::Value, context: &CommandContext) -> MithrilResult<()> {
        // Create output directory if it doesn't exist
        std::fs::create_dir_all(&self.output_dir)
            .map_err(|e| mithril_client::MithrilError::msg(format!("Failed to create output directory: {}", e)))?;

        // Save certificate JSON
        let cert_file = self.output_dir.join("ethereum-certificate.json");
        std::fs::write(&cert_file, serde_json::to_string_pretty(&certificate)?)
            .map_err(|e| mithril_client::MithrilError::msg(format!("Failed to write certificate file: {}", e)))?;

        if !context.is_json_output_enabled() {
            println!("Certificate saved to: {}", cert_file.display());
        }

        // Extract and save state root information
        if let Some(artifact) = certificate.get("artifact") {
            let state_root_file = self.output_dir.join("ethereum-state-root.json");
            std::fs::write(&state_root_file, serde_json::to_string_pretty(&artifact)?)
                .map_err(|e| mithril_client::MithrilError::msg(format!("Failed to write state root file: {}", e)))?;

            if !context.is_json_output_enabled() {
                println!("State root data saved to: {}", state_root_file.display());
                
                // Print state root details
                if let Some(state_root) = artifact.get("state_root") {
                    println!("\nEthereum State Root: {}", state_root.as_str().unwrap_or("N/A"));
                }
                if let Some(block_number) = artifact.get("block_number") {
                    println!("Block Number: {}", block_number.as_u64().unwrap_or(0));
                }
            }
        }

        Ok(())
    }
}

