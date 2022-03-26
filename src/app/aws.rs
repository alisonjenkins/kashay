use anyhow::{Result, Context};
use aws_sig_auth::signer::{self, OperationSigningConfig, HttpSignatureType, RequestConfig};
use aws_smithy_http::body::SdkBody;
use aws_types::region::{Region, SigningRegion};
use aws_types::{SigningService, credentials::ProvideCredentials};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
pub struct K8sToken {
    pub kind: String,
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<()>,
    pub status: K8sTokenStatus,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct K8sTokenStatus {
    #[serde(rename = "expirationTimestamp")]
    pub expiration_timestamp: String,
    pub token: String,
}

async fn get_cached_token(cluster_name: &str, region: &str) -> Result<String> {
    let service = "kashay";
    let username = cluster_name;
    let entry = keyring::Entry::new(&service, &username);

    let token_json = match entry.get_password() {
        Ok(token_json) => token_json,
        Err(e) => {
            return Err(anyhow::anyhow!("Error getting cached token: {}", e));
        }
    };

    // Use serde to deserialize the JSON
    match serde_json::from_str::<K8sToken>(&token_json).context("Failed to parse JSON encoded cached token") {
        Ok(token) => {
            let expiration = token.status.expiration_timestamp;
            let expiration_time = chrono::DateTime::parse_from_rfc3339(&expiration)?;
            let now = chrono::Utc::now();
            if now < expiration_time {
                return Ok(token_json);
            } else {
                let token_json = create_eks_token(cluster_name, region).await?;
                cache_token(cluster_name, &token_json).await?;
                return Ok(token_json)
            }
        },
        Err(e) => {
            Err(anyhow::anyhow!("Error deserializing token: {}", e))
        },
    }
}

async fn cache_token(cluster_name: &str, k8s_token: &str) -> Result<()> {
    let service = "kashay";
    let username = cluster_name;
    let entry = keyring::Entry::new(&service, &username);

    entry.set_password(k8s_token)?;
    Ok(())
}

async fn create_eks_token(cluster_name: &str, region: &str) -> Result<String> {
    // Convert region to AWS region
    let region = region.to_owned();
    let region = Region::new(region);
    
    // Get credentials
    let config = aws_config::load_from_env().await;
    let credentials = config.credentials_provider().unwrap().provide_credentials().await?;
    
    // Setup signer
    let signer = signer::SigV4Signer::new();
    let mut operation_config = OperationSigningConfig::default_config();
    operation_config.signature_type = HttpSignatureType::HttpRequestQueryParams;
    operation_config.expires_in = Some(Duration::from_secs(15 * 60));

    let request_ts = chrono::Utc::now();
    let request_config = RequestConfig {
        request_ts: request_ts.into(),
        region: &SigningRegion::from(region.clone()),
        service: &SigningService::from_static("sts"),
        payload_override: None,
    };

    // Create the request
    let mut request = http::Request::builder()
        .uri(format!("https://sts.{}.amazonaws.com/?Action=GetCallerIdentity&Version=2011-06-15", &region))
        .header("x-k8s-aws-id", cluster_name)
        .body(SdkBody::empty())
        .expect("valid request");

    // Sign the request
    let _signature = signer.sign(
        &operation_config,
        &request_config,
        &credentials,
        &mut request,
    );

    let uri = request.uri().to_string();
    let uri = base64::encode(&uri);
    let uri = "k8s-aws-v1.".to_owned() + &uri;
    let request_ts = request_ts.to_rfc3339();

    let token = K8sToken {
        kind: "ExecCredential".to_owned(),
        api_version: "client.authentication.k8s.io/v1".to_owned(),
        spec: None,
        status: K8sTokenStatus {
            expiration_timestamp: request_ts.to_owned(),
            token: uri.to_string().to_owned(),
        },
    };

    let token = serde_json::to_string(&token).context("Failed to serialize token")?;

    Ok(token)
}

pub async fn get_eks_token(cluster_name: &str, region: &str) -> Result<String> {
    match get_cached_token(cluster_name, region).await {
        Ok(cached_token) => {
            Ok(cached_token)
        }
        Err(_) => {
            let creds = create_eks_token(cluster_name, region).await?;
            cache_token(cluster_name, &creds).await?;
            return Ok(creds.to_string())
        }
    }
}
