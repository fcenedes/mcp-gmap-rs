use std::{path::PathBuf, sync::Arc, time::Duration};

use google_cloud_auth::credentials::{self, AccessTokenCredentials};
use reqwest::RequestBuilder;
use serde_json::Value;
use tokio::sync::OnceCell;

use crate::client::{Error, Result};

const CLOUD_PLATFORM_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

#[derive(Clone)]
pub(crate) struct OAuth(Arc<OAuthState>);

struct OAuthState {
    credentials_file: Option<PathBuf>,
    quota_project: Option<String>,
    timeout: Duration,
    credentials: OnceCell<AccessTokenCredentials>,
}

impl OAuth {
    pub fn new(
        credentials_file: Option<PathBuf>,
        quota_project: Option<String>,
        timeout: Duration,
    ) -> Self {
        Self(Arc::new(OAuthState {
            credentials_file,
            quota_project,
            timeout,
            credentials: OnceCell::new(),
        }))
    }

    pub async fn authorize(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        let project = self
            .0
            .quota_project
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                Error::Input("GOOGLE_CLOUD_QUOTA_PROJECT is required for OAuth requests".into())
            })?;
        let project = reqwest::header::HeaderValue::from_str(project).map_err(|_| {
            Error::Input(
                "GOOGLE_CLOUD_QUOTA_PROJECT must be a valid Google Cloud project ID or number"
                    .into(),
            )
        })?;
        let token = tokio::time::timeout(self.0.timeout, async {
            let credentials = self
                .0
                .credentials
                .get_or_try_init(|| async {
                    let path = self.0.credentials_file.clone();
                    // ADC discovery and credential-file parsing perform filesystem I/O.
                    tokio::task::spawn_blocking(move || build_credentials(path))
                        .await
                        .map_err(|_| Error::OAuth)?
                })
                .await?;
            credentials.access_token().await.map_err(|_| Error::OAuth)
        })
        .await
        .map_err(|_| Error::OAuthTimeout)??;
        Ok(request
            .bearer_auth(token.token)
            .header("X-Goog-User-Project", project))
    }
}

fn build_credentials(path: Option<PathBuf>) -> Result<AccessTokenCredentials> {
    let Some(path) = path else {
        return credentials::Builder::default()
            .with_scopes([CLOUD_PLATFORM_SCOPE])
            .build_access_token_credentials()
            .map_err(|_| Error::OAuth);
    };
    let bytes = std::fs::read(path).map_err(|_| Error::OAuth)?;
    let json: Value = serde_json::from_slice(&bytes).map_err(|_| Error::OAuth)?;
    // Explicit builders honor paths read from .env without changing process-wide ADC variables.
    match json.get("type").and_then(Value::as_str) {
        Some("service_account") => credentials::service_account::Builder::new(json)
            .with_access_specifier(credentials::service_account::AccessSpecifier::from_scopes(
                [CLOUD_PLATFORM_SCOPE],
            ))
            .build_access_token_credentials(),
        Some("external_account") => credentials::external_account::Builder::new(json)
            .with_scopes([CLOUD_PLATFORM_SCOPE])
            .build_access_token_credentials(),
        Some("impersonated_service_account") => credentials::impersonated::Builder::new(json)
            .with_scopes([CLOUD_PLATFORM_SCOPE])
            .build_access_token_credentials(),
        Some("authorized_user") => credentials::user_account::Builder::new(json)
            .with_scopes([CLOUD_PLATFORM_SCOPE])
            .build_access_token_credentials(),
        _ => return Err(Error::OAuth),
    }
    .map_err(|_| Error::OAuth)
}
