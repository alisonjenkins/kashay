use super::aws::get_eks_token;
use super::cli::parse_args;
use anyhow::Result;
use tokio::io::AsyncWriteExt;

pub async fn run() -> Result<()> {
    let args = parse_args();
    let creds = get_eks_token(&args.cluster_name, &args.region).await?;
    tokio::io::stdout().write_all(&creds.as_bytes()).await?;
    Ok(())
}
