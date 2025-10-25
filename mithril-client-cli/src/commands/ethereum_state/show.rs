use clap::Parser;
use cli_table::{Cell, CellStruct, Table, print_stdout};

use mithril_client::{Client, MithrilResult};

use crate::{
    CommandContext,
    commands::client_builder_with_fallback_genesis_key,
};

/// Clap command to show a specific Ethereum state root certificate
#[derive(Parser, Debug, Clone)]
pub struct EthereumStateShowCommand {
    /// Certificate hash to show
    certificate_hash: String,
}

impl EthereumStateShowCommand {
    /// Main command execution
    pub async fn execute(&self, context: CommandContext) -> MithrilResult<()> {
        let client = client_builder_with_fallback_genesis_key(context.config_parameters())?
            .with_logger(context.logger().clone())
            .build()?;

        self.show_certificate(client, context).await
    }

    async fn show_certificate(&self, client: Client, context: CommandContext) -> MithrilResult<()> {
        let certificate = client
            .ethereum_state()
            .get(&self.certificate_hash)
            .await?
            .ok_or_else(|| mithril_client::MithrilError::msg(format!(
                "Certificate not found: {}",
                self.certificate_hash
            )))?;

        if context.is_json_output_enabled() {
            println!("{}", serde_json::to_string_pretty(&certificate)?);
        } else {
            self.print_certificate_details(&certificate)?;
        }
        
        Ok(())
    }

    fn print_certificate_details(&self, cert: &serde_json::Value) -> MithrilResult<()> {
        let rows: Vec<Vec<CellStruct>> = vec![
            vec![
                "Certificate Hash".cell(),
                cert["hash"].as_str().unwrap_or("N/A").cell(),
            ],
            vec![
                "Epoch".cell(),
                cert["beacon"]["epoch"]
                    .as_u64()
                    .map(|e| format!("{}", e))
                    .unwrap_or_else(|| "N/A".to_string())
                    .cell(),
            ],
            vec![
                "Signed Entity Type".cell(),
                cert["signed_entity_type"]
                    .as_str()
                    .unwrap_or("N/A")
                    .cell(),
            ],
            vec![
                "Multi-Signature".cell(),
                cert["multi_signature"]
                    .as_str()
                    .map(|s| format!("{}...", &s[..32]))
                    .unwrap_or_else(|| "N/A".to_string())
                    .cell(),
            ],
            vec![
                "Aggregate Verification Key".cell(),
                cert["aggregate_verification_key"]
                    .as_str()
                    .map(|s| format!("{}...", &s[..32]))
                    .unwrap_or_else(|| "N/A".to_string())
                    .cell(),
            ],
            vec![
                "Previous Hash".cell(),
                cert["previous_hash"]
                    .as_str()
                    .unwrap_or("N/A")
                    .cell(),
            ],
            vec![
                "Created At".cell(),
                cert["created_at"]
                    .as_str()
                    .unwrap_or("N/A")
                    .cell(),
            ],
        ];

        let table = rows.table();
        print_stdout(table)?;

        // Print additional metadata if available
        if let Some(metadata) = cert["metadata"].as_object() {
            println!("\nMetadata:");
            for (key, value) in metadata {
                println!("  {}: {}", key, value.as_str().unwrap_or("N/A"));
            }
        }

        Ok(())
    }
}

