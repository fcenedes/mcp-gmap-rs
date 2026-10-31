//! Roads speed limits, Places predictions, radio geolocation, Solar and Aerial View.
//!
//! These adapters retain Google's response metadata. Binary Solar downloads are
//! restricted to the documented GeoTIFF endpoint; Aerial View videos are never
//! downloaded because Google's service terms prohibit storing or caching them.

use serde_json::{Value, json};

use crate::{
    catalog::{array, boolean, coordinate, integer, number, object, string},
    client::{Error, MapsClient, Result},
    tools::Output,
};

fn choice(values: &[&str]) -> Value {
    json!({"type":"string","enum":values})
}

fn autocomplete_area(restrict: bool) -> Value {
    let mut radius = number(0., 50_000.);
    if restrict {
        radius["exclusiveMinimum"] = json!(0.);
    }
    let mut area = object(
        json!({
            "circle":object(json!({"center":coordinate(),"radius":radius}), &["center","radius"]),
            "rectangle":object(json!({"low":coordinate(),"high":coordinate()}), &["low","high"])
        }),
        &[],
    );
    area["oneOf"] = json!([{"required":["circle"]},{"required":["rectangle"]}]);
    area
}

pub(crate) fn definitions() -> Vec<(&'static str, &'static str, Value)> {
    let mut speeds = object(
        json!({"path":array(coordinate(),1,100),"roadPlaceIds":array(string(),1,100),"units":choice(&["KPH","MPH"])}),
        &[],
    );
    speeds["oneOf"] = json!([{"required":["path"]},{"required":["roadPlaceIds"]}]);
    let mut autocomplete = object(
        json!({
            "input":string(),
            "locationBias":autocomplete_area(false),
            "locationRestriction":autocomplete_area(true),
            "includedPrimaryTypes":array(string(),0,5),
            "includedRegionCodes":array(json!({"type":"string","pattern":"^[A-Za-z]{2}$"}),0,15),
            "languageCode":string(),
            "regionCode":{"type":"string","pattern":"^[A-Za-z]{2}$"},
            "origin":coordinate(),
            "inputOffset":integer(0,u32::MAX),
            "includeQueryPredictions":boolean(),
            "sessionToken":{"type":"string","minLength":1,"maxLength":36,"pattern":"^[A-Za-z0-9_-]+$"},
            "includePureServiceAreaBusinesses":boolean(),
            "includeFutureOpeningBusinesses":boolean()
        }),
        &["input"],
    );
    autocomplete["not"] = json!({"required":["locationBias","locationRestriction"]});

    // The native body remains extensible: Google's radio-specific cell fields
    // evolve independently of this executable. The wrapper itself is strict.
    let geolocate = object(
        json!({"body":{
            "type":"object","additionalProperties":true,
            "description":"Native Google Geolocation request body. All radio-specific fields are forwarded; see https://developers.google.com/maps/documentation/geolocation/requests-geolocation. An empty object allows Google's IP fallback. Set considerIp:false to prohibit it."
        }}),
        &["body"],
    );
    let mut solar = object(
        json!({
            "operation":choice(&["building_insights","data_layers","geo_tiff"]),
            "location":coordinate(),
            "requiredQuality":choice(&["HIGH","MEDIUM","BASE"]),
            "exactQualityRequired":boolean(),
            "experiments":array(choice(&["EXPANDED_COVERAGE"]),0,1),
            "additionalInsights":array(choice(&["DETECTED_ARRAYS"]),0,1),
            "radiusMeters":{"type":"number","exclusiveMinimum":0,"maximum":1000},
            "view":choice(&["DSM_LAYER","IMAGERY_LAYERS","IMAGERY_AND_ANNUAL_FLUX_LAYERS","IMAGERY_AND_ALL_FLUX_LAYERS","FULL_LAYERS"]),
            "pixelSizeMeters":{"type":"number","enum":[0.1,0.25,0.5,1.0]},
            "id":{"type":"string","minLength":1,"maxLength":8192}
        }),
        &["operation"],
    );
    solar["oneOf"] = json!([
        {"properties":{"operation":{"const":"building_insights"}},"required":["location"],"not":{"anyOf":[{"required":["id"]},{"required":["radiusMeters"]},{"required":["view"]},{"required":["pixelSizeMeters"]}]}},
        {"properties":{"operation":{"const":"data_layers"}},"required":["location","radiusMeters"],"not":{"anyOf":[{"required":["id"]},{"required":["additionalInsights"]}]}},
        {"properties":{"operation":{"const":"geo_tiff"}},"required":["id"],"not":{"anyOf":[{"required":["location"]},{"required":["requiredQuality"]},{"required":["exactQualityRequired"]},{"required":["experiments"]},{"required":["additionalInsights"]},{"required":["radiusMeters"]},{"required":["view"]},{"required":["pixelSizeMeters"]}]}}
    ]);
    let mut aerial = object(
        json!({"operation":choice(&["lookup","metadata"]),"address":string(),"videoId":string()}),
        &[],
    );
    aerial["oneOf"] = json!([{"required":["address"]},{"required":["videoId"]}]);
    vec![
        (
            "maps_speed_limits",
            "Get posted automobile speed limits for up to 100 GPS points or road-segment place IDs. Restricted Google Asset Tracking license/access is required; enabling Roads alone does not grant access. Each returned limit can incur a separate charge. Data is not real-time or guaranteed accurate.",
            speeds,
        ),
        (
            "maps_autocomplete",
            "Get Places API (New) place and optional query predictions, with session tokens, language and geographic filters. Returns the full native Google JSON. Session billing requires a fresh token and a qualifying concluding Details or Address Validation request in the same project.",
            autocomplete,
        ),
        (
            "maps_geolocate",
            "Estimate device location and an accuracy radius from a native Google Geolocation body containing cell towers or WiFi access points. Requires an API key. IP fallback identifies this server's egress location, not the user; set body.considerIp=false to disable fallback.",
            geolocate,
        ),
        (
            "maps_solar",
            "Retrieve Solar building insights, regional data layers, or a GeoTIFF asset by ID. Coverage, imagery quality and EEA terms vary; expanded-coverage experiments are pre-GA. Data-layer URLs are temporary and returned without credentials. GeoTIFF binary size is bounded by the executable's response limit.",
            solar,
        ),
        (
            "maps_aerial_view",
            "Look up Aerial View playback URLs or free metadata by US postal address or video ID. lookup is billable; metadata omits playback URLs. Coverage is limited to supported US locations. Returns Google's original metadata and short-lived URLs; videos must not be downloaded, stored or cached.",
            aerial,
        ),
        (
            "maps_render_aerial_video",
            "Explicitly request asynchronous rendering of an Aerial View video for a supported US postal address. This mutates Google's render queue; an existing video returns metadata. Does not wait for completion or download video. Poll maps_aerial_view with operation metadata and then lookup for playback URLs.",
            object(json!({"address":string()}), &["address"]),
        ),
    ]
}

pub(crate) async fn dispatch(
    client: &MapsClient,
    name: &str,
    args: &Value,
) -> Option<Result<Output>> {
    match name {
        "maps_speed_limits" => Some(speed_limits(client, args).await.map(Output::Json)),
        "maps_autocomplete" => Some(autocomplete(client, args).await.map(Output::Json)),
        "maps_geolocate" => Some(geolocate(client, args).await.map(Output::Json)),
        "maps_solar" => Some(solar(client, args).await),
        "maps_aerial_view" => Some(aerial_view(client, args).await.map(Output::Json)),
        "maps_render_aerial_video" => Some(render_aerial(client, args).await.map(Output::Json)),
        _ => None,
    }
}

fn required_text<'a>(args: &'a Value, field: &str) -> Result<&'a str> {
    args.get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| Error::Input(format!("{field} must be a non-empty string")))
}

async fn speed_limits(client: &MapsClient, args: &Value) -> Result<Value> {
    if args.get("path").is_some() == args.get("roadPlaceIds").is_some() {
        return Err(Error::Input("provide either path or roadPlaceIds".into()));
    }
    let mut query = Vec::new();
    if let Some(path) = args.get("path").and_then(Value::as_array) {
        if path.is_empty() || path.len() > 100 {
            return Err(Error::Input("path requires 1 to 100 coordinates".into()));
        }
        let points = path.iter().map(point).collect::<Result<Vec<_>>>()?;
        query.push(("path".into(), points.join("|")));
    } else {
        let ids = args["roadPlaceIds"]
            .as_array()
            .ok_or_else(|| Error::Input("roadPlaceIds must be an array".into()))?;
        if ids.is_empty() || ids.len() > 100 {
            return Err(Error::Input("roadPlaceIds requires 1 to 100 IDs".into()));
        }
        for id in ids {
            let id = id
                .as_str()
                .filter(|id| !id.trim().is_empty())
                .ok_or_else(|| Error::Input("road place IDs must be non-empty strings".into()))?;
            query.push(("placeId".into(), id.to_owned()));
        }
    }
    query.push((
        "units".into(),
        args["units"].as_str().unwrap_or("KPH").into(),
    ));
    client
        .get("roads.googleapis.com", "/v1/speedLimits", query)
        .await
        .map_err(|error| match error {
            Error::Api(message) if message.starts_with("HTTP 403") => Error::Api(format!(
                "{message}; speed limits have restricted Asset Tracking access: check your Google license/entitlement as well as API enablement, key restrictions and permissions"
            )),
            other => other,
        })
}

fn point(value: &Value) -> Result<String> {
    let latitude = value["latitude"]
        .as_f64()
        .filter(|n| (-90. ..=90.).contains(n));
    let longitude = value["longitude"]
        .as_f64()
        .filter(|n| (-180. ..=180.).contains(n));
    match (latitude, longitude) {
        (Some(lat), Some(lng)) => Ok(format!("{lat},{lng}")),
        _ => Err(Error::Input("invalid WGS84 coordinate".into())),
    }
}

async fn autocomplete(client: &MapsClient, args: &Value) -> Result<Value> {
    let input = required_text(args, "input")?;
    if args.get("locationBias").is_some() && args.get("locationRestriction").is_some() {
        return Err(Error::Input(
            "locationBias and locationRestriction are mutually exclusive".into(),
        ));
    }
    if args["inputOffset"]
        .as_u64()
        .is_some_and(|offset| offset > input.chars().count() as u64)
    {
        return Err(Error::Input(
            "inputOffset exceeds the input's Unicode character count".into(),
        ));
    }
    if let Some(types) = args["includedPrimaryTypes"].as_array()
        && types.len() > 1
        && types
            .iter()
            .any(|kind| matches!(kind.as_str(), Some("(cities)" | "(regions)")))
    {
        return Err(Error::Input(
            "(cities) or (regions) must be the only includedPrimaryTypes value".into(),
        ));
    }
    for name in ["locationBias", "locationRestriction"] {
        if let Some(area) = args.get(name) {
            if area.get("circle").is_some() == area.get("rectangle").is_some() {
                return Err(Error::Input(format!(
                    "{name} requires exactly one circle or rectangle"
                )));
            }
            if let Some(rectangle) = area.get("rectangle") {
                let low = rectangle["low"]["latitude"]
                    .as_f64()
                    .ok_or(Error::Response)?;
                let high = rectangle["high"]["latitude"]
                    .as_f64()
                    .ok_or(Error::Response)?;
                if low > high {
                    return Err(Error::Input(
                        "rectangle low latitude must not exceed high latitude".into(),
                    ));
                }
            }
        }
    }
    let mut body = args.clone();
    if body.get("languageCode").is_none() {
        body["languageCode"] = json!(client.language.as_ref());
    }
    client
        .post(
            "places.googleapis.com",
            "/v1/places:autocomplete",
            &body,
            None,
        )
        .await
}

async fn geolocate(client: &MapsClient, args: &Value) -> Result<Value> {
    let body = args
        .get("body")
        .filter(|body| body.is_object())
        .ok_or_else(|| Error::Input("body must be a native Geolocation request object".into()))?;
    // Geolocation's documented endpoint requires the key query parameter,
    // including for POST. call_json applies the selected host auth policy.
    client
        .call_json(
            "POST",
            "www.googleapis.com",
            "/geolocation/v1/geolocate",
            Vec::new(),
            Some(body),
            true,
        )
        .await
}

async fn solar(client: &MapsClient, args: &Value) -> Result<Output> {
    let operation = required_text(args, "operation")?;
    if operation == "geo_tiff" {
        let id = required_text(args, "id")?;
        if id.contains("://") || id.chars().any(char::is_control) {
            return Err(Error::Input(
                "id must be the GeoTIFF asset ID, not a URL".into(),
            ));
        }
        return client
            .download(
                "solar.googleapis.com",
                "/v1/geoTiff:get",
                vec![("id".into(), id.into())],
            )
            .await
            .map(Output::Binary);
    }
    let location = args.get("location").ok_or_else(|| {
        Error::Input("location is required for Solar insights or data layers".into())
    })?;
    point(location)?;
    let mut query = vec![
        ("location.latitude".into(), location["latitude"].to_string()),
        (
            "location.longitude".into(),
            location["longitude"].to_string(),
        ),
    ];
    for key in ["requiredQuality", "exactQualityRequired"] {
        push_scalar(&mut query, args, key);
    }
    for key in ["experiments", "additionalInsights"] {
        if let Some(values) = args[key].as_array() {
            for value in values {
                query.push((key.into(), value.as_str().ok_or(Error::Response)?.into()));
            }
        }
    }
    let path = match operation {
        "building_insights" => "/v1/buildingInsights:findClosest",
        "data_layers" => {
            let radius = args["radiusMeters"]
                .as_f64()
                .ok_or_else(|| Error::Input("radiusMeters is required for data layers".into()))?;
            let pixel = args["pixelSizeMeters"].as_f64().unwrap_or(0.1);
            if radius > 100. && radius > pixel * 1000. {
                return Err(Error::Input(
                    "radiusMeters over 100 must not exceed pixelSizeMeters × 1000".into(),
                ));
            }
            if radius > 175.
                && !matches!(
                    args["view"].as_str(),
                    Some("DSM_LAYER" | "IMAGERY_LAYERS" | "IMAGERY_AND_ANNUAL_FLUX_LAYERS")
                )
            {
                return Err(Error::Input(
                    "radii over 175m require a view without monthly flux or hourly shade".into(),
                ));
            }
            for key in ["radiusMeters", "view", "pixelSizeMeters"] {
                push_scalar(&mut query, args, key);
            }
            "/v1/dataLayers:get"
        }
        _ => return Err(Error::Input("unknown Solar operation".into())),
    };
    let mut data = client.get("solar.googleapis.com", path, query).await?;
    strip_url_credentials(&mut data);
    Ok(Output::Json(data))
}

fn push_scalar(query: &mut Vec<(String, String)>, args: &Value, key: &str) {
    if let Some(value) = args.get(key) {
        query.push((
            key.into(),
            value
                .as_str()
                .map_or_else(|| value.to_string(), str::to_owned),
        ));
    }
}

// Google's Solar data layer URLs normally contain only an opaque id. Strip
// credential query parameters defensively before surfacing any URL to agents.
fn strip_url_credentials(value: &mut Value) {
    match value {
        Value::String(text) => {
            if let Ok(mut url) = reqwest::Url::parse(text)
                && matches!(url.scheme(), "https" | "http")
            {
                let pairs: Vec<(String, String)> = url
                    .query_pairs()
                    .filter(|(key, _)| {
                        !matches!(
                            key.to_ascii_lowercase().as_str(),
                            "key" | "x-goog-api-key" | "access_token" | "oauth_token"
                        )
                    })
                    .map(|(key, value)| (key.into_owned(), value.into_owned()))
                    .collect();
                if url.query().is_some() {
                    url.set_query(None);
                    if !pairs.is_empty() {
                        url.query_pairs_mut().extend_pairs(pairs);
                    }
                }
                let _ = url.set_username("");
                let _ = url.set_password(None);
                *text = url.into();
            }
        }
        Value::Array(items) => items.iter_mut().for_each(strip_url_credentials),
        Value::Object(fields) => fields.values_mut().for_each(strip_url_credentials),
        _ => {}
    }
}

async fn aerial_view(client: &MapsClient, args: &Value) -> Result<Value> {
    if args.get("address").is_some() == args.get("videoId").is_some() {
        return Err(Error::Input("provide either address or videoId".into()));
    }
    let selector = if args.get("address").is_some() {
        "address"
    } else {
        "videoId"
    };
    let query = vec![(selector.into(), required_text(args, selector)?.into())];
    let path = match args["operation"].as_str().unwrap_or("lookup") {
        "lookup" => "/v1/videos:lookupVideo",
        "metadata" => "/v1/videos:lookupVideoMetadata",
        _ => return Err(Error::Input("unknown Aerial View operation".into())),
    };
    client.get("aerialview.googleapis.com", path, query).await
}

async fn render_aerial(client: &MapsClient, args: &Value) -> Result<Value> {
    let address = required_text(args, "address")?;
    client
        .call_json(
            "POST",
            "aerialview.googleapis.com",
            "/v1/videos:renderVideo",
            Vec::new(),
            Some(&json!({"address":address})),
            false,
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::Fixture;
    use axum::http::StatusCode;

    fn query(record: &crate::tests::Record) -> Vec<(String, String)> {
        reqwest::Url::parse(&format!("https://fixture.invalid/?{}", record.query))
            .unwrap()
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect()
    }

    #[tokio::test]
    async fn speed_limits_support_paths_repeated_ids_and_full_metadata() {
        let fixture = Fixture::new().await;
        let expected = json!({"speedLimits":[{"placeId":"road-a","speedLimit":50,"units":"KPH"}],"warningMessage":"Check signs","snappedPoints":[{"placeId":"road-a","originalIndex":0}]});
        fixture.override_response(
            "/roads.googleapis.com/v1/speedLimits",
            StatusCode::OK,
            expected.clone(),
        );
        assert_eq!(fixture.call("maps_speed_limits", json!({"path":[{"latitude":48.85,"longitude":2.35},{"latitude":48.86,"longitude":2.36}]})).await, expected);
        fixture
            .call(
                "maps_speed_limits",
                json!({"roadPlaceIds":["road-a","road-b"],"units":"MPH"}),
            )
            .await;
        let records = fixture.records();
        let first = query(&records[0]);
        assert!(first.contains(&("path".into(), "48.85,2.35|48.86,2.36".into())));
        assert!(first.contains(&("units".into(), "KPH".into())));
        let second = query(&records[1]);
        assert_eq!(
            second
                .iter()
                .filter(|(key, _)| key == "placeId")
                .map(|(_, value)| value.as_str())
                .collect::<Vec<_>>(),
            ["road-a", "road-b"]
        );
        assert!(second.contains(&("units".into(), "MPH".into())));
    }

    #[tokio::test]
    async fn restricted_speed_limit_errors_preserve_google_reason_without_secret() {
        let fixture = Fixture::new().await;
        fixture.override_response(
            "/roads.googleapis.com/v1/speedLimits",
            StatusCode::FORBIDDEN,
            json!({"error":{"message":"API key fixture-secret-key is restricted"}}),
        );
        let error = fixture
            .engine
            .call("maps_speed_limits", json!({"roadPlaceIds":["road-a"]}))
            .await
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("restricted"));
        assert!(error.contains("Asset Tracking"));
        assert!(error.contains("key restrictions"));
        assert!(!error.contains("fixture-secret-key"));
    }

    #[tokio::test]
    async fn autocomplete_preserves_predictions_and_forwards_session_options() {
        let fixture = Fixture::new().await;
        let expected = json!({"suggestions":[{"placePrediction":{"placeId":"p1","text":{"text":"Café","matches":[{"endOffset":3}]},"distanceMeters":120}},{"queryPrediction":{"text":{"text":"Cafés nearby"}}}],"attributions":[{"provider":"fixture"}]});
        fixture.override_response(
            "/places.googleapis.com/v1/places:autocomplete",
            StatusCode::OK,
            expected.clone(),
        );
        let args = json!({"input":"Café","inputOffset":4,"sessionToken":"550e8400-e29b-41d4-a716-446655440000","includeQueryPredictions":true,"includeFutureOpeningBusinesses":true,"includedPrimaryTypes":["cafe"],"includedRegionCodes":["FR"],"origin":{"latitude":48.85,"longitude":2.35},"locationBias":{"circle":{"center":{"latitude":48.85,"longitude":2.35},"radius":500}}});
        assert_eq!(
            fixture.call("maps_autocomplete", args.clone()).await,
            expected
        );
        let record = &fixture.records()[0];
        assert_eq!(record.body["inputOffset"], 4);
        assert_eq!(record.body["sessionToken"], args["sessionToken"]);
        assert_eq!(record.body["locationBias"], args["locationBias"]);
        assert_eq!(record.body["languageCode"], "en");
        assert!(record.headers.get("X-Goog-FieldMask").is_none());
    }

    #[tokio::test]
    async fn geolocation_forwards_native_radio_body_with_query_key() {
        let fixture = Fixture::new().await;
        let expected = json!({"location":{"lat":48.85,"lng":2.35},"accuracy":42.5});
        fixture.override_response(
            "/www.googleapis.com/geolocation/v1/geolocate",
            StatusCode::OK,
            expected.clone(),
        );
        let body = json!({"considerIp":false,"radioType":"nr","homeMobileCountryCode":208,"cellTowers":[{"newRadioCellId":68719476735_u64,"mobileCountryCode":208,"mobileNetworkCode":1}],"wifiAccessPoints":[{"macAddress":"00:25:9c:cf:1c:ac","signalStrength":-43},{"macAddress":"00:25:9c:cf:1c:ad","signalStrength":-55}]});
        assert_eq!(
            fixture.call("maps_geolocate", json!({"body":body})).await,
            expected
        );
        let record = &fixture.records()[0];
        assert_eq!(record.body, body);
        assert!(query(record).contains(&("key".into(), "fixture-secret-key".into())));
    }

    #[tokio::test]
    async fn solar_uses_documented_parameters_and_removes_embedded_url_credentials() {
        let fixture = Fixture::new().await;
        fixture.override_response("/solar.googleapis.com/v1/buildingInsights:findClosest", StatusCode::OK, json!({"name":"buildings/test","solarPotential":{"maxArrayPanelsCount":12},"detectedArrays":{"detectionStatus":"DETECTION_STATUS_ARRAYS_DETECTED"}}));
        fixture.call("maps_solar", json!({"operation":"building_insights","location":{"latitude":37.4,"longitude":-122.1},"requiredQuality":"BASE","exactQualityRequired":true,"experiments":["EXPANDED_COVERAGE"],"additionalInsights":["DETECTED_ARRAYS"]})).await;
        fixture.override_response("/solar.googleapis.com/v1/dataLayers:get", StatusCode::OK, json!({"dsmUrl":"https://solar.googleapis.com/v1/geoTiff:get?id=asset1&key=fixture-secret-key","hourlyShadeUrls":["https://solar.googleapis.com/v1/geoTiff:get?id=asset2&access_token=secret"],"imageryQuality":"HIGH"}));
        let result = fixture.call("maps_solar", json!({"operation":"data_layers","location":{"latitude":37.4,"longitude":-122.1},"radiusMeters":200,"pixelSizeMeters":0.25,"view":"IMAGERY_LAYERS"})).await;
        assert_eq!(
            result["dsmUrl"],
            "https://solar.googleapis.com/v1/geoTiff:get?id=asset1"
        );
        assert_eq!(
            result["hourlyShadeUrls"][0],
            "https://solar.googleapis.com/v1/geoTiff:get?id=asset2"
        );
        assert_eq!(result["imageryQuality"], "HIGH");
        let records = fixture.records();
        let first = query(&records[0]);
        assert!(first.contains(&("location.latitude".into(), "37.4".into())));
        assert!(first.contains(&("requiredQuality".into(), "BASE".into())));
        assert!(first.contains(&("additionalInsights".into(), "DETECTED_ARRAYS".into())));
        let second = query(&records[1]);
        assert!(second.contains(&("radiusMeters".into(), "200".into())));
        assert!(second.contains(&("view".into(), "IMAGERY_LAYERS".into())));
    }

    #[tokio::test]
    async fn solar_geotiff_download_preserves_bytes_mime_and_cache_metadata() {
        use axum::{Router, extract::State, http::Uri, routing::get};
        use std::sync::{Arc, Mutex};
        let seen = Arc::new(Mutex::new(Vec::<String>::new()));
        async fn tiff(
            State(seen): State<Arc<Mutex<Vec<String>>>>,
            uri: Uri,
        ) -> impl axum::response::IntoResponse {
            seen.lock().unwrap().push(uri.to_string());
            (
                [
                    ("content-type", "image/tiff"),
                    ("cache-control", "private, max-age=300"),
                ],
                b"II\x2a\x00fixture-tiff".to_vec(),
            )
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = Router::new()
            .route("/solar.googleapis.com/v1/geoTiff:get", get(tiff))
            .with_state(seen.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let mut fixture = Fixture::new().await;
        fixture.engine.client.test_base = Some(format!("http://{address}"));
        let output = fixture
            .engine
            .call(
                "maps_solar",
                json!({"operation":"geo_tiff","id":"opaque/asset+id"}),
            )
            .await
            .unwrap();
        task.abort();
        match output {
            Output::Binary(binary) => {
                assert_eq!(binary.data, b"II\x2a\x00fixture-tiff");
                assert_eq!(binary.mime_type, "image/tiff");
                assert_eq!(
                    binary.headers.get("cache-control").map(String::as_str),
                    Some("private, max-age=300")
                );
            }
            _ => panic!("GeoTIFF must remain a binary response"),
        }
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        let url = reqwest::Url::parse(&format!("http://fixture{}", seen[0])).unwrap();
        assert_eq!(url.path(), "/solar.googleapis.com/v1/geoTiff:get");
        assert!(
            url.query_pairs()
                .any(|(key, value)| key == "id" && value == "opaque/asset+id")
        );
    }

    #[tokio::test]
    async fn solar_oauth_does_not_fall_back_to_the_configured_api_key() {
        let mut fixture = Fixture::new().await;
        let _credentials = fixture.enable_oauth();
        fixture.override_response(
            "/solar.googleapis.com/v1/buildingInsights:findClosest",
            StatusCode::OK,
            json!({"name":"buildings/oauth-fixture"}),
        );
        fixture.call("maps_solar",json!({"operation":"building_insights","location":{"latitude":37.4,"longitude":-122.1}})).await;
        let records = fixture.records();
        let request = records
            .iter()
            .find(|record| record.path.starts_with("/solar.googleapis.com/"))
            .unwrap();
        assert_eq!(
            request.headers.get("authorization").unwrap(),
            "Bearer fixture-oauth-token"
        );
        assert!(request.headers.get("x-goog-api-key").is_none());
        assert!(!query(request).iter().any(|(key, _)| key == "key"));
    }

    #[tokio::test]
    async fn aerial_lookup_metadata_and_render_never_follow_media_urls() {
        let fixture = Fixture::new().await;
        let video = json!({"state":"ACTIVE","metadata":{"videoId":"video1"},"uris":{"MP4_HIGH":{"landscapeUri":"https://googlevideo.example/short-lived?sig=keep"}}});
        fixture.override_response(
            "/aerialview.googleapis.com/v1/videos:lookupVideo",
            StatusCode::OK,
            video.clone(),
        );
        fixture.override_response(
            "/aerialview.googleapis.com/v1/videos:lookupVideoMetadata",
            StatusCode::OK,
            json!({"videoId":"video1","state":"ACTIVE"}),
        );
        fixture.override_response(
            "/aerialview.googleapis.com/v1/videos:renderVideo",
            StatusCode::OK,
            json!({"state":"PROCESSING"}),
        );
        assert_eq!(
            fixture
                .call(
                    "maps_aerial_view",
                    json!({"address":"600 Montgomery St, San Francisco, CA 94111"})
                )
                .await,
            video
        );
        fixture
            .call(
                "maps_aerial_view",
                json!({"operation":"metadata","videoId":"video1"}),
            )
            .await;
        assert_eq!(
            fixture
                .call(
                    "maps_render_aerial_video",
                    json!({"address":"600 Montgomery St, San Francisco, CA 94111"})
                )
                .await["state"],
            "PROCESSING"
        );
        let records = fixture.records();
        assert_eq!(records.len(), 3);
        assert!(
            records
                .iter()
                .all(|record| record.path.starts_with("/aerialview.googleapis.com/"))
        );
        assert!(query(&records[1]).contains(&("videoId".into(), "video1".into())));
        assert_eq!(
            records[2].body["address"],
            "600 Montgomery St, San Francisco, CA 94111"
        );
    }

    #[tokio::test]
    async fn invalid_cross_field_inputs_fail_before_spending_google_quota() {
        let fixture = Fixture::new().await;
        let coordinate = json!({"latitude":1,"longitude":2});
        let cases = [
            ("maps_speed_limits", json!({})),
            (
                "maps_speed_limits",
                json!({"path":[coordinate.clone()],"roadPlaceIds":["road"]}),
            ),
            (
                "maps_speed_limits",
                json!({"roadPlaceIds":vec!["road";101]}),
            ),
            ("maps_autocomplete", json!({"input":"é","inputOffset":2})),
            (
                "maps_autocomplete",
                json!({"input":"x","includedPrimaryTypes":["(cities)","cafe"]}),
            ),
            (
                "maps_autocomplete",
                json!({"input":"x","locationBias":{"circle":{"center":coordinate.clone(),"radius":1}},"locationRestriction":{"circle":{"center":coordinate.clone(),"radius":1}}}),
            ),
            (
                "maps_autocomplete",
                json!({"input":"x","locationRestriction":{"circle":{"center":coordinate.clone(),"radius":0}}}),
            ),
            ("maps_geolocate", json!({"body":[],"arbitrary":true})),
            (
                "maps_solar",
                json!({"operation":"building_insights","id":"asset"}),
            ),
            (
                "maps_solar",
                json!({"operation":"geo_tiff","id":"https://attacker.invalid/secret"}),
            ),
            (
                "maps_solar",
                json!({"operation":"geo_tiff","id":"asset","location":coordinate.clone()}),
            ),
            (
                "maps_solar",
                json!({"operation":"data_layers","location":coordinate.clone(),"radiusMeters":101}),
            ),
            (
                "maps_solar",
                json!({"operation":"data_layers","location":coordinate.clone(),"radiusMeters":200,"pixelSizeMeters":0.25,"view":"FULL_LAYERS"}),
            ),
            (
                "maps_aerial_view",
                json!({"address":"US address","videoId":"video"}),
            ),
            ("maps_render_aerial_video", json!({"videoId":"video"})),
        ];
        for (name, args) in cases {
            assert!(fixture.engine.call(name, args).await.is_err(), "{name}");
        }
        assert!(fixture.records().is_empty());
    }

    #[tokio::test]
    async fn render_queue_failures_are_not_retried() {
        let mut fixture = Fixture::new().await;
        fixture.engine.client.policy.max_retries = 2;
        fixture.override_response(
            "/aerialview.googleapis.com/v1/videos:renderVideo",
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"error":{"message":"render queue unavailable"}}),
        );
        assert!(
            fixture
                .engine
                .call(
                    "maps_render_aerial_video",
                    json!({"address":"600 Montgomery St, San Francisco, CA 94111"})
                )
                .await
                .is_err()
        );
        assert_eq!(fixture.records().len(), 1);
    }
}
