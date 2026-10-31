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
    streetview_credentials: OnceCell<AccessTokenCredentials>,
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
            streetview_credentials: OnceCell::new(),
        }))
    }

    pub async fn authorize(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        self.authorize_scoped(request, false).await
    }

    pub async fn authorize_streetview(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        self.authorize_scoped(request, true).await
    }

    async fn authorize_scoped(
        &self,
        request: RequestBuilder,
        streetview: bool,
    ) -> Result<RequestBuilder> {
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
            let cache = if streetview {
                &self.0.streetview_credentials
            } else {
                &self.0.credentials
            };
            let credentials = cache
                .get_or_try_init(|| async {
                    let path = self.0.credentials_file.clone();
                    // ADC discovery and credential-file parsing perform filesystem I/O.
                    tokio::task::spawn_blocking(move || build_credentials(path, streetview))
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

    pub async fn check_local(&self) -> Result<bool> {
        self.0.quota_project.as_deref().filter(|value| !value.trim().is_empty())
            .and_then(|value| reqwest::header::HeaderValue::from_str(value).ok())
            .ok_or_else(|| Error::Input("GOOGLE_CLOUD_QUOTA_PROJECT is required and must be a valid project ID or number".into()))?;
        let Some(path) = self.0.credentials_file.clone() else {
            return Ok(false);
        };
        tokio::task::spawn_blocking(move || build_credentials(Some(path), false))
            .await
            .map_err(|_| Error::OAuth)??;
        Ok(true)
    }
}

fn build_credentials(path: Option<PathBuf>, streetview: bool) -> Result<AccessTokenCredentials> {
    let scope = if streetview {
        "https://www.googleapis.com/auth/streetviewpublish"
    } else {
        CLOUD_PLATFORM_SCOPE
    };
    if streetview && path.is_none() {
        return Err(Error::OAuthRequired(
            "Street View Publish: an authorized_user credential file with streetviewpublish consent",
        ));
    }
    let Some(path) = path else {
        return credentials::Builder::default()
            .with_scopes([scope])
            .build_access_token_credentials()
            .map_err(|_| Error::OAuth);
    };
    let bytes = std::fs::read(path).map_err(|_| Error::OAuth)?;
    let json: Value = serde_json::from_slice(&bytes).map_err(|_| Error::OAuth)?;
    if streetview && json.get("type").and_then(Value::as_str) != Some("authorized_user") {
        return Err(Error::OAuthRequired(
            "Street View Publish: end-user OAuth, not a service account",
        ));
    }
    // Explicit builders honor paths read from .env without changing process-wide ADC variables.
    match json.get("type").and_then(Value::as_str) {
        Some("service_account") => credentials::service_account::Builder::new(json)
            .with_access_specifier(credentials::service_account::AccessSpecifier::from_scopes(
                [scope],
            ))
            .build_access_token_credentials(),
        Some("external_account") => credentials::external_account::Builder::new(json)
            .with_scopes([scope])
            .build_access_token_credentials(),
        Some("impersonated_service_account") => credentials::impersonated::Builder::new(json)
            .with_scopes([scope])
            .build_access_token_credentials(),
        Some("authorized_user") => credentials::user_account::Builder::new(json)
            .with_scopes([scope])
            .build_access_token_credentials(),
        _ => return Err(Error::OAuth),
    }
    .map_err(|_| Error::OAuth)
}
