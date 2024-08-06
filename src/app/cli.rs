use crate::app::aws::GetEKSTokenInput;
use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    /// AWS profile to use for authentication
    pub aws_profile: String,

    /// Name of the AWS region that the cluster is in
    #[clap(short, long, default_value = "eu-west-2")]
    pub aws_region: String,

    /// Name of the EKS Kubernetes cluster to get a token for
    #[clap(short, long)]
    pub cluster_name: String,

    /// Session name to use when assuming the role
    #[clap(short = 's', long, default_value = None)]
    pub session_name: Option<String>,
}

impl From<CliArgs> for GetEKSTokenInput {
    fn from(val: CliArgs) -> Self {
        GetEKSTokenInput {
            aws_profile: val.aws_profile,
            aws_region: val.aws_region,
            cluster_name: val.cluster_name,
            session_name: val.session_name,
        }
    }
}
