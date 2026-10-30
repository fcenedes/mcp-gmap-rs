use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::any,
};
use serde_json::{Value, json};
use tokio::{net::TcpListener, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    catalog::Catalog,
    client::{Error, MapsClient},
    server::{HttpOptions, http_router},
    tools::{EngineState, Output},
};

#[derive(Clone, Debug)]
struct Record {
    path: String,
    query: String,
    headers: HeaderMap,
    body: Value,
}

type Overrides = HashMap<String, (StatusCode, Value)>;

#[derive(Clone, Default)]
struct FixtureState {
    records: Arc<Mutex<Vec<Record>>>,
    overrides: Arc<Mutex<Overrides>>,
}

struct Fixture {
    engine: EngineState,
    state: FixtureState,
    task: JoinHandle<()>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Fixture {
    async fn new() -> Self {
        let state = FixtureState::default();
        let app = Router::new()
            .fallback(any(google))
            .with_state(state.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let mut client = MapsClient::new(
            Some("fixture-secret-key".into()),
            Duration::from_secs(2),
            "en".into(),
        )
        .unwrap();
        client.test_base = Some(format!("http://{address}"));
        Self {
            engine: EngineState {
                client,
                catalog: Catalog::new(None).unwrap(),
            },
            state,
            task,
        }
    }

    fn records(&self) -> Vec<Record> {
        self.state.records.lock().unwrap().clone()
    }
    fn override_response(&self, path: &str, status: StatusCode, value: Value) {
        self.state
            .overrides
            .lock()
            .unwrap()
            .insert(path.into(), (status, value));
    }
    async fn call(&self, name: &str, args: Value) -> Value {
        self.engine.call(name, args).await.unwrap().into_json()
    }
}

fn place() -> Value {
    json!({"id":"p1","name":"places/p1","displayName":{"text":"Fixture Cafe"},"formattedAddress":"1 Test Street","location":{"latitude":48.85,"longitude":2.35},"rating":4.7,"currentOpeningHours":{"openNow":true},"googleMapsUri":"https://maps.google.com/?cid=1","reviews":[{"text":{"text":"Great"},"authorAttribution":{"displayName":"Author","uri":"https://example.org/author"},"googleMapsUri":"https://maps.google.com/review/1"}],"photos":[{"name":"places/p1/photos/photo1","authorAttributions":[{"displayName":"Photographer"}],"googleMapsUri":"https://maps.google.com/photo/1"}],"reviewSummary":{"text":{"text":"Summary"},"disclosureText":{"text":"Summarized with AI"},"reviewsUri":"https://maps.google.com/reviews/1"}})
}

async fn google(
    State(state): State<FixtureState>,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    let path = uri.path();
    state.records.lock().unwrap().push(Record {
        path: path.into(),
        query: uri.query().unwrap_or_default().into(),
        headers,
        body: body.clone(),
    });
    let override_response = state.overrides.lock().unwrap().get(path).cloned();
    if let Some((status, value)) = override_response {
        return (status, axum::Json(value)).into_response();
    }
    let value=match path {
        "/maps.googleapis.com/maps/api/geocode/json"=>{
            let query=uri.query().unwrap_or_default();
            if query.contains("fallback") || query.contains("missing") {json!({"status":"ZERO_RESULTS","results":[]})}
            else {json!({"status":"OK","results":[{"place_id":"p1","formatted_address":"1 Test Street","geometry":{"location":{"lat":48.85,"lng":2.35}},"address_components":[{"long_name":"Test Street"}]}]})}
        }
        "/places.googleapis.com/v1/places:searchText"=>{
            if body["textQuery"]=="missing" {json!({"places":[]})} else {json!({"places":[place()],"routingSummaries":[{"legs":[{"duration":"30s","distanceMeters":100}]}]})}
        }
        "/places.googleapis.com/v1/places:searchNearby"=>json!({"places":[place(),{"id":"closed","rating":2,"currentOpeningHours":{"openNow":false}}]}),
        "/places.googleapis.com/v1/places/p1"=>place(),
        "/places.googleapis.com/v1/places/p1/photos/photo1/media"=>json!({"photoUri":"https://example.org/photo.png"}),
        "/routes.googleapis.com/directions/v2:computeRoutes"=>{
            let intermediates=body["intermediates"].as_array().map_or(0,Vec::len);
            let mut route=json!({"distanceMeters":1000,"duration":"120.5s","description":"Fixture route","polyline":{"encodedPolyline":"abc"},"legs":(0..=intermediates).map(|_|json!({"distanceMeters":500,"duration":"60s","steps":[{"navigationInstruction":{"instructions":"Continue"}}]})).collect::<Vec<_>>()});
            if body["optimizeWaypointOrder"]==true {route["optimizedIntermediateWaypointIndex"]=json!((0..intermediates).rev().collect::<Vec<_>>());}
            json!({"routes":[route]})
        }
        "/routes.googleapis.com/distanceMatrix/v2:computeRouteMatrix"=>json!(body["origins"].as_array().unwrap().iter().enumerate().flat_map(|(i,_)|body["destinations"].as_array().unwrap().iter().enumerate().map(move |(j,_)|json!({"originIndex":i,"destinationIndex":j,"distanceMeters":1000,"duration":"120s","condition":"ROUTE_EXISTS","status":{}}))).collect::<Vec<_>>()),
        "/maps.googleapis.com/maps/api/elevation/json"=>json!({"status":"OK","results":[{"elevation":35,"location":{"lat":48.85,"lng":2.35}}]}),
        "/maps.googleapis.com/maps/api/timezone/json"=>json!({"status":"OK","timeZoneId":"Europe/Paris","timeZoneName":"Central European Time","rawOffset":3600,"dstOffset":0}),
        "/weather.googleapis.com/v1/currentConditions:lookup"=>json!({"temperature":{"degrees":18},"weatherCondition":{"description":{"text":"Sunny"}}}),
        "/weather.googleapis.com/v1/forecast/days:lookup"=>json!({"forecastDays":[{"displayDate":{"year":2026,"month":10,"day":30}}]}),
        "/weather.googleapis.com/v1/forecast/hours:lookup"=>json!({"forecastHours":[{"temperature":{"degrees":18}}]}),
        "/airquality.googleapis.com/v1/currentConditions:lookup"=>json!({"indexes":[{"aqi":80}],"healthRecommendations":{"generalPopulation":"Enjoy outside"}}),
        "/maps.googleapis.com/maps/api/staticmap"=>return ([("content-type","image/png")],b"\x89PNG\r\n\x1a\nfixture".to_vec()).into_response(),
        _=>return (StatusCode::NOT_FOUND,axum::Json(json!({"error":{"message":format!("unhandled fixture {path}")}}))).into_response(),
    };
    axum::Json(value).into_response()
}

#[test]
fn catalog_has_all_18_tools_and_rejects_bad_filters() {
    let catalog = Catalog::new(None).unwrap();
    assert_eq!(catalog.iter().count(), 18);
    assert_eq!(
        Catalog::new(Some("maps_geocode,maps_weather"))
            .unwrap()
            .iter()
            .count(),
        2
    );
    assert!(Catalog::new(Some("maps_typo")).is_err());
    assert!(Catalog::new(Some(",,")).is_err());
}

#[tokio::test]
async fn invalid_inputs_never_reach_google() {
    let fixture = Fixture::new().await;
    for (name, args) in [
        ("maps_geocode", json!({})),
        ("maps_geocode", json!({"address":""})),
        ("maps_geocode", json!({"address":"Paris","apiKey":"hidden"})),
        ("maps_reverse_geocode", json!({"latitude":91,"longitude":1})),
        ("maps_batch_geocode", json!({"addresses":[]})),
        ("maps_plan_route", json!({"stops":["one"]})),
        (
            "maps_local_rank_tracker",
            json!({"placeId":"p1","center":{"latitude":1,"longitude":2}}),
        ),
        (
            "maps_weather",
            json!({"latitude":1,"longitude":2,"forecastHours":241}),
        ),
        (
            "maps_static_map",
            json!({"center":"Paris","size":"999x999"}),
        ),
        (
            "maps_directions",
            json!({"origin":"Paris","destination":"London","mode":"walking","avoid_tolls":true}),
        ),
        (
            "maps_directions",
            json!({"origin":"Paris","destination":"London","departure_time":"invalid"}),
        ),
        (
            "maps_plan_route",
            json!({"stops":["A","B","C"],"mode":"transit"}),
        ),
        (
            "maps_search_nearby",
            json!({"center":{"value":"1,2junk","isCoordinates":true}}),
        ),
    ] {
        assert!(
            fixture.engine.call(name, args).await.is_err(),
            "{name} should fail"
        );
    }
    assert!(fixture.records().is_empty());
}

#[tokio::test]
async fn geocode_reverse_geocode_and_batch_work() {
    let f = Fixture::new().await;
    assert_eq!(
        f.call("maps_geocode", json!({"address":"Paris & cafes"}))
            .await["location"]["lat"],
        48.85
    );
    assert_eq!(
        f.call(
            "maps_reverse_geocode",
            json!({"latitude":48.85,"longitude":2.35})
        )
        .await["address_components"][0]["long_name"],
        "Test Street"
    );
    let batch = f
        .call(
            "maps_batch_geocode",
            json!({"addresses":["Paris","missing","London"]}),
        )
        .await;
    assert_eq!(batch[0]["success"], true);
    assert_eq!(batch[1]["success"], false);
    assert_eq!(batch[2]["address"], "London");
    assert!(f.records()[0].query.contains("address=Paris+%26+cafes"));
}

#[tokio::test]
async fn places_filters_and_attribution_are_preserved() {
    let f = Fixture::new().await;
    f.call("maps_search_places",json!({"query":"cafe","locationBias":{"latitude":48.85,"longitude":2.35},"openNow":true,"minRating":4,"includedType":"cafe"})).await;
    let records = f.records();
    assert_eq!(records[0].body["locationBias"]["circle"]["radius"], 5000.0);
    assert_eq!(records[0].headers["x-goog-api-key"], "fixture-secret-key");
    assert!(
        records[0].headers["x-goog-fieldmask"]
            .to_str()
            .unwrap()
            .contains("places.attributions")
    );
    let nearby=f.call("maps_search_nearby",json!({"center":{"value":"48.85,2.35","isCoordinates":true},"openNow":true,"minRating":4})).await;
    assert_eq!(nearby.as_array().unwrap().len(), 1);
    let place = f
        .call("maps_place_details", json!({"placeId":"p1","maxPhotos":1}))
        .await;
    assert_eq!(
        place["reviews"][0]["authorAttribution"]["displayName"],
        "Author"
    );
    assert_eq!(
        place["reviewSummary"]["disclosureText"]["text"],
        "Summarized with AI"
    );
    assert_eq!(
        place["photoUrls"][0]["authorAttributions"][0]["displayName"],
        "Photographer"
    );
}

#[tokio::test]
async fn exact_coordinates_and_landmark_fallback_use_stable_waypoints() {
    let f = Fixture::new().await;
    f.call(
        "maps_directions",
        json!({"origin":"48.1,2.1","destination":"fallback"}),
    )
    .await;
    let records = f.records();
    assert_eq!(
        records
            .iter()
            .filter(|r| r.path.contains("geocode"))
            .count(),
        1
    );
    let route = &records
        .iter()
        .find(|r| r.path.contains("computeRoutes"))
        .unwrap()
        .body;
    assert_eq!(
        route["origin"],
        json!({"location":{"latLng":{"latitude":48.1,"longitude":2.1}}})
    );
    assert_eq!(route["destination"], json!({"placeId":"p1"}));
    assert!(f.engine.client.resolve("91,1").await.is_err());
    assert!(f.engine.client.resolve("missing").await.is_err());
}

#[tokio::test]
async fn route_optimization_reorders_only_intermediates() {
    let f = Fixture::new().await;
    let result = f
        .call(
            "maps_plan_route",
            json!({"stops":["1,1","2,2","3,3","4,4"]}),
        )
        .await;
    let names = result["stops"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["originalName"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(names, ["1,1", "3,3", "2,2", "4,4"]);
    assert_eq!(result["legs"].as_array().unwrap().len(), 3);
    assert_eq!(result["polyline"]["encodedPolyline"], "abc");
    assert_eq!(f.records().len(), 1);
    let ordinary = f
        .call("maps_plan_route", json!({"stops":["1,1","2,2","3,3"]}))
        .await;
    assert_eq!(ordinary["optimized"], false);
    assert!(f.records()[1].body.get("optimizeWaypointOrder").is_none());
}

#[tokio::test]
async fn bad_optimization_indices_fail_instead_of_corrupting_stops() {
    let f = Fixture::new().await;
    f.override_response(
        "/routes.googleapis.com/directions/v2:computeRoutes",
        StatusCode::OK,
        json!({"routes":[{"optimizedIntermediateWaypointIndex":[0,0]}]}),
    );
    assert!(
        f.engine
            .call(
                "maps_plan_route",
                json!({"stops":["1,1","2,2","3,3","4,4"]})
            )
            .await
            .is_err()
    );
}

#[tokio::test]
async fn matrix_puts_modifiers_on_origins_and_keeps_element_errors() {
    let f = Fixture::new().await;
    f.override_response("/routes.googleapis.com/distanceMatrix/v2:computeRouteMatrix",StatusCode::OK,json!([
        {"distanceMeters":1000,"duration":"120.5s","condition":"ROUTE_EXISTS"},
        {"destinationIndex":1,"status":{"code":7,"message":"denied"},"condition":"ROUTE_NOT_FOUND"}
    ]));
    let data=f.call("maps_distance_matrix",json!({"origins":["Paris"],"destinations":["London","Rome","Madrid"],"avoid_tolls":true})).await;
    let request = &f.records()[0].body;
    assert!(request.get("routeModifiers").is_none());
    assert_eq!(request["origins"][0]["routeModifiers"]["avoidTolls"], true);
    assert_eq!(data["durations"][0][0]["value"], 120.5);
    assert_eq!(data["distances"][0][1], Value::Null);
    assert_eq!(data["errors"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn elevation_timezone_weather_and_air_quality_work() {
    let f = Fixture::new().await;
    assert_eq!(
        f.call(
            "maps_elevation",
            json!({"locations":[{"latitude":48.85,"longitude":2.35}]})
        )
        .await[0]["elevation"],
        35
    );
    assert_eq!(
        f.call(
            "maps_timezone",
            json!({"latitude":48.85,"longitude":2.35,"timestamp":0})
        )
        .await["localTime"],
        "1970-01-01T01:00:00+01:00"
    );
    assert_eq!(
        f.call("maps_weather", json!({"latitude":48.85,"longitude":2.35}))
            .await["temperature"]["degrees"],
        18
    );
    assert!(
        f.call(
            "maps_weather",
            json!({"latitude":48.85,"longitude":2.35,"type":"forecast_daily","forecastDays":10})
        )
        .await["forecastDays"]
            .is_array()
    );
    assert!(
        f.call(
            "maps_weather",
            json!({"latitude":48.85,"longitude":2.35,"type":"forecast_hourly","forecastHours":240})
        )
        .await["forecastHours"]
            .is_array()
    );
    assert_eq!(
        f.call(
            "maps_air_quality",
            json!({"latitude":48.85,"longitude":2.35,"includePollutants":true})
        )
        .await["indexes"][0]["aqi"],
        80
    );
    assert_eq!(
        f.records().last().unwrap().body["extraComputations"],
        json!(["HEALTH_RECOMMENDATIONS", "POLLUTANT_CONCENTRATION"])
    );
}

#[tokio::test]
async fn static_map_returns_png_with_repeated_markers() {
    let f = Fixture::new().await;
    let output = f
        .engine
        .call(
            "maps_static_map",
            json!({"center":"Paris","markers":["color:red|1,1","color:blue|2,2"],"size":"640x640"}),
        )
        .await
        .unwrap();
    assert!(matches!(output,Output::Image(ref bytes) if bytes.starts_with(b"\x89PNG")));
    assert_eq!(f.records()[0].query.matches("markers=").count(), 2);
}

#[tokio::test]
async fn composites_explore_compare_along_route_and_rank_work() {
    let f = Fixture::new().await;
    let area = f
        .call(
            "maps_explore_area",
            json!({"location":"48.85,2.35","types":["cafe"],"topN":1}),
        )
        .await;
    assert_eq!(area["categories"][0]["top"][0]["details"]["id"], "p1");
    let compare = f
        .call(
            "maps_compare_places",
            json!({"query":"cafes","userLocation":{"latitude":48.8,"longitude":2.3}}),
        )
        .await;
    assert_eq!(compare["places"][0]["distance"]["value"], 1000.0);
    let along = f
        .call(
            "maps_search_along_route",
            json!({"origin":"1,1","destination":"2,2","textQuery":"cafe"}),
        )
        .await;
    assert_eq!(along["places"][0]["id"], "p1");
    assert!(
        f.records()
            .iter()
            .any(|r| r.body["searchAlongRouteParameters"]["polyline"]["encodedPolyline"] == "abc")
    );
    let rank=f.call("maps_local_rank_tracker",json!({"keyword":"cafe","placeId":"p1","center":{"latitude":89.999,"longitude":179.999},"gridSize":3})).await;
    assert_eq!(rank["keywords"][0]["metrics"]["arp"], 1.0);
    assert_eq!(rank["keywords"][0]["metrics"]["solv"], 100.0);
    for point in rank["keywords"][0]["grid"].as_array().unwrap() {
        assert!(point["lat"].as_f64().unwrap().abs() <= 90.);
        assert!(point["lng"].as_f64().unwrap().abs() <= 180.);
    }
}

#[tokio::test]
async fn partial_composite_failures_remain_visible() {
    let f = Fixture::new().await;
    f.override_response(
        "/places.googleapis.com/v1/places:searchNearby",
        StatusCode::FORBIDDEN,
        json!({"error":{"message":"quota failure"}}),
    );
    let area = f
        .call(
            "maps_explore_area",
            json!({"location":"1,1","types":["cafe"]}),
        )
        .await;
    assert!(
        area["categories"][0]["error"]
            .as_str()
            .unwrap()
            .contains("quota failure")
    );
    f.override_response(
        "/places.googleapis.com/v1/places:searchText",
        StatusCode::TOO_MANY_REQUESTS,
        json!({"error":{"message":"quota"}}),
    );
    let rank = f
        .call(
            "maps_local_rank_tracker",
            json!({"keyword":"cafe","placeId":"p1","center":{"latitude":1,"longitude":2}}),
        )
        .await;
    assert_eq!(rank["keywords"][0]["metrics"]["failed_points"], 9);
    assert_eq!(rank["keywords"][0]["metrics"]["atrp"], Value::Null);
}

#[tokio::test]
async fn api_errors_are_redacted_and_missing_key_is_explicit() {
    let f = Fixture::new().await;
    f.override_response(
        "/maps.googleapis.com/maps/api/geocode/json",
        StatusCode::FORBIDDEN,
        json!({"error":{"message":"bad fixture-secret-key"}}),
    );
    let error = f
        .engine
        .call("maps_geocode", json!({"address":"Paris"}))
        .await
        .err()
        .unwrap()
        .to_string();
    assert!(!error.contains("fixture-secret-key"));
    assert!(error.contains("[REDACTED]"));
    let client = MapsClient::new(None, Duration::from_secs(1), "en".into()).unwrap();
    assert!(matches!(
        client.geocode("Paris").await,
        Err(Error::MissingKey)
    ));
}

#[tokio::test]
async fn network_and_timeout_errors_do_not_expose_credential_urls() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let mut client = MapsClient::new(
        Some("never-log-this-key".into()),
        Duration::from_millis(50),
        "en".into(),
    )
    .unwrap();
    client.test_base = Some(format!("http://{address}"));
    // A bound socket without a responder forces a real HTTP response timeout.
    let error = client.geocode("Paris").await.err().unwrap();
    assert!(matches!(error, Error::Timeout));
    assert!(!error.to_string().contains("never-log-this-key"));
    drop(listener);
    // Windows can take longer than the timeout probe's 50 ms to report refusal.
    // Use a normal request budget when testing network-error classification.
    let mut client = MapsClient::new(
        Some("never-log-this-key".into()),
        Duration::from_secs(5),
        "en".into(),
    )
    .unwrap();
    client.test_base = Some(format!("http://{address}"));
    let error = client.geocode("Paris").await.err().unwrap();
    assert!(matches!(error, Error::Network), "{error}");
    assert!(!error.to_string().contains("never-log-this-key"));
}

fn rpc(id: u32, method: &str, params: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
}

pub(crate) fn decode_rpc(body: &str) -> Value {
    if let Ok(value) = serde_json::from_str(body) {
        return value;
    }
    body.lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|value| value.get("id").is_some())
        .unwrap_or_else(|| panic!("no JSON-RPC response in {body}"))
}

#[tokio::test]
async fn streamable_http_supports_sessions_tools_calls_auth_and_origin_checks() {
    let fixture = Fixture::new().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let cancellation = CancellationToken::new();
    let options = HttpOptions {
        listen: address,
        token: Some("http-secret".into()),
        allowed_hosts: vec![],
        allowed_origins: vec![],
    };
    let app = http_router(fixture.engine.clone(), &options, cancellation.clone());
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let url = format!("http://{address}/mcp");
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let initialize = rpc(
        1,
        "initialize",
        json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"integration-test","version":"1"}}),
    );
    assert_eq!(
        http.post(&url)
            .json(&initialize)
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        http.post(&url)
            .bearer_auth("http-secret")
            .header("origin", "https://evil.example")
            .header("accept", "application/json, text/event-stream")
            .json(&initialize)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let response = http
        .post(&url)
        .bearer_auth("http-secret")
        .header("accept", "application/json, text/event-stream")
        .json(&initialize)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let session = response.headers()["mcp-session-id"]
        .to_str()
        .unwrap()
        .to_owned();
    let init = decode_rpc(&response.text().await.unwrap());
    assert_eq!(init["result"]["serverInfo"]["name"], "mcp-gmap-rs");
    let request = |value: Value| {
        http.post(&url)
            .bearer_auth("http-secret")
            .header("accept", "application/json, text/event-stream")
            .header("mcp-session-id", &session)
            .header("mcp-protocol-version", "2025-11-25")
            .json(&value)
    };
    assert_eq!(
        request(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .send()
            .await
            .unwrap()
            .status(),
        202
    );
    let events = http
        .get(&url)
        .bearer_auth("http-secret")
        .header("accept", "text/event-stream")
        .header("mcp-session-id", &session)
        .send()
        .await
        .unwrap();
    assert_eq!(events.status(), 200);
    assert!(
        events.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/event-stream")
    );
    drop(events);
    let tools = decode_rpc(
        &request(rpc(2, "tools/list", json!({})))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
    );
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 18);
    let geocode = decode_rpc(
        &request(rpc(
            3,
            "tools/call",
            json!({"name":"maps_geocode","arguments":{"address":"Paris"}}),
        ))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap(),
    );
    assert_eq!(
        geocode["result"]["structuredContent"]["location"]["lat"],
        48.85
    );
    let image = decode_rpc(
        &request(rpc(
            4,
            "tools/call",
            json!({"name":"maps_static_map","arguments":{"center":"Paris"}}),
        ))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap(),
    );
    assert_eq!(image["result"]["content"][0]["type"], "image");
    let invalid = decode_rpc(
        &request(rpc(
            5,
            "tools/call",
            json!({"name":"maps_geocode","arguments":{}}),
        ))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap(),
    );
    assert_eq!(invalid["result"]["isError"], true);
    let deleted = http
        .delete(&url)
        .bearer_auth("http-secret")
        .header("mcp-session-id", &session)
        .send()
        .await
        .unwrap();
    assert!(deleted.status().is_success());
    // Current MCP inline negotiation also works without a legacy session.
    let meta = json!({"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}});
    for (id, method) in [(6, "server/discover"), (7, "tools/list")] {
        let response = http
            .post(&url)
            .bearer_auth("http-secret")
            .header("accept", "application/json, text/event-stream")
            .header("mcp-protocol-version", "2026-07-28")
            .header("mcp-method", method)
            .json(&rpc(id, method, json!({"_meta":meta})))
            .send()
            .await
            .unwrap();
        let status = response.status();
        let has_session = response.headers().contains_key("mcp-session-id");
        let body = response.text().await.unwrap();
        assert_eq!(status, 200, "{method}: {body}");
        assert!(!has_session);
        let result = decode_rpc(&body);
        assert!(result.get("error").is_none(), "{result}");
        if method == "tools/list" {
            assert_eq!(result["result"]["tools"].as_array().unwrap().len(), 18);
        }
    }
    cancellation.cancel();
    task.abort();
}
