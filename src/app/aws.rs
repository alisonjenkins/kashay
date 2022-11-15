use anyhow::{Context, Result};
use aws_sig_auth::signer::{self, HttpSignatureType, OperationSigningConfig, RequestConfig};
use aws_smithy_http::body::SdkBody;
use aws_types::region::{Region, SigningRegion};
use aws_types::{credentials::ProvideCredentials, SigningService};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug, Deserialize, Serialize)]
pub struct K8sToken {
    pub kind: String,
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: HashMap<String, ()>,
    pub status: K8sTokenStatus,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct K8sTokenStatus {
    #[serde(rename = "expirationTimestamp")]
    pub expiration_timestamp: String,
    pub token: String,
}

// async fn get_cached_token(cluster_name: &str, role_arn: &Option<String>) -> Result<String> {
//     let service = "kashay";
//     let username = cluster_name;
//     let entry = keyring::Entry::new(service, username);
//
//     let token_json = match entry.get_password() {
//         Ok(token_json) => token_json,
//         Err(e) => {
//             return Err(anyhow::anyhow!("Error getting cached token: {}", e));
//         }
//     };
//
//     // Use serde to deserialize the JSON
//     match serde_json::from_str::<K8sToken>(&token_json)
//         .context("Failed to parse JSON encoded cached token")
//     {
//         Ok(token) => {
//             let expiration = token.status.expiration_timestamp;
//             let expiration_time = chrono::DateTime::parse_from_rfc3339(&expiration)?;
//             let now = chrono::Utc::now();
//             if now < expiration_time {
//                 Ok(token_json)
//             } else {
//                 let token_json = create_eks_token(cluster_name, role_arn).await?;
//                 cache_token(cluster_name, &token_json).await?;
//                 Ok(token_json)
//             }
//         }
//         Err(e) => Err(anyhow::anyhow!("Error deserializing token: {}", e)),
//     }
// }
//
// async fn cache_token(cluster_name: &str, k8s_token: &str) -> Result<()> {
//     let service = "kashay";
//     let username = cluster_name;
//     let entry = keyring::Entry::new(service, username);
//
//     entry.set_password(k8s_token)?;
//     Ok(())
// }

async fn create_eks_token(cluster_name: &str, role_arn: &Option<String>) -> Result<String> {
    // Convert region to AWS region
    let region = Region::new("eu-west-2");

    // Get credentials
    let credentials = if role_arn.is_some() {
        let shared_config = aws_config::load_from_env().await;
        let sts_config = aws_sdk_sts::config::Builder::from(&shared_config)
            .retry_config(aws_config::retry::RetryConfig::standard())
            .build();
        let sts_client = aws_sdk_sts::Client::from_conf(sts_config);
        let assumed_role = sts_client
            .assume_role()
            .role_arn(role_arn.as_ref().unwrap())
            .role_session_name("kashay")
            .send()
            .await?;
        let assumed_credentials = aws_sdk_iam::Credentials::new(
            assumed_role
                .credentials
                .as_ref()
                .unwrap()
                .access_key_id
                .as_ref()
                .unwrap(),
            assumed_role
                .credentials
                .as_ref()
                .unwrap()
                .secret_access_key
                .as_ref()
                .unwrap(),
            assumed_role
                .credentials
                .as_ref()
                .expect("Unable to get session token for assumed role")
                .session_token
                .clone(),
            Some(SystemTime::try_from(
                assumed_role
                    .credentials
                    .as_ref()
                    .expect("Unable to get the expiration for the assumed role credentials")
                    .expiration()
                    .unwrap()
                    .to_owned(),
            )?),
            "meh",
        );

        assumed_credentials
    } else {
        let config = aws_config::load_from_env().await;
        config
            .credentials_provider()
            .unwrap()
            .provide_credentials()
            .await?
    };

    // Setup signer
    let signer = signer::SigV4Signer::new();
    let mut operation_config = OperationSigningConfig::default_config();
    operation_config.signature_type = HttpSignatureType::HttpRequestQueryParams;
    operation_config.expires_in = Some(Duration::from_secs(60));

    let request_ts = chrono::Utc::now();
    let request_config = RequestConfig {
        request_ts: request_ts.into(),
        region: &SigningRegion::from(region.clone()),
        service: &SigningService::from_static("sts"),
        payload_override: None,
    };

    // Create the request
    let mut request = http::Request::builder()
        .uri(format!(
            "https://sts.eu-west-2.amazonaws.com/?Action=GetCallerIdentity&Version=2011-06-15"
        ))
        .header("x-k8s-aws-id", cluster_name)
        .body(SdkBody::empty())?;

    // Sign the request
    let _signature = signer.sign(
        &operation_config,
        &request_config,
        &credentials,
        &mut request,
    );

    let uri = format!(
        "k8s-aws-v1.{}",
        base64::encode_config(request.uri().to_string(), base64::URL_SAFE_NO_PAD)
    );
    let request_ts = request_ts.to_rfc3339();

    // Generate output JSON
    let token = K8sToken {
        kind: "ExecCredential".to_string(),
        api_version: "client.authentication.k8s.io/v1beta1".to_string(),
        spec: HashMap::new(),
        status: K8sTokenStatus {
            expiration_timestamp: request_ts,
            token: uri,
        },
    };

    serde_json::to_string(&token).context("Failed to serialize token")
}

pub async fn get_eks_token(cluster_name: &str, role_arn: &Option<String>) -> Result<String> {
    create_eks_token(cluster_name, role_arn).await
}

#[cfg(test)]
mod test {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn test_get_eks_token() -> Result<()> {
        let reqwest_client = reqwest::Client::new();
        let cluster_name = "syn-scout-k8s-playground";

        for _ in 0..9 {
            let result = get_eks_token(cluster_name, &None).await;

            if result.as_ref().is_err() {
                println!("Failed to generate token: {:?}", result);
            }

            println!("Successfully generated token: {:?}", result);

            // extract the url to call from the token
            let parsed_json: K8sToken = serde_json::from_str(&result.unwrap())?;

            let token = parsed_json.status.token;
            let url = base64::decode(token.replace("k8s-aws-v1.", ""))?;
            let url = std::str::from_utf8(&url)?;
            println!("Decoded to url: {:?}", url);

            let resp = reqwest_client
                .get(url)
                .header("x-k8s-aws-id", cluster_name)
                .send()
                .await?;

            let status = resp.status();
            let body = resp.text().await?;

            if status != 200 {
                println!(
                    "Request failed with http: {} and response body: {}",
                    status, &body
                );
            }

            println!(
                "Request succeeded with http: {} and response body: {}",
                status, &body
            );
        }
        Ok(())
    }
}
