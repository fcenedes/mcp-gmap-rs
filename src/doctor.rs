//! CLI diagnostics. Network probes require an explicit `doctor --live` opt-in.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use crate::{catalog::Catalog, settings::Settings, tools::EngineState};

pub(crate) async fn report(engine: &EngineState, settings: &Settings, live: bool) -> Value {
    let mut checks = Vec::new();
    let enabled: Vec<_> = engine
        .catalog
        .iter()
        .map(|definition| definition.tool.name.to_string())
        .collect();
    let apis: BTreeSet<_> = enabled
        .iter()
        .flat_map(|name| requirements(name).iter().copied())
        .collect();
    checks.push(json!({"name":"configuration","status":"pass","message":"Configuration and tool selection parsed successfully"}));
    if engine.client.has_key() {
        checks.push(json!({"name":"api_key","status":"pass","message":"API key is configured; validity and API restrictions require live checks"}));
    } else if engine.client.oauth.is_none()
        || apis.iter().any(|api| {
            matches!(
                *api,
                "geocoding" | "elevation" | "timezone" | "static_map" | "pollen" | "roads"
            )
        })
    {
        checks.push(json!({"name":"api_key","status":"fail","message":"GOOGLE_MAPS_API_KEY is required by one or more enabled tool paths"}));
    }
    if let Some(oauth) = &engine.client.oauth {
        match oauth.check_local().await {
            Ok(true) => checks.push(json!({"name":"oauth_credentials","status":"pass","message":"Explicit credential file parsed; token acquisition and IAM require live checks"})),
            Ok(false) => checks.push(json!({"name":"oauth_credentials","status":"not_tested","message":"ADC discovery and workload identity are deferred to live requests"})),
            Err(error) => checks.push(json!({"name":"oauth_credentials","status":"fail","message":error.to_string()})),
        }
    }
    match settings.http_options() {
        Ok(options) => {
            let protected = options.listen.ip().is_loopback() || options.token.is_some();
            checks.push(json!({"name":"http_authentication","status":if protected { "pass" } else { "fail" },"message":if protected { "HTTP binding satisfies the bearer-token requirement" } else { "Non-loopback HTTP requires MCP_HTTP_TOKEN" }}));
            if let Some(files) = options.tls {
                match files.load().await {
                    Ok(_) => checks.push(json!({"name":"tls","status":"pass","message":"TLS certificate and private key match; trust, hostname and expiry must also be checked by clients"})),
                    Err(_) => checks.push(json!({"name":"tls","status":"fail","message":"TLS certificate, key or client CA could not be loaded"})),
                }
            } else {
                checks.push(json!({"name":"tls","status":"not_tested","message":"Native TLS is disabled; remote deployments need TLS at a gateway or native TLS settings"}));
            }
        }
        Err(_) => checks.push(json!({"name":"http_configuration","status":"fail","message":"Invalid HTTP or TLS settings; check listen address, token and paired TLS paths"})),
    }
    if live {
        // Probe the API dependencies of the enabled tools, without invoking expensive composites.
        // A restricted catalog must not prevent diagnostics from testing its underlying APIs.
        if let Ok(catalog) = Catalog::new(None) {
            let probe_engine = EngineState {
                client: engine.client.clone(),
                catalog,
            };
            for api in apis {
                let (tool, args) = probe(api);
                match probe_engine.call(tool, args).await {
                    Ok(_) => checks.push(json!({"name":format!("api:{api}"),"status":"pass","message":"Google accepted the probe"})),
                    Err(error) => checks.push(json!({"name":format!("api:{api}"),"status":"fail","message":error.to_string()})),
                }
            }
        }
    }
    json!({
        "version":env!("CARGO_PKG_VERSION"),"live":live,
        "ok":!checks.iter().any(|check| check["status"]=="fail"),
        "auth_mode":if engine.client.oauth.is_some() { "oauth" } else { "api-key" },
        "enabled_tools":enabled,
        "limits":{"google_attempts_per_tool":settings.policy.max_requests,"retries_per_request":settings.policy.max_retries,"tool_timeout_seconds":settings.policy.tool_timeout.as_secs()},
        "checks":checks
    })
}

fn requirements(tool: &str) -> &'static [&'static str] {
    match tool {
        "maps_geocode" | "maps_reverse_geocode" | "maps_batch_geocode" => &["geocoding"],
        "maps_search_places" | "maps_place_details" | "maps_local_rank_tracker" => &["places"],
        "maps_search_nearby" | "maps_explore_area" => &["geocoding", "places"],
        "maps_directions"
        | "maps_distance_matrix"
        | "maps_plan_route"
        | "maps_compare_places"
        | "maps_search_along_route" => &["geocoding", "places", "routes"],
        "maps_elevation" => &["elevation"],
        "maps_timezone" => &["timezone"],
        "maps_weather" => &["weather"],
        "maps_air_quality" => &["air_quality"],
        "maps_static_map" => &["static_map"],
        "maps_validate_address" => &["address_validation"],
        "maps_area_insights" => &["area_insights"],
        "maps_pollen" => &["pollen"],
        "maps_snap_to_roads" | "maps_nearest_roads" => &["roads"],
        _ => &[],
    }
}

fn probe(api: &str) -> (&'static str, Value) {
    let location = json!({"latitude":48.8584,"longitude":2.2945});
    match api {
        "geocoding" => (
            "maps_geocode",
            json!({"address":"Eiffel Tower, Paris, France"}),
        ),
        "places" => (
            "maps_search_places",
            json!({"query":"Eiffel Tower, Paris, France"}),
        ),
        "routes" => (
            "maps_directions",
            json!({"origin":"48.8584,2.2945","destination":"48.8606,2.3376","mode":"walking"}),
        ),
        "elevation" => ("maps_elevation", json!({"locations":[location]})),
        "timezone" => ("maps_timezone", location),
        "weather" => ("maps_weather", location),
        "air_quality" => ("maps_air_quality", location),
        "static_map" => (
            "maps_static_map",
            json!({"center":"48.8584,2.2945","size":"64x64"}),
        ),
        "address_validation" => (
            "maps_validate_address",
            json!({"addressLines":["5 Avenue Anatole France"],"locality":"Paris","postalCode":"75007","regionCode":"FR"}),
        ),
        "area_insights" => (
            "maps_area_insights",
            json!({"area":{"circle":{"latLng":location,"radius":500}},"includedTypes":["restaurant"]}),
        ),
        "pollen" => ("maps_pollen", location),
        _ => ("maps_nearest_roads", json!({"points":[location]})),
    }
}
