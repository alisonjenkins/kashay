use super::aws::get_eks_token;
use crate::app::cli::CliArgs;
use anyhow::Result;
use clap::Parser;
use tokio::io::AsyncWriteExt;

pub async fn run() -> Result<()> {
    let args = CliArgs::parse();
    let creds = get_eks_token(&args.cluster_name, &args.skip_cache, &args.role_arn).await?;
    tokio::io::stdout().write_all(creds.as_bytes()).await?;
    Ok(())
}
