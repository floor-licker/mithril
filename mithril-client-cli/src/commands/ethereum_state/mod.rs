//! Commands for Ethereum state root artifacts
mod list;
mod show;
mod download;

pub use list::*;
pub use show::*;
pub use download::*;

use crate::CommandContext;
use clap::Subcommand;
use mithril_client::MithrilResult;

/// Ethereum state management (alias: eth)
#[derive(Subcommand, Debug, Clone)]
pub enum EthereumStateCommands {
    /// List available Ethereum state root certificates
    #[clap(arg_required_else_help = false)]
    List(EthereumStateListCommand),

    /// Show detailed information about an Ethereum state root certificate
    #[clap(arg_required_else_help = true)]
    Show(EthereumStateShowCommand),

    /// Download an Ethereum state root certificate and verify it
    #[clap(arg_required_else_help = true)]
    Download(EthereumStateDownloadCommand),
}

impl EthereumStateCommands {
    /// Execute Ethereum state command
    pub async fn execute(&self, context: CommandContext) -> MithrilResult<()> {
        match self {
            Self::List(cmd) => cmd.execute(context).await,
            Self::Show(cmd) => cmd.execute(context).await,
            Self::Download(cmd) => cmd.execute(context).await,
        }
    }
}

