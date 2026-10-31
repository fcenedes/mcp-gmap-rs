use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{DateTime, FixedOffset, Utc};
use serde_json::{Value, json};

use crate::{
    catalog::Catalog,
    client::{Error, MapsClient, Result},
    routes::{coordinates, route_options},
};

pub(crate) enum Output {
    Json(Value),
    Image(Vec<u8>),
}

impl Output {
    pub fn into_json(self) -> Value {
        match self {
            Self::Json(value) => value,
            Self::Image(data) => json!({"mimeType":"image/png","base64":STANDARD.encode(data)}),
        }
    }
}

#[derive(Clone)]
pub(crate) struct EngineState {
    pub client: MapsClient,
    pub catalog: Catalog,
}

impl EngineState {
    pub async fn call(&self, name: &str, args: Value) -> Result<Output> {
        let definition = self.catalog.get(name)?;
        let start = std::time::Instant::now();
        let result = async {
            definition.validate(&args)?;
            let client = self.client.for_call();
            tokio::time::timeout(client.policy.tool_timeout, client.execute(name, &args))
                .await
                .map_err(|_| Error::ToolTimeout)?
        }
        .await;
        let elapsed = start.elapsed();
        self.client.metrics.tool(name, result.is_ok(), elapsed);
        tracing::info!(
            tool = name,
            success = result.is_ok(),
            duration_ms = elapsed.as_millis() as u64,
            "tool completed"
        );
        result
    }
}

impl MapsClient {
    async fn execute(&self, name: &str, a: &Value) -> Result<Output> {
        let value = match name {
            "maps_validate_address" => self.validate_address(a).await?,
            "maps_pollen" => self.pollen(a).await?,
            "maps_area_insights" => self.area_insights(a).await?,
            "maps_snap_to_roads" => self.roads(a, true).await?,
            "maps_nearest_roads" => self.roads(a, false).await?,
            "maps_geocode" => self.geocode(text(a, "address")).await?,
            "maps_reverse_geocode" => {
                let data = self
                    .get(
                        "maps.googleapis.com",
                        "/maps/api/geocode/json",
                        vec![
                            (
                                "latlng".into(),
                                format!("{},{}", a["latitude"], a["longitude"]),
                            ),
                            ("language".into(), self.language.to_string()),
                        ],
                    )
                    .await?;
                data["results"]
                    .as_array()
                    .and_then(|r| r.first())
                    .cloned()
                    .ok_or_else(|| Error::Api("no address found".into()))?
            }
            "maps_search_places" => {
                let mut body =
                    json!({"textQuery":a["query"],"languageCode":self.language,"pageSize":20});
                for key in ["openNow", "minRating", "includedType"] {
                    if let Some(v) = a.get(key) {
                        body[key] = v.clone();
                    }
                }
                if let Some(bias) = a.get("locationBias") {
                    body["locationBias"] = circle(
                        bias["latitude"].as_f64().ok_or(Error::Response)?,
                        bias["longitude"].as_f64().ok_or(Error::Response)?,
                        bias["radius"].as_f64().unwrap_or(5000.),
                    );
                }
                json!(self.search_text(&body).await?)
            }
            "maps_search_nearby" => json!(self.nearby(a).await?),
            "maps_place_details" => {
                self.details(
                    text(a, "placeId"),
                    a["maxPhotos"].as_u64().unwrap_or(0) as usize,
                )
                .await?
            }
            "maps_directions" => {
                route_options(a)?;
                let origin = self.resolve(text(a, "origin")).await?;
                let destination = self.resolve(text(a, "destination")).await?;
                self.compute_route(&[origin, destination], a, false).await?
            }
            "maps_distance_matrix" => {
                self.matrix(&strings(a, "origins"), &strings(a, "destinations"), a)
                    .await?
            }
            "maps_elevation" => {
                let locations = a["locations"]
                    .as_array()
                    .ok_or(Error::Response)?
                    .iter()
                    .map(|l| format!("{},{}", l["latitude"], l["longitude"]))
                    .collect::<Vec<_>>()
                    .join("|");
                self.get(
                    "maps.googleapis.com",
                    "/maps/api/elevation/json",
                    vec![("locations".into(), locations)],
                )
                .await?["results"]
                    .clone()
            }
            "maps_timezone" => {
                let timestamp = a["timestamp"]
                    .as_i64()
                    .unwrap_or_else(|| Utc::now().timestamp_millis())
                    / 1000;
                let mut data = self
                    .get(
                        "maps.googleapis.com",
                        "/maps/api/timezone/json",
                        vec![
                            (
                                "location".into(),
                                format!("{},{}", a["latitude"], a["longitude"]),
                            ),
                            ("timestamp".into(), timestamp.to_string()),
                            ("language".into(), self.language.to_string()),
                        ],
                    )
                    .await?;
                let offset = (data["rawOffset"].as_i64().unwrap_or(0)
                    + data["dstOffset"].as_i64().unwrap_or(0)) as i32;
                let offset = FixedOffset::east_opt(offset).ok_or(Error::Response)?;
                data["localTime"] = json!(
                    DateTime::from_timestamp(timestamp, 0)
                        .ok_or(Error::Response)?
                        .with_timezone(&offset)
                        .to_rfc3339()
                );
                data
            }
            "maps_weather" => {
                let (path, extra) = match a["type"].as_str().unwrap_or("current") {
                    "forecast_daily" => (
                        "/v1/forecast/days:lookup",
                        Some(("days", a["forecastDays"].as_u64().unwrap_or(5))),
                    ),
                    "forecast_hourly" => (
                        "/v1/forecast/hours:lookup",
                        Some(("hours", a["forecastHours"].as_u64().unwrap_or(24))),
                    ),
                    _ => ("/v1/currentConditions:lookup", None),
                };
                let mut query = vec![
                    ("location.latitude".into(), a["latitude"].to_string()),
                    ("location.longitude".into(), a["longitude"].to_string()),
                    ("languageCode".into(), self.language.to_string()),
                ];
                if let Some((key, value)) = extra {
                    query.push((key.into(), value.to_string()));
                }
                self.get("weather.googleapis.com", path, query).await?
            }
            "maps_air_quality" => {
                let mut extra = Vec::new();
                if a["includeHealthRecommendations"].as_bool().unwrap_or(true) {
                    extra.push("HEALTH_RECOMMENDATIONS");
                }
                if a["includePollutants"].as_bool().unwrap_or(false) {
                    extra.push("POLLUTANT_CONCENTRATION");
                }
                self.post("airquality.googleapis.com","/v1/currentConditions:lookup",&json!({"location":{"latitude":a["latitude"],"longitude":a["longitude"]},"extraComputations":extra,"languageCode":self.language}),None).await?
            }
            "maps_static_map" => {
                let size = a["size"].as_str().unwrap_or("600x400");
                let valid = size
                    .split_once('x')
                    .and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?)))
                    .is_some_and(|(w, h)| (1..=640).contains(&w) && (1..=640).contains(&h));
                if !valid {
                    return Err(Error::Input(
                        "static map size must be between 1x1 and 640x640".into(),
                    ));
                }
                if a.get("center").is_none()
                    && a.get("markers").is_none()
                    && a.get("path").is_none()
                {
                    return Err(Error::Input(
                        "static map requires center, markers or path".into(),
                    ));
                }
                let mut query = vec![
                    ("size".into(), size.into()),
                    ("format".into(), "png".into()),
                    (
                        "maptype".into(),
                        a["maptype"].as_str().unwrap_or("roadmap").into(),
                    ),
                ];
                for key in ["center", "zoom"] {
                    if let Some(v) = a.get(key) {
                        query.push((
                            key.into(),
                            v.as_str()
                                .map(str::to_owned)
                                .unwrap_or_else(|| v.to_string()),
                        ));
                    }
                }
                for key in ["markers", "path"] {
                    for entry in strings(a, key) {
                        query.push((key.into(), entry));
                    }
                }
                return Ok(Output::Image(self.static_map(query).await?));
            }
            "maps_batch_geocode" => self.batch_geocode(&strings(a, "addresses")).await,
            "maps_explore_area" => self.explore(a).await?,
            "maps_plan_route" => self.plan(a).await?,
            "maps_compare_places" => self.compare(a).await?,
            "maps_search_along_route" => self.along_route(a).await?,
            "maps_local_rank_tracker" => self.rank(a).await?,
            _ => return Err(Error::Input("unknown tool".into())),
        };
        Ok(Output::Json(value))
    }

    pub async fn nearby(&self, a: &Value) -> Result<Vec<Value>> {
        let center = &a["center"];
        let value = text(center, "value");
        let (lat, lng) = if center["isCoordinates"].as_bool().unwrap_or(false) {
            coordinates(value)?.ok_or_else(|| {
                Error::Input(
                    "center.value must be latitude,longitude when isCoordinates is true".into(),
                )
            })?
        } else {
            let resolved = self.resolve(value).await?;
            (resolved.latitude, resolved.longitude)
        };
        let mut body = json!({"locationRestriction":circle(lat,lng,a["radius"].as_f64().unwrap_or(1000.)),"maxResultCount":20,"languageCode":self.language});
        if let Some(kind) = a.get("keyword") {
            body["includedTypes"] = json!([kind]);
        }
        let data = self
            .post(
                "places.googleapis.com",
                "/v1/places:searchNearby",
                &body,
                Some(SEARCH_MASK_NEARBY),
            )
            .await?;
        let places = data["places"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|p| {
                (!a["openNow"].as_bool().unwrap_or(false)
                    || p["currentOpeningHours"]["openNow"].as_bool() == Some(true))
                    && (a.get("minRating").is_none()
                        || p["rating"].as_f64().unwrap_or(0.)
                            >= a["minRating"].as_f64().unwrap_or(0.))
            })
            .collect();
        Ok(places)
    }
}

pub(crate) fn strings(a: &Value, key: &str) -> Vec<String> {
    a[key]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
pub(crate) fn text<'a>(a: &'a Value, key: &str) -> &'a str {
    a[key].as_str().unwrap_or_default()
}
pub(crate) fn circle(latitude: f64, longitude: f64, radius: f64) -> Value {
    json!({"circle":{"center":{"latitude":latitude,"longitude":longitude},"radius":radius}})
}
const SEARCH_MASK_NEARBY: &str = "places.id,places.name,places.displayName,places.formattedAddress,places.location,places.primaryType,places.rating,places.userRatingCount,places.currentOpeningHours,places.priceLevel,places.googleMapsUri,places.attributions";
