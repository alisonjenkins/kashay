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

async fn create_eks_token(cluster_name: &str, role_arn: &Option<String>) -> Result<String> {
    let region = Region::new("eu-west-2");

    let credentials = if let Some(role_arn) = role_arn {
        // Assume role logic here
    } else {
        // Get credentials from the default provider
        let config = aws_config::load_from_env().await;
        config
            .credentials_provider()
            .unwrap()
            .provide_credentials()
            .await?
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

pub async fn get_eks_token(cluster_name: &str, role_arn: &Option<String>) -> Result<String> {
    create_eks_token(cluster_name, role_arn).await
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_get_eks_token() -> Result<()> {
        let reqwest_client = reqwest::Client::new();
        let cluster_name = "syn-scout-k8s-playground";

        for _ in 0..9 {
            let result = get_eks_token(cluster_name, &None).await?;

            let parsed_json: K8sToken = serde_json::from_str(&result)?;

            let token = parsed_json.status.token;
            let url = base64::decode(token.replace("k8s-aws-v1.", ""))?;
            let url = std::str::from_utf8(&url)?;

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
