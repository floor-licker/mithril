use clap::Parser;
use cli_table::{Cell, Table, format::Justify, print_stdout};

use mithril_client::{Client, MithrilResult};

use crate::{
    CommandContext,
    commands::client_builder_with_fallback_genesis_key,
};

/// Clap command to list existing Ethereum state root certificates
#[derive(Parser, Debug, Clone)]
pub struct EthereumStateListCommand {
    /// Maximum number of certificates to list
    #[clap(long, default_value = "10")]
    limit: usize,
}

impl EthereumStateListCommand {
    /// Main command execution
    pub async fn execute(&self, context: CommandContext) -> MithrilResult<()> {
        let client = client_builder_with_fallback_genesis_key(context.config_parameters())?
            .with_logger(context.logger().clone())
            .build()?;

        self.list_certificates(client, context).await
    }

    async fn list_certificates(&self, client: Client, context: CommandContext) -> MithrilResult<()> {
        // Use the client library to fetch certificates
        let mut certificates = client.ethereum_state().list().await?;
        certificates.truncate(self.limit);

        if context.is_json_output_enabled() {
            println!("{}", serde_json::to_string(&certificates)?);
        } else {
            self.print_table(&certificates)?;
        }
        
        Ok(())
    }
    fn print_table(&self, certificates: &[serde_json::Value]) -> MithrilResult<()> {
        if certificates.is_empty() {
            println!("No Ethereum state root certificates available yet.");
            return Ok(());
        }

        let items = certificates
            .iter()
            .map(|cert| {
                vec![
                    cert["beacon"]["epoch"]
                        .as_u64()
                        .map(|e| format!("{}", e))
                        .unwrap_or_else(|| "N/A".to_string())
                        .cell(),
                    cert["signed_entity_type"]
                        .as_str()
                        .unwrap_or("N/A")
                        .cell(),
                    cert["hash"]
                        .as_str()
                        .map(|h| format!("{}...", &h[..16]))
                        .unwrap_or_else(|| "N/A".to_string())
                        .cell(),
                    cert["created_at"]
                        .as_str()
                        .unwrap_or("N/A")
                        .cell()
                        .justify(Justify::Right),
                ]
            })
            .collect::<Vec<_>>()
            .table()
            .title(vec![
                "Epoch".cell(),
                "Type".cell(),
                "Certificate Hash".cell(),
                "Created".cell().justify(Justify::Right),
            ]);

        print_stdout(items)?;
        
        Ok(())
    }
}

