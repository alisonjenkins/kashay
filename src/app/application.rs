use super::super::cli::args::parse_args;
use super::aws::get_eks_token;
use anyhow::Result;
use std::io::Write;

pub async fn run() -> Result<()> {
    let args = parse_args();
    let creds = get_eks_token(&args.cluster_name, &args.region).await?;
    std::io::stdout().write_all(&creds.as_bytes())?;
    Ok(())
}
