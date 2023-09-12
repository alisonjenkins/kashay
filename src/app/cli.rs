use clap::{App, Arg};

pub struct CliArgs {
    pub cluster_name: String,
    pub role_arn: Option<String>,
    pub skip_cache: bool,
    pub region: String,
}

impl CliArgs {
    pub fn parse() -> CliArgs {
        let matches = App::new("EKS Token Generator")
            .version("1.0")
            .author("Your Name")
            .about("Generates an EKS token")
            .arg(
                Arg::with_name("cluster_name")
                    .short("c")
                    .long("cluster-name")
                    .value_name("CLUSTER_NAME")
                    .help("Name of the EKS Kubernetes cluster")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("role_arn")
                    .short("r")
                    .long("role-arn")
                    .value_name("ROLE_ARN")
                    .help("ARN of a role to assume for authentication")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("skip_cache")
                    .short("s")
                    .long("skip-cache")
                    .help("Skip caching and always get a new token")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("region")
                    .short("R")
                    .long("region")
                    .value_name("REGION")
                    .help("AWS region of the cluster")
                    .default_value("eu-west-2")
                    .takes_value(true),
            )
            .get_matches();

        CliArgs {
            cluster_name: matches.value_of("cluster_name").unwrap().to_string(),
            role_arn: matches.value_of("role_arn").map(String::from),
            skip_cache: matches.is_present("skip_cache"),
            region: matches.value_of("region").unwrap().to_string(),
        }
    }
}

fn main() {
    let args = CliArgs::parse();

    // Use args.cluster_name, args.role_arn, args.skip_cache, and args.region in your code.
    println!("Cluster Name: {}", args.cluster_name);
    println!("Role ARN: {:?}", args.role_arn);
    println!("Skip Cache: {}", args.skip_cache);
    println!("Region: {}", args.region);
}
