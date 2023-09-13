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

// Assumes the specified role arn and returns the credentials
async fn assume_role(role_arn: &str, session_name: &Option<String>) ->  Result<aws_sdk_iam::Credentials> {
    let shared_config = aws_config::load_from_env().await;
    let session_name = if let Some(session_name) = session_name {
        session_name.clone()
    } else {
        "kashay".to_string()
    };

    let sts_config = aws_sdk_sts::config::Builder::from(&shared_config)
        .retry_config(aws_config::retry::RetryConfig::standard())
        .build();

    let sts_client = aws_sdk_sts::Client::from_conf(sts_config);

    let assumed_role = sts_client
        .assume_role()
        .role_arn(role_arn)
        .role_session_name(session_name)
        .send()
        .await?;

    let assumed_role_credentials = assumed_role.credentials.as_ref().ok_or_else(|| anyhow::anyhow!("Unable to get credentials for assumed role"))?;

    let assumed_credentials = aws_sdk_iam::Credentials::new(
        assumed_role_credentials
            .access_key_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Unable to get access key id for assumed role"))?,
        assumed_role_credentials
            .secret_access_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Unable to get secret access key for assumed role"))?,
        assumed_role_credentials
            .session_token
            .clone(),
        Some(SystemTime::try_from(
            assumed_role_credentials
                .expiration()
                .ok_or_else(|| anyhow::anyhow!("Unable to get expiration for assumed role"))?
                .to_owned(),
        )?),
        "meh",
    );

    Ok(assumed_credentials)
}

pub async fn get_eks_token(cluster_name: &str, role_arn: &Option<String>, session_name: &Option<String>) -> Result<String> {
    let region = Region::new("eu-west-2");

    let credentials = match role_arn {
        Some(role_arn) => assume_role(role_arn, session_name).await?,
        None => {
            let config = aws_config::load_from_env().await;
            config
                .credentials_provider().ok_or_else(|| anyhow::anyhow!("Unable to get credentials provider"))?
                .provide_credentials()
                .await?
        }
    };

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

    let mut request = http::Request::builder()
        .uri(format!(
            "https://sts.eu-west-2.amazonaws.com/?Action=GetCallerIdentity&Version=2011-06-15"
        ))
        .header("x-k8s-aws-id", cluster_name)
        .body(SdkBody::empty())?;

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

#[cfg(test)]
mod test {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn test_get_eks_token() -> Result<()> {
        let reqwest_client = reqwest::Client::new();
        let cluster_name = "syn-scout-k8s-playground";

        for _ in 0..9 {
            let result = get_eks_token(cluster_name, &None, &None).await?;

            let parsed_json: K8sToken = serde_json::from_str(&result)?;

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
