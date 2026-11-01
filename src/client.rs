use std::{collections::BTreeMap, sync::Arc, time::Duration};

use reqwest::{Client, Method, RequestBuilder, Response};
use serde_json::{Value, json};
use tokio::sync::Semaphore;

use crate::{
    auth::OAuth,
    metrics::Metrics,
    policy::{CallBudget, ClientPolicy},
};

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("{0}")]
    Input(String),
    #[error("GOOGLE_MAPS_API_KEY is required to call Google Maps")]
    MissingKey,
    #[error(
        "Google OAuth authentication failed; check GOOGLE_APPLICATION_CREDENTIALS / ADC, service-account permissions and connectivity"
    )]
    OAuth,
    #[error("Google OAuth token acquisition timed out")]
    OAuthTimeout,
    #[error(
        "Google OAuth credentials are required for {0}; configure OAuth mode and the documented credential type"
    )]
    OAuthRequired(&'static str),
    #[error("Google API request failed: {0}")]
    Api(String),
    #[error("Google API request timed out")]
    Timeout,
    #[error("Google API network request failed")]
    Network,
    #[error("Google API returned an invalid or oversized response")]
    Response,
    #[error("Google request budget for this tool call was exhausted")]
    Budget,
    #[error("Tool execution deadline exceeded")]
    ToolTimeout,
}

pub(crate) struct BinaryResponse {
    pub data: Vec<u8>,
    pub mime_type: String,
    pub headers: BTreeMap<String, String>,
}

#[derive(Clone)]
pub(crate) struct MapsClient {
    http: Client,
    key: Option<Arc<str>>,
    pub oauth: Option<OAuth>,
    pub language: Arc<str>,
    permits: Arc<Semaphore>,
    pub metrics: Arc<Metrics>,
    pub policy: ClientPolicy,
    timeout: Duration,
    budget: Option<Arc<CallBudget>>,
    #[cfg(test)]
    pub test_base: Option<String>,
}

impl MapsClient {
    pub fn new(key: Option<String>, timeout: Duration, language: String) -> Result<Self> {
        let http = Client::builder()
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("mcp-gmap-rs/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| Error::Network)?;
        Ok(Self {
            http,
            key: key.filter(|s| !s.trim().is_empty()).map(Arc::from),
            oauth: None,
            language: language.into(),
            permits: Arc::new(Semaphore::new(5)),
            metrics: Arc::default(),
            policy: ClientPolicy::default(),
            timeout,
            budget: None,
            #[cfg(test)]
            test_base: None,
        })
    }

    pub fn for_call(&self) -> Self {
        let mut client = self.clone();
        client.budget = Some(Arc::new(CallBudget::new(self.policy.max_requests)));
        client
    }

    pub fn has_key(&self) -> bool {
        self.key.is_some()
    }

    pub fn redact(&self, message: &str) -> String {
        match self.key.as_deref() {
            Some(key) => message.replace(key, "[REDACTED]"),
            None => message.to_owned(),
        }
    }

    fn uses_oauth(&self, host: &str) -> bool {
        self.oauth.is_some()
            && matches!(
                host,
                "places.googleapis.com"
                    | "routes.googleapis.com"
                    | "weather.googleapis.com"
                    | "airquality.googleapis.com"
                    | "addressvalidation.googleapis.com"
                    | "areainsights.googleapis.com"
                    | "solar.googleapis.com"
                    | "tile.googleapis.com"
                    | "mapsplatformdatasets.googleapis.com"
                    | "routeoptimization.googleapis.com"
                    | "streetviewpublish.googleapis.com"
            )
    }

    async fn request(&self, method: Method, host: &str, path: &str) -> Result<RequestBuilder> {
        if !matches!(
            host,
            "maps.googleapis.com"
                | "places.googleapis.com"
                | "routes.googleapis.com"
                | "weather.googleapis.com"
                | "airquality.googleapis.com"
                | "addressvalidation.googleapis.com"
                | "areainsights.googleapis.com"
                | "roads.googleapis.com"
                | "pollen.googleapis.com"
                | "solar.googleapis.com"
                | "aerialview.googleapis.com"
                | "www.googleapis.com"
                | "tile.googleapis.com"
                | "mapsplatformdatasets.googleapis.com"
                | "routeoptimization.googleapis.com"
                | "streetviewpublish.googleapis.com"
        ) || !path.starts_with('/')
            || path.starts_with("//")
            || path.contains(['?', '#', '\\'])
            || path.split('/').any(|part| part == "." || part == "..")
        {
            return Err(Error::Input("invalid Google API endpoint".into()));
        }
        if matches!(
            host,
            "mapsplatformdatasets.googleapis.com"
                | "routeoptimization.googleapis.com"
                | "streetviewpublish.googleapis.com"
        ) && self.oauth.is_none()
        {
            return Err(Error::OAuthRequired("this Google service"));
        }
        let url = format!("https://{host}{path}");
        #[cfg(test)]
        let url = self
            .test_base
            .as_ref()
            .map_or(url, |base| format!("{base}/{host}{path}"));
        let request = self.http.request(method, url);
        if self.uses_oauth(host)
            && let Some(oauth) = &self.oauth
        {
            return if host == "streetviewpublish.googleapis.com" {
                oauth.authorize_streetview(request).await
            } else {
                oauth.authorize(request).await
            };
        }
        let key = self.key.as_deref().ok_or(Error::MissingKey)?;
        Ok(request.header("X-Goog-Api-Key", key))
    }

    fn authenticated_query(
        &self,
        host: &str,
        query: Vec<(String, String)>,
    ) -> Result<Vec<(String, String)>> {
        let mut signature = None;
        let mut pairs = Vec::with_capacity(query.len() + 1);
        for (name, value) in query {
            if name == "signature" {
                if signature.replace(value).is_some() {
                    return Err(Error::Input("duplicate URL signature".into()));
                }
            } else {
                pairs.push((name, value));
            }
        }
        if !self.uses_oauth(host) {
            pairs.push((
                "key".into(),
                self.key.as_deref().ok_or(Error::MissingKey)?.into(),
            ));
        }
        // Google signs the complete encoded path/query, including the key; the
        // caller's signature is appended only after that unsigned URI is final.
        if let Some(value) = signature {
            pairs.push(("signature".into(), value));
        }
        Ok(pairs)
    }

    pub async fn get(&self, host: &str, path: &str, query: Vec<(String, String)>) -> Result<Value> {
        let request = self.request(Method::GET, host, path).await?;
        let query = self.authenticated_query(host, query)?;
        self.json(request.query(&query)).await
    }

    pub async fn post(
        &self,
        host: &str,
        path: &str,
        body: &Value,
        mask: Option<&str>,
    ) -> Result<Value> {
        let mut request = self.request(Method::POST, host, path).await?.json(body);
        if let Some(mask) = mask {
            request = request.header("X-Goog-FieldMask", mask);
        }
        self.json(request).await
    }

    async fn json(&self, request: RequestBuilder) -> Result<Value> {
        self.json_response(request, false).await
    }

    async fn json_response(&self, request: RequestBuilder, allow_empty: bool) -> Result<Value> {
        let request = request.build().map_err(network_error)?;
        let bearer = request
            .headers()
            .get(reqwest::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(str::to_owned);
        let redact = |message: &str| {
            let message = self.redact(message);
            match bearer.as_deref().filter(|value| !value.is_empty()) {
                Some(token) => message.replace(token, "[REDACTED]"),
                None => message,
            }
        };
        let (status, bytes, _) = self.send(request).await?;
        let mut data: Value = if bytes.is_empty()
            && (matches!(status.as_u16(), 204 | 205) || (allow_empty && status.is_success()))
        {
            json!({})
        } else {
            serde_json::from_slice(&bytes).map_err(|_| Error::Response)?
        };
        if !status.is_success() || (data.get("error").is_some() && data.get("done").is_none()) {
            let message = data
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("request rejected");
            return Err(Error::Api(format!("HTTP {status}: {}", redact(message))));
        }
        if let Some(status) = data.get("status").and_then(Value::as_str)
            && status != "OK"
            && status != "ZERO_RESULTS"
        {
            return Err(Error::Api(redact(&format!(
                "{status}: {}",
                data["error_message"].as_str().unwrap_or("request rejected")
            ))));
        }
        redact_values(&mut data, &redact);
        Ok(data)
    }

    /// Fixed-host JSON transport. Mutating operations must set retry_safe to false.
    pub async fn call_json(
        &self,
        method: &str,
        host: &str,
        path: &str,
        query: Vec<(String, String)>,
        body: Option<&Value>,
        retry_safe: bool,
    ) -> Result<Value> {
        let method = Method::from_bytes(method.as_bytes())
            .map_err(|_| Error::Input("invalid HTTP method".into()))?;
        let mut request = self.request(method, host, path).await?;
        let query = self.authenticated_query(host, query)?;
        request = request.query(&query);
        if let Some(body) = body {
            request = request.json(body);
        }
        let mut client = self.clone();
        if !retry_safe {
            client.policy.max_retries = 0;
        }
        client.json(request).await
    }

    pub async fn download(
        &self,
        host: &str,
        path: &str,
        query: Vec<(String, String)>,
    ) -> Result<BinaryResponse> {
        let request = self.request(Method::GET, host, path).await?;
        let query = self.authenticated_query(host, query)?;
        let request = request.query(&query).build().map_err(network_error)?;
        let bearer = request
            .headers()
            .get(reqwest::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let (status, data, headers) = self.send(request).await?;
        if !status.is_success() {
            let message = serde_json::from_slice::<Value>(&data)
                .ok()
                .and_then(|v| {
                    v.pointer("/error/message")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
                .unwrap_or_else(|| "binary request rejected".into());
            let mut message = self.redact(&message);
            if let Some(token) = bearer.as_deref().and_then(|v| v.strip_prefix("Bearer ")) {
                message = message.replace(token, "[REDACTED]");
            }
            return Err(Error::Api(format!("HTTP {status}: {message}")));
        }
        let mime_type = headers
            .get("content-type")
            .and_then(|v| v.split(';').next())
            .unwrap_or("application/octet-stream")
            .trim()
            .to_ascii_lowercase();
        if !matches!(
            mime_type.as_str(),
            "image/png"
                | "image/jpeg"
                | "image/webp"
                | "image/tiff"
                | "application/octet-stream"
                | "model/gltf-binary"
                | "application/vnd.google-earth.kmz"
                | "text/csv"
                | "application/geo+json"
                | "application/json"
                | "application/vnd.google-earth.kml+xml"
                | "text/plain"
        ) || (mime_type == "image/png" && !data.starts_with(b"\x89PNG\r\n\x1a\n"))
            || (mime_type == "image/jpeg" && !data.starts_with(b"\xff\xd8\xff"))
            || (mime_type == "image/webp"
                && !(data.starts_with(b"RIFF") && data.get(8..12) == Some(b"WEBP")))
        {
            return Err(Error::Response);
        }
        Ok(BinaryResponse {
            data,
            mime_type,
            headers,
        })
    }

    pub async fn upload(
        &self,
        host: &str,
        path: &str,
        query: Vec<(String, String)>,
        data: Vec<u8>,
        mime_type: &str,
    ) -> Result<Value> {
        if data.is_empty() || data.len() > 8 * 1024 * 1024 {
            return Err(Error::Input("upload must contain 1 byte to 8 MiB".into()));
        }
        let request = self.request(Method::POST, host, path).await?;
        let query = self.authenticated_query(host, query)?;
        let request = request
            .query(&query)
            .header(reqwest::header::CONTENT_TYPE, mime_type)
            .body(data);
        let mut client = self.clone();
        client.policy.max_retries = 0;
        client.json_response(request, true).await
    }

    pub async fn static_map(&self, mut query: Vec<(String, String)>) -> Result<Vec<u8>> {
        query.push((
            "key".into(),
            self.key.as_deref().ok_or(Error::MissingKey)?.into(),
        ));
        let request = self
            .request(Method::GET, "maps.googleapis.com", "/maps/api/staticmap")
            .await?
            .query(&query)
            .build()
            .map_err(|_| Error::Input("invalid static map request".into()))?;
        if request.url().as_str().len() > 16384 {
            return Err(Error::Input(
                "static map URL exceeds 16,384 characters".into(),
            ));
        }
        let (status, bytes, _) = self.send(request).await?;
        if !status.is_success() {
            return Err(Error::Api(format!("static map HTTP {status}")));
        }
        if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err(Error::Api("static map response was not a PNG image".into()));
        }
        Ok(bytes)
    }

    async fn send(
        &self,
        request: reqwest::Request,
    ) -> Result<(reqwest::StatusCode, Vec<u8>, BTreeMap<String, String>)> {
        tokio::time::timeout(self.timeout, self.send_with_retries(request))
            .await
            .map_err(|_| Error::Timeout)?
    }

    async fn send_with_retries(
        &self,
        request: reqwest::Request,
    ) -> Result<(reqwest::StatusCode, Vec<u8>, BTreeMap<String, String>)> {
        let api = request.url().host_str().unwrap_or("unknown");
        for attempt in 0..=self.policy.max_retries {
            let retry_request = request.try_clone().ok_or(Error::Network)?;
            let permit = self.permits.acquire().await.map_err(|_| Error::Network)?;
            if let Some(budget) = &self.budget {
                budget.take()?;
            }
            let response = self.http.execute(retry_request).await;
            let delay = match response {
                Ok(response) => {
                    let status = response.status();
                    self.metrics.upstream(api, status.as_str());
                    let delay = response
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|value| value.to_str().ok())
                        .and_then(retry_after);
                    let headers = [
                        "content-type",
                        "cache-control",
                        "etag",
                        "expires",
                        "last-modified",
                    ]
                    .into_iter()
                    .filter_map(|name| {
                        response
                            .headers()
                            .get(name)
                            .and_then(|v| v.to_str().ok())
                            .map(|v| (name.to_owned(), self.redact(v)))
                    })
                    .collect();
                    let bytes = bounded_body(response).await?;
                    if attempt == self.policy.max_retries
                        || !matches!(status.as_u16(), 429 | 500 | 502 | 503 | 504)
                    {
                        return Ok((status, bytes, headers));
                    }
                    delay
                }
                Err(error) => {
                    self.metrics.upstream(
                        api,
                        if error.is_timeout() {
                            "timeout"
                        } else {
                            "network"
                        },
                    );
                    if attempt == self.policy.max_retries {
                        return Err(network_error(error));
                    }
                    None
                }
            };
            drop(permit);
            let jitter = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
                % 100;
            let backoff =
                Duration::from_millis(100 * (1_u64 << attempt.min(5)) + u64::from(jitter));
            tokio::time::sleep(delay.unwrap_or(backoff)).await;
        }
        Err(Error::Network)
    }

    pub async fn geocode(&self, address: &str) -> Result<Value> {
        let data = self
            .get(
                "maps.googleapis.com",
                "/maps/api/geocode/json",
                vec![
                    ("address".into(), address.into()),
                    ("language".into(), self.language.to_string()),
                ],
            )
            .await?;
        let first = data["results"]
            .as_array()
            .and_then(|a| a.first())
            .ok_or_else(|| Error::Api("no location found".into()))?;
        Ok(
            json!({"location":first["geometry"]["location"],"formatted_address":first["formatted_address"],"place_id":first["place_id"]}),
        )
    }

    pub async fn search_text(&self, body: &Value) -> Result<Vec<Value>> {
        let data = self
            .post(
                "places.googleapis.com",
                "/v1/places:searchText",
                body,
                Some(SEARCH_MASK),
            )
            .await?;
        Ok(data["places"].as_array().cloned().unwrap_or_default())
    }

    pub async fn details(&self, id: &str, photos: usize) -> Result<Value> {
        self.details_session(id, photos, None).await
    }

    pub async fn details_session(
        &self,
        id: &str,
        photos: usize,
        session_token: Option<&str>,
    ) -> Result<Value> {
        let id = id.strip_prefix("places/").unwrap_or(id);
        if id.is_empty()
            || !id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        {
            return Err(Error::Input("invalid place ID".into()));
        }
        let mut query = vec![("languageCode", self.language.as_ref())];
        if let Some(token) = session_token {
            query.push(("sessionToken", token));
        }
        // Preserve the full Google response, including author/source/disclosure metadata.
        let mut place = self
            .json(
                self.request(
                    Method::GET,
                    "places.googleapis.com",
                    &format!("/v1/places/{id}"),
                )
                .await?
                .query(&query)
                .header("X-Goog-FieldMask", DETAIL_MASK),
            )
            .await?;
        if photos > 0 {
            let mut urls = Vec::new();
            if let Some(entries) = place["photos"].as_array() {
                for photo in entries.iter().take(photos) {
                    if let Some(name) = photo["name"].as_str() {
                        if !name.starts_with(&format!("places/{id}/photos/"))
                            || name.contains("..")
                            || name.contains('?')
                            || name.contains('#')
                        {
                            return Err(Error::Response);
                        }
                        let media = self
                            .json(
                                self.request(
                                    Method::GET,
                                    "places.googleapis.com",
                                    &format!("/v1/{name}/media"),
                                )
                                .await?
                                .query(&[("maxWidthPx", "800"), ("skipHttpRedirect", "true")]),
                            )
                            .await?;
                        urls.push(json!({"photoUri":media["photoUri"],"authorAttributions":photo["authorAttributions"],"googleMapsUri":photo["googleMapsUri"]}));
                    }
                }
            }
            place["photoUrls"] = json!(urls);
        }
        Ok(place)
    }
}

fn retry_after(value: &str) -> Option<Duration> {
    value
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
        .or_else(|| {
            httpdate::parse_http_date(value).ok().map(|date| {
                date.duration_since(std::time::SystemTime::now())
                    .unwrap_or_default()
            })
        })
}

fn network_error(error: reqwest::Error) -> Error {
    // reqwest errors may contain request URLs (and query-string API keys).
    if error.is_timeout() {
        Error::Timeout
    } else {
        Error::Network
    }
}

async fn bounded_body(mut response: Response) -> Result<Vec<u8>> {
    const LIMIT: usize = 16 * 1024 * 1024;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(network_error)? {
        if bytes.len() + chunk.len() > LIMIT {
            return Err(Error::Response);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

pub(crate) const SEARCH_MASK: &str = "places.id,places.name,places.displayName,places.formattedAddress,places.location,places.primaryType,places.rating,places.userRatingCount,places.currentOpeningHours,places.priceLevel,places.googleMapsUri,places.attributions";
const DETAIL_MASK: &str = "id,name,displayName,formattedAddress,location,utcOffsetMinutes,primaryType,types,regularOpeningHours,currentOpeningHours,nationalPhoneNumber,websiteUri,priceLevel,rating,userRatingCount,editorialSummary,reviews,photos,googleMapsUri,attributions,parkingOptions,accessibilityOptions,servesVegetarianFood,servesBeer,servesWine,servesCocktails,servesBreakfast,servesLunch,servesDinner,servesBrunch,servesCoffee,servesDessert,dineIn,delivery,takeout,curbsidePickup,reservable,goodForGroups,goodForChildren,goodForWatchingSports,liveMusic,outdoorSeating,allowsDogs,menuForChildren,restroom,paymentOptions,reviewSummary,generativeSummary";

fn redact_values(value: &mut Value, redact: &impl Fn(&str) -> String) {
    match value {
        Value::String(text) => *text = redact(text),
        Value::Array(items) => items.iter_mut().for_each(|v| redact_values(v, redact)),
        Value::Object(items) => items.values_mut().for_each(|v| redact_values(v, redact)),
        _ => {}
    }
}
