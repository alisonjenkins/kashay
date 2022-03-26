use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Name of the EKS Kubernetes cluster to get credentials for
    #[clap(short, long)]
    pub cluster_name: String,

    /// Name of the AWS region that the cluster is in
    #[clap(short, long, default_value = "eu-west-2")]
    pub region: String,
}

pub fn parse_args() -> CliArgs {
    CliArgs::parse()
}
