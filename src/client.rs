use std::{sync::Arc, time::Duration};

use reqwest::{Client, Method, RequestBuilder, Response};
use serde_json::{Value, json};
use tokio::sync::Semaphore;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("{0}")]
    Input(String),
    #[error("GOOGLE_MAPS_API_KEY is required to call Google Maps")]
    MissingKey,
    #[error("Google API request failed: {0}")]
    Api(String),
    #[error("Google API request timed out")]
    Timeout,
    #[error("Google API network request failed")]
    Network,
    #[error("Google API returned an invalid or oversized response")]
    Response,
}

#[derive(Clone)]
pub(crate) struct MapsClient {
    http: Client,
    key: Option<Arc<str>>,
    pub language: Arc<str>,
    permits: Arc<Semaphore>,
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
            language: language.into(),
            permits: Arc::new(Semaphore::new(5)),
            #[cfg(test)]
            test_base: None,
        })
    }

    pub fn redact(&self, message: &str) -> String {
        match self.key.as_deref() {
            Some(key) => message.replace(key, "[REDACTED]"),
            None => message.to_owned(),
        }
    }

    fn request(&self, method: Method, host: &str, path: &str) -> Result<RequestBuilder> {
        let key = self.key.as_deref().ok_or(Error::MissingKey)?;
        let url = format!("https://{host}{path}");
        #[cfg(test)]
        let url = self
            .test_base
            .as_ref()
            .map_or(url, |base| format!("{base}/{host}{path}"));
        Ok(self.http.request(method, url).header("X-Goog-Api-Key", key))
    }

    pub async fn get(
        &self,
        host: &str,
        path: &str,
        mut query: Vec<(String, String)>,
    ) -> Result<Value> {
        // Legacy Maps APIs and Weather use the key query parameter.
        query.push((
            "key".into(),
            self.key.as_deref().ok_or(Error::MissingKey)?.into(),
        ));
        self.json(self.request(Method::GET, host, path)?.query(&query))
            .await
    }

    pub async fn post(
        &self,
        host: &str,
        path: &str,
        body: &Value,
        mask: Option<&str>,
    ) -> Result<Value> {
        let mut request = self.request(Method::POST, host, path)?.json(body);
        if let Some(mask) = mask {
            request = request.header("X-Goog-FieldMask", mask);
        }
        self.json(request).await
    }

    async fn json(&self, request: RequestBuilder) -> Result<Value> {
        let _permit = self.permits.acquire().await.map_err(|_| Error::Network)?;
        let response = request.send().await.map_err(network_error)?;
        let status = response.status();
        let bytes = bounded_body(response).await?;
        let data: Value = serde_json::from_slice(&bytes).map_err(|_| Error::Response)?;
        if !status.is_success() || data.get("error").is_some() {
            let message = data
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("request rejected");
            return Err(Error::Api(format!(
                "HTTP {status}: {}",
                self.redact(message)
            )));
        }
        if let Some(status) = data.get("status").and_then(Value::as_str)
            && status != "OK"
            && status != "ZERO_RESULTS"
        {
            return Err(Error::Api(self.redact(&format!(
                "{status}: {}",
                data["error_message"].as_str().unwrap_or("request rejected")
            ))));
        }
        Ok(data)
    }

    pub async fn static_map(&self, mut query: Vec<(String, String)>) -> Result<Vec<u8>> {
        query.push((
            "key".into(),
            self.key.as_deref().ok_or(Error::MissingKey)?.into(),
        ));
        let request = self
            .request(Method::GET, "maps.googleapis.com", "/maps/api/staticmap")?
            .query(&query)
            .build()
            .map_err(|_| Error::Input("invalid static map request".into()))?;
        if request.url().as_str().len() > 16384 {
            return Err(Error::Input(
                "static map URL exceeds 16,384 characters".into(),
            ));
        }
        let _permit = self.permits.acquire().await.map_err(|_| Error::Network)?;
        let response = self.http.execute(request).await.map_err(network_error)?;
        if !response.status().is_success() {
            return Err(Error::Api(format!("static map HTTP {}", response.status())));
        }
        let bytes = bounded_body(response).await?;
        if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err(Error::Api("static map response was not a PNG image".into()));
        }
        Ok(bytes)
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
        let id = id.strip_prefix("places/").unwrap_or(id);
        if id.is_empty()
            || !id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        {
            return Err(Error::Input("invalid place ID".into()));
        }
        // Preserve the full Google response, including author/source/disclosure metadata.
        let mut place = self
            .json(
                self.request(
                    Method::GET,
                    "places.googleapis.com",
                    &format!("/v1/places/{id}"),
                )?
                .query(&[("languageCode", self.language.as_ref())])
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
                                )?
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
