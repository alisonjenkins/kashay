use super::aws::get_eks_token;
use super::aws::GetEKSTokenInput;
use anyhow::Result;
use clap::Parser;
use tokio::io::AsyncWriteExt;

pub async fn run() -> Result<()> {
    let args = GetEKSTokenInput::parse();
    let creds = get_eks_token(&args).await?;
    tokio::io::stdout().write_all(creds.as_bytes()).await?;
    tokio::io::stdout().flush().await?;
    Ok(())
}
