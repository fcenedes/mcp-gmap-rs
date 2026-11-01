use super::*;

fn circle_area() -> Value {
    json!({"circle":{"latLng":{"latitude":48.85,"longitude":2.35},"radius":500}})
}

#[tokio::test]
async fn live_doctor_probes_distinct_dependencies_and_continues_after_api_failures() {
    let mut fixture = Fixture::new().await;
    let settings = crate::settings::Settings::test_defaults();
    fixture.engine.catalog = Catalog::new(Some("maps_snap_to_roads,maps_nearest_roads")).unwrap();
    let report = crate::doctor::report(&fixture.engine, &settings, true).await;
    assert_eq!(report["ok"], true);
    assert_eq!(fixture.records().len(), 1);
    assert!(fixture.records()[0].path.ends_with("/nearestRoads"));
    fixture.state.records.lock().unwrap().clear();
    fixture.engine.catalog = Catalog::new(None).unwrap();
    fixture.override_response(
        "/addressvalidation.googleapis.com/v1:validateAddress",
        StatusCode::FORBIDDEN,
        json!({"error":{"message":"fixture-secret-key denied"}}),
    );
    let report = crate::doctor::report(&fixture.engine, &settings, true).await;
    assert_eq!(report["ok"], false);
    assert_eq!(fixture.records().len(), 12);
    let apis: Vec<_> = report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|check| check["name"].as_str().unwrap().starts_with("api:"))
        .collect();
    assert_eq!(apis.len(), 21);
    assert_eq!(
        apis.iter()
            .filter(|check| check["status"] == "not_tested")
            .count(),
        9
    );
    assert_eq!(
        apis.iter()
            .filter(|check| check["status"] == "fail")
            .count(),
        1
    );
    assert!(!report.to_string().contains("fixture-secret-key"));
}

#[tokio::test]
async fn extended_tools_send_google_contracts_and_preserve_full_responses() {
    let fixture = Fixture::new().await;
    let address = fixture.call("maps_validate_address", json!({"addressLines":["5 Avenue Anatole France"],"regionCode":"FR","locality":"Paris","postalCode":"75007"})).await;
    assert_eq!(address["result"]["verdict"]["addressComplete"], true);
    assert_eq!(address["responseId"], "fixture-validation");
    let pollen = fixture
        .call(
            "maps_pollen",
            json!({"latitude":48.85,"longitude":2.35,"days":5,"plantsDescription":true}),
        )
        .await;
    assert_eq!(pollen["dailyInfo"][0]["pollenTypeInfo"][0]["code"], "GRASS");
    let area = fixture.call("maps_area_insights", json!({"area":circle_area(),"includedTypes":["restaurant"],"excludedTypes":["cafe"],"includePlaces":true,"minRating":4,"maxRating":5,"priceLevels":["PRICE_LEVEL_MODERATE"],"operatingStatus":["OPERATING_STATUS_OPERATIONAL"]})).await;
    assert_eq!(area["count"], "42");
    assert_eq!(area["placeInsights"][0]["place"], "places/p1");
    let points = json!([{"latitude":48.85,"longitude":2.35},{"latitude":48.851,"longitude":2.351}]);
    for (tool, args) in [
        (
            "maps_snap_to_roads",
            json!({"path":points,"interpolate":true}),
        ),
        ("maps_nearest_roads", json!({"points":points})),
    ] {
        let roads = fixture.call(tool, args).await;
        assert_eq!(roads["snappedPoints"][0]["originalIndex"], 0);
        assert_eq!(roads["warningMessage"], "fixture attribution");
    }
    let records = fixture.records();
    assert_eq!(records.len(), 5);
    assert_eq!(records[0].body["address"]["postalCode"], "75007");
    assert_eq!(records[0].body["enableUspsCass"], false);
    assert!(records[1].query.contains("days=5"));
    assert!(records[1].query.contains("plantsDescription=true"));
    assert!(records[1].query.contains("pageSize=5"));
    assert_eq!(records[2].body["filter"]["locationFilter"], circle_area());
    assert_eq!(
        records[2].body["filter"]["ratingFilter"],
        json!({"minRating":4,"maxRating":5})
    );
    assert_eq!(
        records[2].body["insights"],
        json!(["INSIGHT_COUNT", "INSIGHT_PLACES"])
    );
    assert!(
        records[3]
            .query
            .contains("path=48.85%2C2.35%7C48.851%2C2.351")
    );
    assert!(records[3].query.contains("interpolate=true"));
    assert!(records[4].query.contains("points="));
    fixture.call("maps_area_insights", json!({"area":{"customArea":{"polygon":{"coordinates":[{"latitude":48,"longitude":2},{"latitude":48,"longitude":3},{"latitude":49,"longitude":3},{"latitude":48.0,"longitude":2.0}]}}},"includedTypes":["restaurant"]})).await;
    assert_eq!(
        fixture.records().last().unwrap().body["filter"]["locationFilter"]["customArea"]["polygon"]
            ["coordinates"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
}

#[tokio::test]
async fn extended_tools_reject_invalid_and_inconsistent_arguments_before_network_calls() {
    let fixture = Fixture::new().await;
    for (tool, args) in [
        ("maps_validate_address", json!({"addressLines":[]})),
        (
            "maps_validate_address",
            json!({"addressLines":["Paris"],"regionCode":"FR","enableUspsCass":true}),
        ),
        ("maps_pollen", json!({"latitude":0,"longitude":0,"days":6})),
        (
            "maps_snap_to_roads",
            json!({"path":[{"latitude":0,"longitude":0}]}),
        ),
        (
            "maps_nearest_roads",
            json!({"points":[{"latitude":91,"longitude":0}]}),
        ),
        (
            "maps_area_insights",
            json!({"area":circle_area(),"includedTypes":[]}),
        ),
        (
            "maps_area_insights",
            json!({"area":circle_area(),"includedTypes":["cafe"],"minRating":5,"maxRating":4}),
        ),
        (
            "maps_area_insights",
            json!({"area":{"circle":{"latLng":{"latitude":0,"longitude":0},"radius":500},"region":{"place":"places/p1"}},"includedTypes":["cafe"]}),
        ),
        (
            "maps_area_insights",
            json!({"area":{"customArea":{"polygon":{"coordinates":[{"latitude":0,"longitude":0},{"latitude":0,"longitude":1},{"latitude":1,"longitude":1},{"latitude":1,"longitude":0}]}}},"includedTypes":["cafe"]}),
        ),
    ] {
        assert!(fixture.engine.call(tool, args).await.is_err(), "{tool}");
    }
    assert!(fixture.records().is_empty());
}

#[tokio::test]
async fn additional_oauth_apis_use_bearers_while_key_only_apis_keep_key_authentication() {
    let mut fixture = Fixture::new().await;
    let _credentials = fixture.enable_oauth();
    fixture
        .call("maps_validate_address", json!({"addressLines":["Paris"]}))
        .await;
    fixture
        .call(
            "maps_area_insights",
            json!({"area":{"region":{"place":"places/p1"}},"includedTypes":["cafe"]}),
        )
        .await;
    fixture
        .call("maps_pollen", json!({"latitude":48.85,"longitude":2.35}))
        .await;
    fixture
        .call(
            "maps_nearest_roads",
            json!({"points":[{"latitude":48.85,"longitude":2.35}]}),
        )
        .await;
    let records = fixture.records();
    assert_eq!(records.len(), 5);
    for record in &records[1..3] {
        assert_eq!(
            record.headers["authorization"],
            "Bearer fixture-oauth-token"
        );
        assert_eq!(record.headers["x-goog-user-project"], "fixture-project");
        assert!(!record.headers.contains_key("x-goog-api-key"));
        assert!(!record.query.contains("key="));
    }
    for record in &records[3..] {
        assert!(!record.headers.contains_key("authorization"));
        assert!(record.query.contains("key=fixture-secret-key"));
    }
}

#[tokio::test]
async fn budgets_cover_parallel_composites_and_reset_between_invocations() {
    let mut fixture = Fixture::new().await;
    fixture.engine.client.policy.max_requests = 2;
    let result = fixture
        .call("maps_batch_geocode", json!({"addresses":["A","B","C","D"]}))
        .await;
    assert_eq!(fixture.records().len(), 2);
    let results = result.as_array().unwrap();
    assert_eq!(
        results.iter().filter(|row| row["success"] == true).count(),
        2
    );
    assert_eq!(
        results
            .iter()
            .filter(|row| row["error"]
                .as_str()
                .is_some_and(|message| message.contains("budget")))
            .count(),
        2
    );
    fixture
        .call("maps_geocode", json!({"address":"Paris"}))
        .await;
    assert_eq!(fixture.records().len(), 3);
}

#[tokio::test]
async fn retries_handle_throttling_and_transient_failures_and_are_included_in_budgets_and_metrics()
{
    let mut fixture = Fixture::new().await;
    fixture.engine.client.policy.max_retries = 2;
    let path = "/roads.googleapis.com/v1/nearestRoads";
    fixture.state.queued.lock().unwrap().insert(
        path.into(),
        [
            (
                StatusCode::TOO_MANY_REQUESTS,
                json!({"error":{"message":"throttled"}}),
                Some("Sun, 06 Nov 1994 08:49:37 GMT"),
            ),
            (
                StatusCode::SERVICE_UNAVAILABLE,
                json!({"error":{"message":"unavailable"}}),
                Some("0"),
            ),
        ]
        .into(),
    );
    let args = json!({"points":[{"latitude":48.85,"longitude":2.35}]});
    fixture.call("maps_nearest_roads", args.clone()).await;
    assert_eq!(fixture.records().len(), 3);
    let metrics = fixture.engine.client.metrics.render();
    assert!(metrics.contains("status=\"429\"} 1"));
    assert!(metrics.contains("status=\"503\"} 1"));
    assert!(metrics.contains("tool=\"maps_nearest_roads\",outcome=\"success\"} 1"));
    assert!(!metrics.contains("fixture-secret-key"));
    fixture.engine.client.policy.max_requests = 1;
    fixture.override_response(
        path,
        StatusCode::SERVICE_UNAVAILABLE,
        json!({"error":{"message":"unavailable"}}),
    );
    assert!(matches!(
        fixture
            .engine
            .call("maps_nearest_roads", args.clone())
            .await,
        Err(Error::Budget)
    ));
    assert_eq!(fixture.records().len(), 4);
    fixture.override_response(
        path,
        StatusCode::FORBIDDEN,
        json!({"error":{"message":"forbidden"}}),
    );
    assert!(matches!(
        fixture.engine.call("maps_nearest_roads", args).await,
        Err(Error::Api(_))
    ));
    assert_eq!(fixture.records().len(), 5);
}

#[tokio::test]
async fn retry_after_waits_are_bounded_by_request_and_tool_deadlines() {
    let mut fixture = Fixture::new().await;
    fixture.engine.client.policy.max_retries = 2;
    fixture.engine.client.policy.tool_timeout = Duration::from_millis(100);
    fixture.state.queued.lock().unwrap().insert(
        "/roads.googleapis.com/v1/nearestRoads".into(),
        [(
            StatusCode::TOO_MANY_REQUESTS,
            json!({"error":{"message":"throttled"}}),
            Some("60"),
        )]
        .into(),
    );
    assert!(matches!(
        fixture
            .engine
            .call(
                "maps_nearest_roads",
                json!({"points":[{"latitude":48.85,"longitude":2.35}]})
            )
            .await,
        Err(Error::ToolTimeout)
    ));
    assert_eq!(fixture.records().len(), 1);
}

#[tokio::test]
async fn http_limits_are_shared_across_sessions_and_metrics_are_protected_and_exempt() {
    let fixture = Fixture::new().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let cancellation = CancellationToken::new();
    let options = HttpOptions {
        listen: address,
        token: Some("http-secret".into()),
        allowed_hosts: vec![],
        allowed_origins: vec![],
        rate_limit: 1,
        metrics: true,
        tls: None,
    };
    let app = http_router(fixture.engine.clone(), &options, cancellation.clone());
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let client = reqwest::Client::new();
    let base = format!("http://{address}");
    assert_eq!(
        client
            .get(format!("{base}/metrics"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let initialize = rpc(
        1,
        "initialize",
        json!({"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"rate-test","version":"1"}}),
    );
    // Unauthenticated requests cannot consume the authenticated request budget.
    assert_eq!(
        client
            .post(format!("{base}/mcp"))
            .json(&initialize)
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let post = || {
        client
            .post(format!("{base}/mcp"))
            .bearer_auth("http-secret")
            .header("accept", "application/json, text/event-stream")
            .json(&initialize)
    };
    assert_eq!(post().send().await.unwrap().status(), 200);
    let limited = post().send().await.unwrap();
    assert_eq!(limited.status(), 429);
    assert!(limited.headers().contains_key("retry-after"));
    fixture
        .call("maps_pollen", json!({"latitude":48.85,"longitude":2.35}))
        .await;
    for _ in 0..2 {
        assert_eq!(
            client
                .get(format!("{base}/health"))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        let response = client
            .get(format!("{base}/metrics"))
            .bearer_auth("http-secret")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert!(
            response.headers()["content-type"]
                .to_str()
                .unwrap()
                .contains("version=0.0.4")
        );
        let text = response.text().await.unwrap();
        assert!(text.contains("mcp_gmap_http_rate_limited_total 1"));
        assert!(text.contains("mcp_gmap_tool_duration_seconds_bucket"));
        assert!(!text.contains("http-secret"));
    }
    cancellation.cancel();
    task.abort();
}
