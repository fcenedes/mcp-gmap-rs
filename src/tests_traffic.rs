use super::*;

const LINE: &str = "_p~iF~ps|U_ulLnnqC_mqNvxq`@";
const LEG: &str = "_p~iF~ps|U_ulLnnqC";
const ENDPOINT: &str = "/routes.googleapis.com/directions/v2:computeRoutes";

fn traffic_route() -> Value {
    json!({"routes":[{
        "distanceMeters":1000,"duration":"120s","staticDuration":"100s",
        "polyline":{"encodedPolyline":LINE},
        "travelAdvisory":{"speedReadingIntervals":[{"endPolylinePointIndex":1,"speed":"NORMAL"},{"startPolylinePointIndex":1,"endPolylinePointIndex":2,"speed":"SLOW"}]},
        "legs":[{"polyline":{"encodedPolyline":LEG},"travelAdvisory":{"speedReadingIntervals":[{"endPolylinePointIndex":1,"speed":"TRAFFIC_JAM"}]}}]
    }]})
}

#[tokio::test]
async fn route_quality_can_be_selected_without_requesting_traffic_segments() {
    let fixture = Fixture::new().await;
    fixture.override_response(ENDPOINT, StatusCode::OK, traffic_route());
    for quality in ["HIGH_QUALITY", "OVERVIEW"] {
        for (tool, mut args) in [
            (
                "maps_directions",
                json!({"origin":"38.5,-120.2","destination":"43.252,-126.453","mode":"walking"}),
            ),
            (
                "maps_plan_route",
                json!({"stops":["38.5,-120.2","43.252,-126.453"],"mode":"walking"}),
            ),
        ] {
            args["polylineQuality"] = json!(quality);
            let output = fixture.call(tool, args).await;
            let route = if tool == "maps_directions" {
                &output["routes"][0]
            } else {
                &output
            };
            assert_eq!(route["polyline"]["encodedPolyline"], LINE);
            assert!(route.get("segments").is_none());
            let request = fixture.records().pop().unwrap();
            assert_eq!(request.body["polylineQuality"], quality);
            assert_eq!(request.body["travelMode"], "WALK");
            assert!(request.body.get("extraComputations").is_none());
            assert!(
                !request.headers["x-goog-fieldmask"]
                    .to_str()
                    .unwrap()
                    .contains("routes.travelAdvisory.speedReadingIntervals")
            );
        }
    }
    assert_eq!(fixture.records().len(), 4);
}

#[tokio::test]
async fn explicit_overview_quality_is_honored_with_traffic_and_optimized_stops() {
    let fixture = Fixture::new().await;
    let mut native = traffic_route();
    native["routes"][0]["optimizedIntermediateWaypointIndex"] = json!([1, 0]);
    let leg = native["routes"][0]["legs"][0].clone();
    native["routes"][0]["legs"] = json!([leg, leg, leg]);
    fixture.override_response(ENDPOINT, StatusCode::OK, native);
    let output = fixture.call("maps_plan_route", json!({"stops":["38.5,-120.2","39,-121","40,-122","43.252,-126.453"],"optimize":true,"trafficSegments":true,"polylineQuality":"OVERVIEW"})).await;
    let requests = fixture.records();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].body["polylineQuality"], "OVERVIEW");
    assert_eq!(
        requests[0].body["extraComputations"],
        json!(["TRAFFIC_ON_POLYLINE"])
    );
    assert_eq!(requests[0].body["optimizeWaypointOrder"], true);
    assert_eq!(output["segments"][0]["endPolylinePointIndex"], 1);
    assert_eq!(output["legs"][2]["segmentMetadata"]["legIndex"], 2);
}

#[tokio::test]
async fn invalid_quality_values_fail_schema_validation_before_google_calls() {
    let fixture = Fixture::new().await;
    for quality in [json!("HIGH"), json!("high_quality"), json!(null), json!(42)] {
        for (tool, mut args) in [
            (
                "maps_directions",
                json!({"origin":"Paris","destination":"Lyon"}),
            ),
            ("maps_plan_route", json!({"stops":["Paris","Lyon"]})),
        ] {
            args["polylineQuality"] = quality.clone();
            assert!(fixture.engine.call(tool, args).await.is_err());
        }
    }
    assert!(fixture.records().is_empty());
}

#[tokio::test]
async fn traffic_segments_are_opt_in_and_use_one_routes_call_with_native_indices() {
    let fixture = Fixture::new().await;
    fixture.override_response(ENDPOINT, StatusCode::OK, traffic_route());
    let args = json!({"origin":"38.5,-120.2","destination":"43.252,-126.453"});
    let plain = fixture.call("maps_directions", args.clone()).await;
    assert!(plain["routes"][0].get("segments").is_none());
    let mut requested = args;
    requested["trafficSegments"] = json!(true);
    let output = fixture.call("maps_directions", requested).await;
    let route = &output["routes"][0];
    assert_eq!(route["polyline"]["encodedPolyline"], LINE);
    assert_eq!(route["staticDuration"], "100s");
    assert_eq!(route["segments"][0]["startPolylinePointIndex"], 0);
    assert_eq!(route["segments"][1]["trafficCategory"], "SLOW");
    assert_eq!(
        route["legs"][0]["segments"][0]["startPolylinePointIndex"],
        0
    );
    assert_eq!(
        route["legs"][0]["segments"][0]["trafficCategory"],
        "TRAFFIC_JAM"
    );
    for field in ["roadCategory", "speedLimit", "trafficObservedAt"] {
        assert!(route["segments"][0][field].is_null());
    }
    assert!(
        chrono::DateTime::parse_from_rfc3339(
            route["segmentMetadata"]["retrievedAt"].as_str().unwrap()
        )
        .is_ok()
    );
    let requests = fixture.records();
    assert_eq!(requests.len(), 2);
    assert!(requests[0].body.get("extraComputations").is_none());
    assert!(requests[0].body.get("polylineQuality").is_none());
    assert_eq!(
        requests[1].body["extraComputations"],
        json!(["TRAFFIC_ON_POLYLINE"])
    );
    assert_eq!(requests[1].body["polylineQuality"], "HIGH_QUALITY");
    assert_eq!(requests[1].body["polylineEncoding"], "ENCODED_POLYLINE");
    assert_eq!(requests[1].body["routingPreference"], "TRAFFIC_AWARE");
    assert!(
        requests[1].headers["x-goog-fieldmask"]
            .to_str()
            .unwrap()
            .contains("routes.travelAdvisory.speedReadingIntervals")
    );
}

#[tokio::test]
async fn traffic_segments_allow_waypoint_optimization_and_do_not_relabel_future_departure_as_observation()
 {
    let fixture = Fixture::new().await;
    let mut native = traffic_route();
    native["routes"][0]["optimizedIntermediateWaypointIndex"] = json!([1, 0]);
    let leg = native["routes"][0]["legs"][0].clone();
    native["routes"][0]["legs"] = json!([leg, leg, leg]);
    fixture.override_response(ENDPOINT, StatusCode::OK, native);
    let output = fixture.call("maps_plan_route", json!({"stops":["38.5,-120.2","39,-121","40,-122","43.252,-126.453"],"optimize":true,"trafficSegments":true,"departure_time":"2027-01-01T09:00:00Z"})).await;
    let requests = fixture.records();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].body["optimizeWaypointOrder"], true);
    assert_eq!(requests[0].body["routingPreference"], "TRAFFIC_AWARE");
    assert_eq!(
        output["segmentMetadata"]["requestedDepartureTime"],
        "2027-01-01T09:00:00+00:00"
    );
    assert!(output["segments"][0]["trafficObservedAt"].is_null());
    assert_eq!(output["legs"][2]["segmentMetadata"]["legIndex"], 2);
    assert_eq!(output["legs"][2]["polyline"]["encodedPolyline"], LEG);
}

#[tokio::test]
async fn unsupported_segment_modes_and_schema_mistakes_fail_before_geocoding() {
    let fixture = Fixture::new().await;
    for mode in ["walking", "bicycling", "transit"] {
        for (tool, mut args) in [
            (
                "maps_directions",
                json!({"origin":"Paris","destination":"Lyon"}),
            ),
            ("maps_plan_route", json!({"stops":["Paris","Lyon"]})),
        ] {
            args["mode"] = json!(mode);
            args["trafficSegments"] = json!(true);
            assert!(fixture.engine.call(tool, args).await.is_err());
        }
    }
    assert!(
        fixture
            .engine
            .call(
                "maps_directions",
                json!({"origin":"Paris","destination":"Lyon","trafficSegments":"true"})
            )
            .await
            .is_err()
    );
    assert!(
        fixture
            .engine
            .call(
                "maps_distance_matrix",
                json!({"origins":["Paris"],"destinations":["Lyon"],"trafficSegments":true})
            )
            .await
            .is_err()
    );
    assert!(fixture.records().is_empty());
}

#[tokio::test]
async fn missing_traffic_stays_unavailable_and_google_fallback_is_preserved() {
    let mut fixture = Fixture::new().await;
    let _credentials = fixture.enable_oauth();
    let mut native = traffic_route();
    native["routes"][0]
        .as_object_mut()
        .unwrap()
        .remove("travelAdvisory");
    native["fallbackInfo"] =
        json!({"routingMode":"FALLBACK_TRAFFIC_UNAWARE","reason":"SERVER_ERROR"});
    fixture.override_response(ENDPOINT, StatusCode::OK, native);
    let output = fixture
        .call(
            "maps_directions",
            json!({"origin":"38.5,-120.2","destination":"43.252,-126.453","trafficSegments":true}),
        )
        .await;
    assert_eq!(output["routes"][0]["segments"], json!([]));
    assert_eq!(
        output["routes"][0]["segmentMetadata"]["trafficStatus"],
        "NOT_RETURNED"
    );
    assert_eq!(
        output["fallbackInfo"]["routingMode"],
        "FALLBACK_TRAFFIC_UNAWARE"
    );
    assert!(fixture.records().iter().filter(|r| r.path == ENDPOINT).all(
        |r| r.headers["authorization"] == "Bearer fixture-oauth-token" && !r.query.contains("key=")
    ));
}
