//! Opt-in live acceptance tests. These call billable Google APIs with the configured key.
//! Run with `cargo test --test live -- --ignored --test-threads=1`.

use std::process::Command;

use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};

const EIFFEL_TOWER_ID: &str = "ChIJLU7jZClu5kcR4PcOOO6p3I0";

fn location() -> Value {
    json!({"latitude":48.8584,"longitude":2.2945})
}

fn invoke(tool: &str, args: Value) -> Value {
    let key = std::env::var("GOOGLE_MAPS_API_KEY")
        .expect("export GOOGLE_MAPS_API_KEY before running ignored live tests");
    assert!(
        !key.trim().is_empty(),
        "GOOGLE_MAPS_API_KEY must be nonempty"
    );
    let output = Command::new(env!("CARGO_BIN_EXE_mcp-gmap-rs"))
        .args(["call", tool, "--args", &args.to_string()])
        .env("GOOGLE_MAPS_API_KEY", &key)
        .env_remove("GOOGLE_MAPS_ENABLED_TOOLS")
        .env_remove("GOOGLE_MAPS_TIMEOUT")
        .env_remove("GOOGLE_MAPS_LANGUAGE")
        .env_remove("RUST_LOG")
        .output()
        .expect("start live-test executable");
    assert!(
        output.status.success(),
        "{tool}: {}",
        String::from_utf8_lossy(&output.stderr).replace(&key, "[REDACTED]")
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("tool must return JSON");
    let mut failures = Vec::new();
    collect_failures(&value, "", &mut failures);
    assert!(
        failures.is_empty(),
        "{tool} returned partial failures: {}",
        failures.join("; ").replace(&key, "[REDACTED]")
    );
    value
}

fn collect_failures(value: &Value, path: &str, failures: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            for (key, entry) in object {
                let path = format!("{path}/{key}");
                let failed = match key.as_str() {
                    "error" | "details_error" | "distance_error" => !entry.is_null(),
                    "errors" | "distance_errors" => entry.as_array().is_some_and(|a| !a.is_empty()),
                    "success" => entry == false,
                    "failed_points" => entry.as_u64().is_some_and(|n| n > 0),
                    _ => false,
                };
                if failed {
                    failures.push(format!("{path}: {entry}"));
                } else {
                    collect_failures(entry, &path, failures);
                }
            }
        }
        Value::Array(array) => {
            for (i, entry) in array.iter().enumerate() {
                collect_failures(entry, &format!("{path}/{i}"), failures);
            }
        }
        _ => {}
    }
}

fn nonempty_string(value: &Value) -> bool {
    value.as_str().is_some_and(|s| !s.is_empty())
}

macro_rules! live_test {
    ($test:ident, $tool:literal, $args:expr, $check:expr) => {
        #[test]
        #[ignore = "requires a Google Maps key, enabled APIs, network access, and billable requests"]
        fn $test() {
            let value = invoke($tool, $args);
            let check: fn(&Value) = $check;
            check(&value);
        }
    };
}

live_test!(
    live_maps_geocode,
    "maps_geocode",
    json!({"address":"Eiffel Tower, Paris, France"}),
    |v| {
        assert!(v["location"]["lat"].as_f64().is_some());
        assert!(v["location"]["lng"].as_f64().is_some());
    }
);

live_test!(
    live_maps_reverse_geocode,
    "maps_reverse_geocode",
    location(),
    |v| {
        assert!(nonempty_string(&v["formatted_address"]));
    }
);

live_test!(
    live_maps_search_places,
    "maps_search_places",
    json!({"query":"Eiffel Tower, Paris, France"}),
    |v| {
        assert!(nonempty_string(&v[0]["id"]));
    }
);

live_test!(
    live_maps_search_nearby,
    "maps_search_nearby",
    json!({"center":{"value":"48.8584,2.2945","isCoordinates":true},"keyword":"cafe","radius":500}),
    |v| {
        assert!(nonempty_string(&v[0]["id"]));
    }
);

live_test!(
    live_maps_place_details,
    "maps_place_details",
    json!({"placeId":EIFFEL_TOWER_ID,"maxPhotos":1}),
    |v| {
        assert_eq!(v["id"], EIFFEL_TOWER_ID);
        assert!(nonempty_string(&v["photoUrls"][0]["photoUri"]));
        assert!(v["photoUrls"][0]["authorAttributions"].is_array());
    }
);

live_test!(
    live_maps_directions,
    "maps_directions",
    json!({"origin":"48.8584,2.2945","destination":"48.8606,2.3376","mode":"walking"}),
    |v| {
        assert!(
            v["total_distance"]["value"]
                .as_f64()
                .is_some_and(|n| n > 0.)
        );
        assert!(nonempty_string(
            &v["routes"][0]["polyline"]["encodedPolyline"]
        ));
    }
);

live_test!(
    live_maps_distance_matrix,
    "maps_distance_matrix",
    json!({"origins":["48.8584,2.2945"],"destinations":["48.8606,2.3376","48.8530,2.3499"],"mode":"walking"}),
    |v| {
        assert_eq!(v["distances"][0].as_array().unwrap().len(), 2);
        assert!(
            v["distances"][0][0]["value"]
                .as_f64()
                .is_some_and(|n| n > 0.)
        );
        assert!(
            v["durations"][0][1]["value"]
                .as_f64()
                .is_some_and(|n| n > 0.)
        );
    }
);

live_test!(
    live_maps_elevation,
    "maps_elevation",
    json!({"locations":[location()]}),
    |v| {
        assert!(v[0]["elevation"].as_f64().is_some());
    }
);

live_test!(live_maps_timezone, "maps_timezone", location(), |v| {
    assert_eq!(v["timeZoneId"], "Europe/Paris");
    assert!(chrono::DateTime::parse_from_rfc3339(v["localTime"].as_str().unwrap()).is_ok());
});

#[test]
#[ignore = "requires Weather API and a Google Maps key; makes three billable requests"]
fn live_maps_weather() {
    let current = invoke("maps_weather", location());
    assert!(current["temperature"]["degrees"].as_f64().is_some());
    let mut daily = location();
    daily["type"] = json!("forecast_daily");
    daily["forecastDays"] = json!(1);
    let forecast = invoke("maps_weather", daily);
    assert!(!forecast["forecastDays"].as_array().unwrap().is_empty());
    let mut hourly = location();
    hourly["type"] = json!("forecast_hourly");
    hourly["forecastHours"] = json!(1);
    let forecast = invoke("maps_weather", hourly);
    assert!(!forecast["forecastHours"].as_array().unwrap().is_empty());
}

live_test!(
    live_maps_air_quality,
    "maps_air_quality",
    json!({"latitude":48.8584,"longitude":2.2945,"includeHealthRecommendations":true,"includePollutants":true}),
    |v| {
        assert!(!v["indexes"].as_array().unwrap().is_empty());
        assert!(v["healthRecommendations"].is_object());
        assert!(!v["pollutants"].as_array().unwrap().is_empty());
    }
);

live_test!(
    live_maps_static_map,
    "maps_static_map",
    json!({"center":"48.8584,2.2945","zoom":13,"size":"320x240","markers":["color:red|48.8584,2.2945"]}),
    |v| {
        let bytes = STANDARD.decode(v["base64"].as_str().unwrap()).unwrap();
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), 320);
        assert_eq!(u32::from_be_bytes(bytes[20..24].try_into().unwrap()), 240);
    }
);

live_test!(
    live_maps_batch_geocode,
    "maps_batch_geocode",
    json!({"addresses":["Eiffel Tower, Paris, France","Louvre Museum, Paris, France"]}),
    |v| {
        assert_eq!(v.as_array().unwrap().len(), 2);
        assert!(
            v.as_array()
                .unwrap()
                .iter()
                .all(|item| item["success"] == true)
        );
    }
);

live_test!(
    live_maps_explore_area,
    "maps_explore_area",
    json!({"location":"48.8584,2.2945","types":["cafe"],"radius":500,"topN":1}),
    |v| {
        assert_eq!(v["categories"][0]["type"], "cafe");
        assert!(nonempty_string(
            &v["categories"][0]["top"][0]["details"]["id"]
        ));
    }
);

live_test!(
    live_maps_plan_route,
    "maps_plan_route",
    json!({"stops":["48.8584,2.2945","48.8606,2.3376","48.8635,2.3370","48.8530,2.3499"],"mode":"walking","optimize":true}),
    |v| {
        assert_eq!(v["optimized"], true);
        assert_eq!(v["stops"].as_array().unwrap().len(), 4);
        assert_eq!(v["legs"].as_array().unwrap().len(), 3);
        assert_eq!(v["stops"][0]["originalName"], "48.8584,2.2945");
        assert_eq!(v["stops"][3]["originalName"], "48.8530,2.3499");
    }
);

live_test!(
    live_maps_compare_places,
    "maps_compare_places",
    json!({"query":"cafe near Eiffel Tower Paris","limit":1,"userLocation":location()}),
    |v| {
        assert_eq!(v["places"].as_array().unwrap().len(), 1);
        assert!(nonempty_string(&v["places"][0]["details"]["id"]));
        assert!(v["places"][0]["distance"]["value"].as_f64().is_some());
    }
);

live_test!(
    live_maps_search_along_route,
    "maps_search_along_route",
    json!({"textQuery":"cafe","origin":"48.8584,2.2945","destination":"48.8606,2.3376","mode":"walking","maxResults":1}),
    |v| {
        assert!(nonempty_string(&v["places"][0]["id"]));
        assert!(nonempty_string(
            &v["route"]["routes"][0]["polyline"]["encodedPolyline"]
        ));
    }
);

live_test!(
    live_maps_local_rank_tracker,
    "maps_local_rank_tracker",
    json!({"keyword":"tourist attraction","placeId":EIFFEL_TOWER_ID,"center":location(),"gridSize":3,"gridSpacing":500}),
    |v| {
        assert_eq!(v["keywords"][0]["grid"].as_array().unwrap().len(), 9);
        assert_eq!(v["keywords"][0]["metrics"]["failed_points"], 0);
        assert!(v["keywords"][0]["metrics"]["atrp"].as_f64().is_some());
    }
);
