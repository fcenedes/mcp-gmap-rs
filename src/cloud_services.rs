//! Cloud resource management and additional environmental Google Maps APIs.
//!
//! Native optimization payloads are deliberately forwarded intact: Google owns
//! the ShipmentModel schema and validates its constraints. Resource names and
//! operation selection are validated locally before any request is sent.

use chrono::DateTime;
use serde_json::{Value, json};

use crate::{
    catalog::{array, boolean, coordinate, integer, object, string},
    client::{Error, MapsClient, Result},
    tools::Output,
};

const DATASETS: &str = "mapsplatformdatasets.googleapis.com";
const ROUTE_OPTIMIZATION: &str = "routeoptimization.googleapis.com";
const AIR: &str = "airquality.googleapis.com";
const WEATHER: &str = "weather.googleapis.com";

fn choices(values: &[&str]) -> Value {
    json!({"type":"string","enum":values})
}

fn native(description: &str) -> Value {
    json!({"type":"object","additionalProperties":true,"description":description})
}

fn time_schema() -> Value {
    json!({"type":"string","minLength":20,"maxLength":30,"description":"RFC3339 UTC timestamp ending in Z, for example 2026-11-01T12:00:00Z."})
}

fn air_schema(history: bool) -> Value {
    let mut properties = json!({
        "location":coordinate(),
        "dateTime":time_schema(),
        "period":object(json!({"startTime":time_schema(),"endTime":time_schema()}), &["startTime","endTime"]),
        "pageSize":integer(1,if history {168} else {96}),
        "pageToken":string(),
        "universalAqi":boolean(),
        "languageCode":string(),
        "uaqiColorPalette":choices(&["RED_GREEN","INDIGO_PERSIAN_DARK","INDIGO_PERSIAN_LIGHT"]),
        "extraComputations":array(choices(&["LOCAL_AQI","HEALTH_RECOMMENDATIONS","POLLUTANT_ADDITIONAL_INFO","DOMINANT_POLLUTANT_CONCENTRATION","POLLUTANT_CONCENTRATION"]),1,5),
        "customLocalAqis":array(object(json!({"regionCode":json!({"type":"string","pattern":"^[A-Za-z]{2}$"}),"aqi":string()}), &["regionCode","aqi"]),1,250)
    });
    if history {
        properties["hours"] = integer(1, 720);
    }
    object(properties, &["location"])
}

fn tile_schema(map_types: &[&str]) -> Value {
    object(
        json!({"mapType":choices(map_types),"zoom":integer(0,16),"x":integer(0,65535),"y":integer(0,65535)}),
        &["mapType", "zoom", "x", "y"],
    )
}

fn weather_schema(kind: &str) -> Value {
    let mut properties = json!({"location":coordinate(),"pageSize":integer(1,if kind == "history" {24} else {1000}),"pageToken":string()});
    if kind != "minutes" {
        properties["languageCode"] = string();
    }
    if kind != "alerts" {
        properties["unitsSystem"] = choices(&["METRIC", "IMPERIAL"]);
    }
    if kind == "history" {
        properties["hours"] = integer(1, 24);
    }
    object(properties, &["location"])
}

/// Tool definitions consumed by the shared CLI and MCP catalog.
pub(crate) fn definitions() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "maps_datasets",
            "Manage Maps Platform datasets using OAuth/IAM: list, get/status, create, update, delete, import from GCS, download, or fetch errors. Writes change cloud resources; import is asynchronous. No version-list endpoint exists in v1.",
            object(
                json!({
                    "operation":choices(&["list","get","create","update","delete","import","download","fetch_errors"]),
                    "parent":json!({"type":"string","pattern":"^projects/[A-Za-z0-9][A-Za-z0-9._:-]*$"}),
                    "name":json!({"type":"string","pattern":"^projects/[A-Za-z0-9][A-Za-z0-9._:-]*/datasets/[A-Za-z0-9_-]+(@active)?$"}),
                    "dataset":native("Dataset metadata for create/update. Official schema: https://developers.google.com/maps/documentation/datasets/reference/rest/v1/projects.datasets . Google validates native fields."),
                    "gcsSource":object(json!({"inputUri":string(),"fileFormat":choices(&["FILE_FORMAT_GEOJSON","FILE_FORMAT_KML","FILE_FORMAT_CSV"])}), &["inputUri","fileFormat"]),
                    "versionDescription":string(),"updateMask":string(),"pageSize":integer(1,500),"pageToken":string(),"tag":choices(&["active"])
                }),
                &["operation"],
            ),
        ),
        (
            "maps_route_optimization",
            "Optimize or validate a native ShipmentModel with the Route Optimization API (OAuth/IAM). Explicit batch/long-running/URI operations create billable jobs; get_operation polls once. Long-running/URI variants are experimental; cancellation is unsupported by Google.",
            object(
                json!({
                    "operation":choices(&["optimize","validate","batch","optimize_long_running","optimize_uri","get_operation"]),
                    "parent":json!({"type":"string","pattern":"^projects/[A-Za-z0-9][A-Za-z0-9._:-]*(/locations/[A-Za-z0-9_-]+)?$"}),
                    "name":json!({"type":"string","pattern":"^projects/[A-Za-z0-9][A-Za-z0-9._:-]*(/locations/[A-Za-z0-9_-]+)?/operations/[A-Za-z0-9_-]+$"}),
                    "request":native("Complete native OptimizeToursRequest, BatchOptimizeToursRequest or OptimizeToursUriRequest body. Includes model.shipments, model.vehicles, costs, time windows and constraints without dropping fields. Google validates the full schema: https://developers.google.com/maps/documentation/route-optimization/reference/rest/v1/projects/optimizeTours . Parent belongs in the tool's parent argument, not in this body.")
                }),
                &["operation"],
            ),
        ),
        (
            "maps_air_quality_forecast",
            "Get one explicit page of hourly air quality forecasts, up to 96 hours ahead, with optional pollutants and health recommendations. Choose dateTime or period.",
            air_schema(false),
        ),
        (
            "maps_air_quality_history",
            "Get one explicit page of air quality history for the previous 30 days. Choose exactly one of dateTime, period or hours.",
            air_schema(true),
        ),
        (
            "maps_air_quality_heatmap",
            "Download a real 256-pixel air quality heatmap PNG tile. Preserve Google attribution and caching restrictions when displaying it.",
            tile_schema(&[
                "UAQI_RED_GREEN",
                "UAQI_INDIGO_PERSIAN",
                "PM25_INDIGO_PERSIAN",
                "GBR_DEFRA",
                "DEU_UBA",
                "CAN_EC",
                "FRA_ATMO",
                "US_AQI",
            ]),
        ),
        (
            "maps_pollen_heatmap",
            "Download a real pollen heatmap PNG tile (tree, grass or weed). Availability depends on supported countries, season and Google Maps content terms.",
            tile_schema(&["TREE_UPI", "GRASS_UPI", "WEED_UPI"]),
        ),
        (
            "maps_weather_history",
            "Get one explicit page of hourly weather history for the previous 24 hours.",
            weather_schema("history"),
        ),
        (
            "maps_weather_alerts",
            "Get one explicit page of public weather alerts whose affected area intersects a location. Preserve the issuing authority and alert attribution.",
            weather_schema("alerts"),
        ),
        (
            "maps_weather_minute_forecast",
            "EXPERIMENTAL/pre-GA: get one page of minute precipitation forecasts over six hours. Google requires attribution, limits caching to one hour, and prohibits recreating a primary-purpose weather app/model.",
            weather_schema("minutes"),
        ),
        (
            "maps_weather_map",
            "EXPERIMENTAL/pre-GA: download a real US or EU precipitation map PNG tile. Limited geographical coverage, Google attribution and a one-hour cache maximum apply; primary-purpose weather apps/models are prohibited.",
            tile_schema(&["US_PRECIPITATION_CURRENT", "EU_PRECIPITATION_CURRENT"]),
        ),
    ]
}

/// Dispatch only the names owned by this module.
pub(crate) async fn dispatch(
    client: &MapsClient,
    name: &str,
    args: &Value,
) -> Option<Result<Output>> {
    let result = match name {
        "maps_datasets" => datasets(client, args).await,
        "maps_route_optimization" => route_optimization(client, args).await.map(Output::Json),
        "maps_air_quality_forecast" => air_lookup(client, args, false).await.map(Output::Json),
        "maps_air_quality_history" => air_lookup(client, args, true).await.map(Output::Json),
        "maps_air_quality_heatmap" => tile(client, AIR, "heatmapTiles", args).await,
        "maps_pollen_heatmap" => tile(client, "pollen.googleapis.com", "heatmapTiles", args).await,
        "maps_weather_history" => weather_lookup(client, args, "history/hours")
            .await
            .map(Output::Json),
        "maps_weather_alerts" => weather_lookup(client, args, "publicAlerts")
            .await
            .map(Output::Json),
        "maps_weather_minute_forecast" => weather_lookup(client, args, "forecast/minutes")
            .await
            .map(Output::Json),
        "maps_weather_map" => tile(client, WEATHER, "mapTiles", args).await,
        _ => return None,
    };
    Some(result)
}

fn text<'a>(args: &'a Value, name: &str) -> Result<&'a str> {
    args.get(name)
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| Error::Input(format!("{name} is required and must be a nonempty string")))
}

fn input(message: &str) -> Error {
    Error::Input(message.into())
}

fn segment(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
}

fn project(s: &str) -> bool {
    !s.is_empty()
        && s.bytes().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._:-".contains(&c))
}

fn valid_parent(name: &str, location: bool) -> bool {
    let p: Vec<_> = name.split('/').collect();
    (p.len() == 2 || (location && p.len() == 4 && p[2] == "locations" && segment(p[3])))
        && p[0] == "projects"
        && project(p[1])
}

fn dataset_name(name: &str, active: bool) -> bool {
    let p: Vec<_> = name.split('/').collect();
    p.len() == 4
        && p[0] == "projects"
        && project(p[1])
        && p[2] == "datasets"
        && segment(if active {
            p[3].strip_suffix("@active").unwrap_or(p[3])
        } else {
            p[3]
        })
}

fn operation_name(name: &str) -> bool {
    name.rsplit_once("/operations/")
        .is_some_and(|(p, id)| valid_parent(p, true) && segment(id))
}

fn query_fields(args: &Value, names: &[&str]) -> Vec<(String, String)> {
    names
        .iter()
        .filter_map(|name| {
            args.get(*name).map(|v| {
                (
                    (*name).into(),
                    match v {
                        Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    },
                )
            })
        })
        .collect()
}

fn native_body<'a>(args: &'a Value, key: &str) -> Result<&'a Value> {
    args.get(key)
        .filter(|v| v.is_object())
        .ok_or_else(|| Error::Input(format!("{key} must be an object")))
}

fn gcs_uri(uri: &str) -> bool {
    uri.strip_prefix("gs://")
        .and_then(|s| s.split_once('/'))
        .is_some_and(|(bucket, object)| {
            !bucket.is_empty()
                && !object.is_empty()
                && !uri.chars().any(char::is_control)
                && !bucket.chars().any(char::is_whitespace)
                && !bucket.contains(['?', '#', '@'])
        })
}

async fn datasets(client: &MapsClient, args: &Value) -> Result<Output> {
    let operation = text(args, "operation")?;
    let mut query = query_fields(args, &["pageSize", "pageToken"]);
    let (method, path, body, safe) = match operation {
        "list" | "create" => {
            let parent = text(args, "parent")?;
            if !valid_parent(parent, false) {
                return Err(input("parent must be projects/{project}"));
            }
            if operation == "list" {
                query.extend(query_fields(args, &["tag"]));
                if args.get("pageSize").is_none() {
                    query.push(("pageSize".into(), "100".into()));
                }
                ("GET", format!("/v1/{parent}/datasets"), None, true)
            } else {
                let dataset = native_body(args, "dataset")?;
                text(dataset, "displayName")?;
                if dataset["usage"] != json!(["USAGE_DATA_DRIVEN_STYLING"]) {
                    return Err(input("dataset.usage must be [USAGE_DATA_DRIVEN_STYLING]"));
                }
                if dataset.get("name").is_some() {
                    return Err(input("omit dataset.name when creating a dataset"));
                }
                query.clear();
                (
                    "POST",
                    format!("/v1/{parent}/datasets"),
                    Some(dataset.clone()),
                    false,
                )
            }
        }
        "get" | "update" | "delete" | "import" | "download" | "fetch_errors" => {
            let name = text(args, "name")?;
            if !dataset_name(name, operation == "get") {
                return Err(input(
                    "name must be projects/{project}/datasets/{dataset}; @active is supported only for get",
                ));
            }
            let path = format!("/v1/{name}");
            match operation {
                "get" => {
                    query.clear();
                    ("GET", path, None, true)
                }
                "fetch_errors" => ("GET", format!("{path}:fetchDatasetErrors"), None, true),
                "download" => {
                    return client
                        .download(
                            DATASETS,
                            &format!("{path}:download"),
                            vec![("alt".into(), "media".into())],
                        )
                        .await
                        .map(Output::Binary);
                }
                "delete" => {
                    query.clear();
                    ("DELETE", path, None, false)
                }
                "update" => {
                    let mut dataset = native_body(args, "dataset")?.clone();
                    if dataset
                        .get("name")
                        .is_some_and(|v| v.as_str() != Some(name))
                    {
                        return Err(input("dataset.name must match name"));
                    }
                    dataset["name"] = json!(name);
                    query = query_fields(args, &["updateMask"]);
                    ("PATCH", path, Some(dataset), false)
                }
                _ => {
                    let source = native_body(args, "gcsSource")?;
                    if !gcs_uri(text(source, "inputUri")?) {
                        return Err(input("gcsSource.inputUri must be gs://bucket/object"));
                    }
                    query.clear();
                    let mut body = json!({"gcsSource":source});
                    if let Some(description) = args.get("versionDescription") {
                        body["versionDescription"] = description.clone();
                    }
                    ("POST", format!("{path}:import"), Some(body), false)
                }
            }
        }
        _ => return Err(input("unsupported dataset operation")),
    };
    client
        .call_json(method, DATASETS, &path, query, body.as_ref(), safe)
        .await
        .map(Output::Json)
}

fn validate_gcs_fields(value: &Value) -> Result<()> {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                if ["uri", "uriPrefix"].contains(&key.as_str())
                    && let Some(uri) = value.as_str()
                    && !gcs_uri(uri)
                {
                    return Err(input(
                        "optimization input/output URIs must be gs://bucket/object",
                    ));
                }
                validate_gcs_fields(value)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                validate_gcs_fields(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

async fn route_optimization(client: &MapsClient, args: &Value) -> Result<Value> {
    let operation = text(args, "operation")?;
    if operation == "get_operation" {
        let name = text(args, "name")?;
        if !operation_name(name) {
            return Err(input(
                "name must be projects/{project}[/locations/{location}]/operations/{operation}",
            ));
        }
        // Do not unwrap error/response: done:false and completed failures are useful status data.
        return client
            .call_json(
                "GET",
                ROUTE_OPTIMIZATION,
                &format!("/v1/{name}"),
                vec![],
                None,
                true,
            )
            .await;
    }
    let parent = text(args, "parent")?;
    if !valid_parent(parent, true) {
        return Err(input(
            "parent must be projects/{project} or projects/{project}/locations/{location}",
        ));
    }
    if matches!(operation, "optimize" | "validate" | "optimize_long_running")
        && parent.contains("/locations/")
        && !parent.ends_with("/locations/-")
    {
        return Err(input(
            "this optimization method supports only an omitted location or locations/-; regional location IDs are unsupported",
        ));
    }
    let mut body = native_body(args, "request")?.clone();
    if body.get("parent").is_some() {
        return Err(input("use the parent argument; omit request.parent"));
    }
    let (method, safe) = match operation {
        "optimize" | "validate" | "optimize_long_running" => {
            if !body["model"].is_object() {
                return Err(input("request.model must be a native ShipmentModel object"));
            }
            if operation == "validate" {
                body["solvingMode"] = json!("VALIDATE_ONLY");
            }
            if operation == "optimize_long_running" {
                ("optimizeToursLongRunning", false)
            } else {
                ("optimizeTours", true)
            }
        }
        "batch" => {
            let models = body["modelConfigs"]
                .as_array()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| input("request.modelConfigs must be a nonempty array"))?;
            if models.len() > 100 {
                return Err(input(
                    "at most 100 modelConfigs may be submitted in one call",
                ));
            }
            for model in models {
                if !model["inputConfig"].is_object() || !model["outputConfig"].is_object() {
                    return Err(input(
                        "each modelConfig requires inputConfig and outputConfig",
                    ));
                }
                for (config, source) in [
                    ("inputConfig", "gcsSource"),
                    ("outputConfig", "gcsDestination"),
                ] {
                    if !gcs_uri(text(&model[config][source], "uri")?) {
                        return Err(input(
                            "each batch model requires a GCS input and output URI",
                        ));
                    }
                    if !matches!(
                        model[config]["dataFormat"].as_str(),
                        Some("JSON" | "PROTO_TEXT")
                    ) {
                        return Err(input(
                            "batch inputConfig/outputConfig.dataFormat must be JSON or PROTO_TEXT",
                        ));
                    }
                }
            }
            validate_gcs_fields(&body)?;
            ("batchOptimizeTours", false)
        }
        "optimize_uri" => {
            for field in ["input", "output"] {
                let uri = text(&body[field], "uri")?;
                if !gcs_uri(uri) || !(uri.ends_with(".json") || uri.ends_with(".txtpb")) {
                    return Err(input(
                        "request.input.uri and request.output.uri must be gs://bucket/object ending in .json or .txtpb",
                    ));
                }
            }
            ("OptimizeToursUri", false)
        }
        _ => return Err(input("unsupported route optimization operation")),
    };
    client
        .call_json(
            "POST",
            ROUTE_OPTIMIZATION,
            &format!("/v1/{parent}:{method}"),
            vec![],
            Some(&body),
            safe,
        )
        .await
}

fn utc_time(value: &Value, field: &str) -> Result<chrono::DateTime<chrono::FixedOffset>> {
    let value = value
        .as_str()
        .ok_or_else(|| Error::Input(format!("{field} must be an RFC3339 UTC string")))?;
    if !value.ends_with('Z') {
        return Err(Error::Input(format!(
            "{field} must use RFC3339 UTC Z notation"
        )));
    }
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| Error::Input(format!("{field} must be a valid RFC3339 UTC timestamp")))
}

async fn air_lookup(client: &MapsClient, args: &Value, history: bool) -> Result<Value> {
    let count = ["dateTime", "period", "hours"]
        .iter()
        .filter(|k| args.get(**k).is_some())
        .count();
    if count != 1 {
        return Err(input(if history {
            "choose exactly one of dateTime, period or hours"
        } else {
            "choose exactly one of dateTime or period"
        }));
    }
    if let Some(time) = args.get("dateTime") {
        utc_time(time, "dateTime")?;
    }
    if let Some(period) = args.get("period") {
        let start = utc_time(&period["startTime"], "period.startTime")?;
        let end = utc_time(&period["endTime"], "period.endTime")?;
        let duration = end.signed_duration_since(start);
        if duration.num_seconds() < 0
            || duration > chrono::Duration::hours(if history { 720 } else { 96 })
        {
            return Err(input(
                "period must be ordered and within the API's maximum time range",
            ));
        }
    }
    client
        .call_json(
            "POST",
            AIR,
            if history {
                "/v1/history:lookup"
            } else {
                "/v1/forecast:lookup"
            },
            vec![],
            Some(args),
            true,
        )
        .await
}

async fn weather_lookup(client: &MapsClient, args: &Value, kind: &str) -> Result<Value> {
    let location = native_body(args, "location")?;
    let mut query = vec![
        ("location.latitude".into(), location["latitude"].to_string()),
        (
            "location.longitude".into(),
            location["longitude"].to_string(),
        ),
    ];
    query.extend(query_fields(
        args,
        &[
            "pageSize",
            "pageToken",
            "languageCode",
            "unitsSystem",
            "hours",
        ],
    ));
    client
        .get(WEATHER, &format!("/v1/{kind}:lookup"), query)
        .await
}

async fn tile(client: &MapsClient, host: &str, collection: &str, args: &Value) -> Result<Output> {
    let zoom = args["zoom"]
        .as_u64()
        .filter(|z| *z <= 16)
        .ok_or_else(|| input("zoom must be 0..16"))?;
    let max = (1u64 << zoom) - 1;
    let x = args["x"]
        .as_u64()
        .filter(|v| *v <= max)
        .ok_or_else(|| input("x must be 0..2^zoom-1"))?;
    let y = args["y"]
        .as_u64()
        .filter(|v| *v <= max)
        .ok_or_else(|| input("y must be 0..2^zoom-1"))?;
    let map_type = text(args, "mapType")?;
    client
        .download(
            host,
            &format!("/v1/mapTypes/{map_type}/{collection}/{zoom}/{x}/{y}"),
            vec![],
        )
        .await
        .map(Output::Binary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{catalog::Catalog, tests::Fixture, tools::EngineState};
    use axum::{Router, http::StatusCode, response::IntoResponse, routing::any};
    use std::time::Duration;

    #[tokio::test]
    async fn dataset_crud_and_gcs_import_preserve_native_resources_and_use_oauth() {
        let mut fixture = Fixture::new().await;
        let _credentials = fixture.enable_oauth();
        let base = "/mapsplatformdatasets.googleapis.com/v1/projects/fixture-project/datasets";
        fixture.override_response(
            base,
            StatusCode::OK,
            json!({"datasets":[],"nextPageToken":"dataset-page-2"}),
        );
        let response = fixture.call("maps_datasets", json!({"operation":"list","parent":"projects/fixture-project","pageToken":"dataset-page-1","tag":"active"})).await;
        assert_eq!(response["nextPageToken"], "dataset-page-2");
        fixture.call("maps_datasets", json!({"operation":"create","parent":"projects/fixture-project","dataset":{"displayName":"Delivery zones","usage":["USAGE_DATA_DRIVEN_STYLING"],"description":"native preserved"}})).await;
        let name = "projects/fixture-project/datasets/dataset-1";
        fixture.override_response(
            &format!("{base}/dataset-1"),
            StatusCode::OK,
            json!({"name":name,"status":{"state":"STATE_IMPORTING"},"versionId":"version-2"}),
        );
        let status = fixture
            .call("maps_datasets", json!({"operation":"get","name":name}))
            .await;
        assert_eq!(status["status"]["state"], "STATE_IMPORTING");
        fixture.call("maps_datasets", json!({"operation":"update","name":name,"dataset":{"description":"Updated"},"updateMask":"description"})).await;
        fixture.override_response(
            &format!("{base}/dataset-1:import"),
            StatusCode::OK,
            json!({"name":format!("{name}@version-3")}),
        );
        let imported = fixture.call("maps_datasets", json!({"operation":"import","name":name,"gcsSource":{"inputUri":"gs://fixture-bucket/zones.geojson","fileFormat":"FILE_FORMAT_GEOJSON"},"versionDescription":"Autumn zones"})).await;
        assert_eq!(imported["name"], format!("{name}@version-3"));
        fixture.override_response(
            &format!("{base}/dataset-1:fetchDatasetErrors"),
            StatusCode::OK,
            json!({"errors":[{"code":3,"message":"fixture row error"}],"nextPageToken":"errors-2"}),
        );
        let errors = fixture
            .call(
                "maps_datasets",
                json!({"operation":"fetch_errors","name":name,"pageSize":5,"pageToken":"errors-1"}),
            )
            .await;
        assert_eq!(errors["errors"][0]["code"], 3);
        fixture
            .call("maps_datasets", json!({"operation":"delete","name":name}))
            .await;
        let records: Vec<_> = fixture
            .records()
            .into_iter()
            .filter(|r| r.path.contains(DATASETS))
            .collect();
        assert_eq!(records.len(), 7);
        assert_eq!(
            records
                .iter()
                .map(|r| r.method.as_str())
                .collect::<Vec<_>>(),
            ["GET", "POST", "GET", "PATCH", "POST", "GET", "DELETE"]
        );
        assert!(records[0].query.contains("pageSize=100"));
        assert!(records[0].query.contains("pageToken=dataset-page-1"));
        assert_eq!(records[1].body["description"], "native preserved");
        assert!(records[3].query.contains("updateMask=description"));
        assert_eq!(records[3].body["name"], name);
        assert_eq!(records[4].body["versionDescription"], "Autumn zones");
        assert_eq!(
            records[4].body["gcsSource"]["inputUri"],
            "gs://fixture-bucket/zones.geojson"
        );
        assert!(records.iter().all(
            |r| r.headers["authorization"] == "Bearer fixture-oauth-token"
                && !r.query.contains("key=")
        ));
    }

    #[tokio::test]
    async fn optimization_forwards_full_models_and_returns_pending_and_failed_operations() {
        let mut fixture = Fixture::new().await;
        let _credentials = fixture.enable_oauth();
        let parent = "projects/fixture-project";
        let endpoint = "/routeoptimization.googleapis.com/v1/projects/fixture-project";
        fixture.override_response(
            &format!("{endpoint}:optimizeTours"),
            StatusCode::OK,
            json!({"routes":[{"vehicleLabel":"cargo-bike"}],"validationErrors":[]}),
        );
        let model = json!({"shipments":[{"label":"delivery","deliveries":[{"arrivalLocation":{"latitude":48.85,"longitude":2.35},"duration":"300s"}],"loadDemands":{"weight":{"amount":"8"}}}],"vehicles":[{"label":"cargo-bike","costPerHour":12,"loadLimits":{"weight":{"maxLoad":"10"}}}],"globalStartTime":"2026-11-01T08:00:00Z","globalEndTime":"2026-11-01T18:00:00Z"});
        let response = fixture.call("maps_route_optimization", json!({"operation":"validate","parent":parent,"request":{"model":model,"populatePolylines":true,"maxValidationErrors":10}})).await;
        assert_eq!(response["routes"][0]["vehicleLabel"], "cargo-bike");
        let name = "projects/fixture-project/locations/global/operations/operation-1";
        fixture.override_response(
            &format!("{endpoint}:batchOptimizeTours"),
            StatusCode::OK,
            json!({"name":name,"done":false}),
        );
        let request = json!({"modelConfigs":[{"displayName":"fixture-model","inputConfig":{"gcsSource":{"uri":"gs://fixture-bucket/input.json"},"dataFormat":"JSON"},"outputConfig":{"gcsDestination":{"uri":"gs://fixture-bucket/output.json"},"dataFormat":"JSON"}}]});
        let response = fixture
            .call(
                "maps_route_optimization",
                json!({"operation":"batch","parent":parent,"request":request}),
            )
            .await;
        assert_eq!(response["done"], false);
        let poll = format!("/routeoptimization.googleapis.com/v1/{name}");
        fixture.override_response(
            &poll,
            StatusCode::OK,
            json!({"name":name,"done":true,"error":{"code":3,"message":"fixture invalid model"}}),
        );
        let response = fixture
            .call(
                "maps_route_optimization",
                json!({"operation":"get_operation","name":name}),
            )
            .await;
        assert_eq!(response["error"]["code"], 3);
        assert_eq!(response["done"], true);
        let records: Vec<_> = fixture
            .records()
            .into_iter()
            .filter(|r| r.path.contains(ROUTE_OPTIMIZATION))
            .collect();
        assert_eq!(records.len(), 3, "one explicit poll; no background polling");
        assert_eq!(
            records
                .iter()
                .map(|r| r.method.as_str())
                .collect::<Vec<_>>(),
            ["POST", "POST", "GET"]
        );
        assert_eq!(records[0].body["model"], model);
        assert_eq!(records[0].body["solvingMode"], "VALIDATE_ONLY");
        assert_eq!(records[0].body["populatePolylines"], true);
        assert_eq!(records[1].body, request);
        assert!(
            records
                .iter()
                .all(|r| r.headers["authorization"] == "Bearer fixture-oauth-token")
        );
    }

    #[tokio::test]
    async fn explicit_experimental_optimization_jobs_use_the_documented_method_names() {
        let mut fixture = Fixture::new().await;
        let _credentials = fixture.enable_oauth();
        let prefix = "/routeoptimization.googleapis.com/v1/projects/fixture-project";
        for (operation, method, request) in [
            (
                "optimize_long_running",
                "optimizeToursLongRunning",
                json!({"model":{"shipments":[],"vehicles":[]},"timeout":"600s"}),
            ),
            (
                "optimize_uri",
                "OptimizeToursUri",
                json!({"input":{"uri":"gs://fixture-bucket/model.json"},"output":{"uri":"gs://fixture-bucket/result.json"}}),
            ),
        ] {
            fixture.override_response(
                &format!("{prefix}:{method}"),
                StatusCode::OK,
                json!({"name":"projects/fixture-project/operations/job-1","done":false}),
            );
            let response = fixture.call("maps_route_optimization",json!({"operation":operation,"parent":"projects/fixture-project","request":request})).await;
            assert_eq!(response["done"], false);
            let records = fixture.records();
            let record = records.last().unwrap();
            assert_eq!(record.path, format!("{prefix}:{method}"));
            assert_eq!(record.method, "POST");
            assert_eq!(record.body, request);
        }
    }

    #[tokio::test]
    async fn environmental_requests_keep_explicit_pagination_and_all_google_fields() {
        let fixture = Fixture::new().await;
        let location = json!({"latitude":48.85,"longitude":2.35});
        for (tool, path, args, body) in [
            (
                "maps_air_quality_history",
                "/airquality.googleapis.com/v1/history:lookup",
                json!({"location":location,"hours":48,"pageSize":12,"pageToken":"air-1","extraComputations":["HEALTH_RECOMMENDATIONS"],"customLocalAqis":[{"regionCode":"FR","aqi":"us_epa"}]}),
                json!({"hoursInfo":[{"dateTime":"2026-11-01T08:00:00Z","indexes":[{"aqi":80}]}],"nextPageToken":"air-2"}),
            ),
            (
                "maps_air_quality_forecast",
                "/airquality.googleapis.com/v1/forecast:lookup",
                json!({"location":location,"period":{"startTime":"2026-11-01T09:00:00Z","endTime":"2026-11-02T09:00:00Z"},"pageSize":6,"pageToken":"forecast-1"}),
                json!({"hourlyForecasts":[{"pollutants":[{"code":"pm25"}]}],"nextPageToken":"forecast-2"}),
            ),
            (
                "maps_weather_history",
                "/weather.googleapis.com/v1/history/hours:lookup",
                json!({"location":location,"hours":12,"pageSize":3,"pageToken":"weather-1","unitsSystem":"IMPERIAL","languageCode":"de"}),
                json!({"historyHours":[{"temperature":{"degrees":64}}],"nextPageToken":"weather-2"}),
            ),
            (
                "maps_weather_alerts",
                "/weather.googleapis.com/v1/publicAlerts:lookup",
                json!({"location":location,"pageSize":2,"pageToken":"alerts-1"}),
                json!({"weatherAlerts":[{"description":"fixture storm","issuingAuthority":{"name":"fixture authority"}}],"nextPageToken":"alerts-2"}),
            ),
            (
                "maps_weather_minute_forecast",
                "/weather.googleapis.com/v1/forecast/minutes:lookup",
                json!({"location":location,"pageSize":5,"pageToken":"minutes-1"}),
                json!({"segments":[{"probability":60}],"nextPageToken":"minutes-2"}),
            ),
        ] {
            fixture.override_response(path, StatusCode::OK, body.clone());
            assert_eq!(fixture.call(tool, args.clone()).await, body);
            let records = fixture.records();
            let record = records.last().unwrap();
            assert_eq!(record.path, path);
            if path.contains("airquality") {
                assert_eq!(record.body, args);
            } else {
                assert!(record.query.contains("location.latitude=48.85"));
                assert!(record.query.contains("pageToken="));
            }
        }
        assert_eq!(fixture.records().len(), 5, "no implicit next-page calls");
    }

    #[tokio::test]
    async fn validation_rejects_bad_resources_time_unions_and_tile_bounds_without_network() {
        let fixture = Fixture::new().await;
        for (tool, args) in [
            (
                "maps_datasets",
                json!({"operation":"get","name":"projects/p/datasets/../secret"}),
            ),
            (
                "maps_datasets",
                json!({"operation":"delete","name":"projects/p/datasets/d@active"}),
            ),
            (
                "maps_datasets",
                json!({"operation":"create","parent":"projects/p/locations/global","dataset":{}}),
            ),
            (
                "maps_datasets",
                json!({"operation":"import","name":"projects/p/datasets/d","gcsSource":{"inputUri":"https://example.org/dataset","fileFormat":"FILE_FORMAT_CSV"}}),
            ),
            (
                "maps_datasets",
                json!({"operation":"update","name":"projects/p/datasets/d","dataset":{"name":"projects/p/datasets/other"}}),
            ),
            (
                "maps_route_optimization",
                json!({"operation":"get_operation","name":"https://example.org/operations/o"}),
            ),
            (
                "maps_route_optimization",
                json!({"operation":"optimize","parent":"projects/p/locations/europe-west1","request":{"model":{}}}),
            ),
            (
                "maps_route_optimization",
                json!({"operation":"batch","parent":"projects/p","request":{"modelConfigs":[]}}),
            ),
            (
                "maps_route_optimization",
                json!({"operation":"optimize_uri","parent":"projects/p","request":{"input":{"uri":"https://example.org/input"},"output":{"uri":"gs://bucket/output.json"}}}),
            ),
            (
                "maps_air_quality_history",
                json!({"location":{"latitude":0,"longitude":0},"hours":24,"dateTime":"2026-11-01T00:00:00Z"}),
            ),
            (
                "maps_air_quality_forecast",
                json!({"location":{"latitude":0,"longitude":0},"dateTime":"2026-11-01T00:00:00+02:00"}),
            ),
            (
                "maps_air_quality_forecast",
                json!({"location":{"latitude":0,"longitude":0},"dateTime":"2026-02-30T00:00:00Z"}),
            ),
            (
                "maps_air_quality_forecast",
                json!({"location":{"latitude":0,"longitude":0},"period":{"startTime":"2026-11-02T00:00:00Z","endTime":"2026-11-01T00:00:00Z"}}),
            ),
            (
                "maps_air_quality_heatmap",
                json!({"mapType":"US_AQI","zoom":2,"x":4,"y":0}),
            ),
            (
                "maps_pollen_heatmap",
                json!({"mapType":"TREE_UPI","zoom":0,"x":0,"y":1}),
            ),
            (
                "maps_weather_history",
                json!({"location":{"latitude":91,"longitude":0}}),
            ),
            (
                "maps_weather_map",
                json!({"mapType":"US_PRECIPITATION_CURRENT","zoom":17,"x":0,"y":0}),
            ),
        ] {
            assert!(fixture.engine.call(tool, args).await.is_err(), "{tool}");
        }
        assert!(fixture.records().is_empty());
    }

    #[tokio::test]
    async fn tile_tools_return_real_binary_with_mime_and_cache_headers() {
        let app = Router::new().fallback(any(|uri: axum::http::Uri| async move {
            assert!(uri.path().contains("/v1/mapTypes/"));
            assert!(uri.path().ends_with("/2/1/2"));
            (
                [
                    ("content-type", "image/png"),
                    ("cache-control", "private, max-age=600"),
                ],
                b"\x89PNG\r\n\x1a\nfixture-tile".to_vec(),
            )
                .into_response()
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let mut client = MapsClient::new(
            Some("fixture-key".into()),
            Duration::from_secs(2),
            "en".into(),
        )
        .unwrap();
        client.test_base = Some(format!("http://{address}"));
        let engine = EngineState {
            client,
            catalog: Catalog::new(None).unwrap(),
        };
        for (name, map_type) in [
            ("maps_air_quality_heatmap", "US_AQI"),
            ("maps_pollen_heatmap", "TREE_UPI"),
            ("maps_weather_map", "EU_PRECIPITATION_CURRENT"),
        ] {
            let output = engine
                .call(name, json!({"mapType":map_type,"zoom":2,"x":1,"y":2}))
                .await
                .unwrap();
            let Output::Binary(binary) = output else {
                panic!("tile must be binary")
            };
            assert_eq!(binary.mime_type, "image/png");
            assert_eq!(binary.data, b"\x89PNG\r\n\x1a\nfixture-tile");
            assert_eq!(binary.headers["cache-control"], "private, max-age=600");
        }
        task.abort();
    }

    #[tokio::test]
    async fn cloud_permissions_errors_are_preserved_and_secrets_redacted() {
        let mut fixture = Fixture::new().await;
        let _credentials = fixture.enable_oauth();
        fixture.override_response("/mapsplatformdatasets.googleapis.com/v1/projects/fixture-project/datasets",StatusCode::FORBIDDEN,json!({"error":{"message":"mapsplatformdatasets.datasets.list denied fixture-oauth-token"}}));
        let error = fixture
            .engine
            .call(
                "maps_datasets",
                json!({"operation":"list","parent":"projects/fixture-project"}),
            )
            .await
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("mapsplatformdatasets.datasets.list"));
        assert!(!error.contains("fixture-oauth-token"));
    }
}
