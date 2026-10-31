use super::*;

#[tokio::test]
async fn mutating_requests_are_never_retried_and_successful_json_redacts_credentials() {
    let mut fixture = Fixture::new().await;
    fixture.engine.client.policy.max_retries = 2;
    let path = "/aerialview.googleapis.com/v1/videos:renderVideo";
    fixture.override_response(
        path,
        StatusCode::SERVICE_UNAVAILABLE,
        json!({"error":{"message":"fixture-secret-key rejected"}}),
    );
    let error = fixture
        .engine
        .call(
            "maps_render_aerial_video",
            json!({"address":"1600 Amphitheatre Parkway, Mountain View, CA"}),
        )
        .await
        .err()
        .unwrap();
    assert!(!error.to_string().contains("fixture-secret-key"));
    assert_eq!(fixture.records().len(), 1);
    assert_eq!(fixture.records()[0].method, axum::http::Method::POST);
    fixture.override_response("/tile.googleapis.com/v1/3dtiles/root.json", StatusCode::OK, json!({"copyright":"Google","nested":[{"uri":"https://tile.googleapis.com/v1/3dtiles/child.json?key=fixture-secret-key&session=s"}]}));
    let result = fixture
        .call("maps_map_tiles", json!({"operation":"root_3d"}))
        .await;
    assert_eq!(result["copyright"], "Google");
    assert!(!result.to_string().contains("fixture-secret-key"));
    assert!(
        result["nested"][0]["uri"]
            .as_str()
            .unwrap()
            .contains("session=s")
    );
}

#[tokio::test]
async fn oauth_only_services_reject_api_key_mode_before_network_and_publish_rejects_service_accounts()
 {
    let mut fixture = Fixture::new().await;
    for (name, args) in [
        (
            "maps_datasets",
            json!({"operation":"list","parent":"projects/fixture-project"}),
        ),
        (
            "maps_route_optimization",
            json!({"operation":"validate","parent":"projects/fixture-project","request":{"model":{}}}),
        ),
        ("maps_streetview_publish", json!({"operation":"list"})),
    ] {
        let error = fixture.engine.call(name, args).await.err().unwrap();
        assert!(matches!(error, Error::OAuthRequired(_)), "{name}: {error}");
    }
    assert!(fixture.records().is_empty());
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("service-account.json");
    std::fs::write(&path, r#"{"type":"service_account"}"#).unwrap();
    fixture.engine.client.oauth = Some(OAuth::new(
        Some(path),
        Some("fixture-project".into()),
        Duration::from_secs(2),
    ));
    let error = fixture
        .engine
        .call("maps_streetview_publish", json!({"operation":"list"}))
        .await
        .err()
        .unwrap();
    assert!(matches!(error, Error::OAuthRequired(_)));
    assert!(fixture.records().is_empty());
}

#[tokio::test]
async fn autocomplete_sessions_can_conclude_with_details_or_address_validation() {
    let fixture = Fixture::new().await;
    fixture
        .call(
            "maps_place_details",
            json!({"placeId":"p1","sessionToken":"session-123"}),
        )
        .await;
    fixture
        .call(
            "maps_validate_address",
            json!({"addressLines":["Paris"],"sessionToken":"session-123"}),
        )
        .await;
    let records = fixture.records();
    assert!(records[0].query.contains("sessionToken=session-123"));
    assert_eq!(records[1].body["sessionToken"], "session-123");
}

#[tokio::test]
async fn doctor_does_not_mutate_cloud_resources_or_probe_restricted_speed_limits() {
    let mut fixture = Fixture::new().await;
    fixture.engine.catalog = Catalog::new(Some("maps_datasets,maps_streetview_publish,maps_render_aerial_video,maps_route_optimization,maps_speed_limits")).unwrap();
    let report = crate::doctor::report(
        &fixture.engine,
        &crate::settings::Settings::test_defaults(),
        true,
    )
    .await;
    let records = fixture.records();
    assert_eq!(records.len(), 1);
    assert!(records[0].path.ends_with("/nearestRoads"));
    assert_eq!(
        report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|check| check["status"] == "not_tested"
                && check["name"].as_str().unwrap().starts_with("api:"))
            .count(),
        5
    );
}

#[test]
fn discovery_exposes_conservative_annotations_for_mutating_tools() {
    let catalog = Catalog::new(None).unwrap();
    for name in [
        "maps_datasets",
        "maps_route_optimization",
        "maps_streetview_publish",
        "maps_render_aerial_video",
        "maps_map_tiles",
    ] {
        let annotations = catalog
            .get(name)
            .unwrap()
            .tool
            .annotations
            .as_ref()
            .unwrap();
        assert_eq!(annotations.read_only_hint, Some(false), "{name}");
        assert_eq!(annotations.idempotent_hint, Some(false), "{name}");
    }
    assert_eq!(
        catalog
            .get("maps_streetview_publish")
            .unwrap()
            .tool
            .annotations
            .as_ref()
            .unwrap()
            .destructive_hint,
        Some(true)
    );
}

#[tokio::test]
async fn signed_streetview_requests_append_signature_after_the_complete_encoded_unsigned_uri() {
    let fixture = Fixture::new().await;
    fixture
        .call(
            "maps_streetview",
            json!({"operation":"image","location":"Café Paris","signature":"signed_value="}),
        )
        .await;
    fixture.override_response(
        "/maps.googleapis.com/maps/api/streetview/metadata",
        StatusCode::OK,
        json!({"status":"OK"}),
    );
    fixture
        .call(
            "maps_streetview",
            json!({"operation":"metadata","location":"Café Paris","signature":"signed_value="}),
        )
        .await;
    let records = fixture.records();
    assert_eq!(
        records[0].query,
        "location=Caf%C3%A9+Paris&size=640x400&return_error_code=true&key=fixture-secret-key&signature=signed_value%3D"
    );
    assert_eq!(
        records[1].query,
        "location=Caf%C3%A9+Paris&key=fixture-secret-key&signature=signed_value%3D"
    );
}
