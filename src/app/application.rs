use clap::Parser;
use crate::app::cli::CliArgs;
use super::aws::get_eks_token;
use anyhow::Result;
use tokio::io::AsyncWriteExt;

pub async fn run() -> Result<()> {
    let args = CliArgs::parse();
    let creds = get_eks_token(&args.cluster_name, &args.region, &args.skip_cache).await?;
    tokio::io::stdout().write_all(&creds.as_bytes()).await?;
    Ok(())
}
