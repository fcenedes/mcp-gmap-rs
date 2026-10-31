//! Normalize Google Routes traffic intervals without inventing road attributes.
//!
//! Native advisories and polylines remain unchanged. Google's interval convention
//! is start-inclusive/end-exclusive; the end index is a boundary vertex, so an
//! interval's drawable subpath also includes that boundary point. Route and leg
//! indexes always refer to their own polylines and must never be interchanged.

use serde_json::{Value, json};

use crate::client::{Error, Result};

// Match the upstream response-size bound. Counting does not allocate decoded
// geometry, but this also bounds work when this pure function is called directly.
const MAX_ENCODED_BYTES: usize = 16 * 1024 * 1024;

/// Add explicit interval data and availability metadata at route and leg scope.
///
/// Validation finishes for every scope before any fields are inserted, so a bad
/// leg does not leave a partially enriched route behind.
pub(crate) fn enrich(
    route: &mut Value,
    route_index: usize,
    retrieved_at: &str,
    requested_departure: Option<&str>,
) -> Result<()> {
    if !route.is_object() {
        return Err(Error::Response);
    }
    let route_result = normalize(route, route_index, None, retrieved_at, requested_departure)?;
    let legs = match route.get("legs") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(legs)) => legs
            .iter()
            .enumerate()
            .map(|(leg_index, leg)| {
                normalize(
                    leg,
                    route_index,
                    Some(leg_index),
                    retrieved_at,
                    requested_departure,
                )
            })
            .collect::<Result<Vec<_>>>()?,
        _ => return Err(Error::Response),
    };

    route["segments"] = route_result.0;
    route["segmentMetadata"] = route_result.1;
    if let Some(Value::Array(native_legs)) = route.get_mut("legs") {
        for (leg, (segments, metadata)) in native_legs.iter_mut().zip(legs) {
            leg["segments"] = segments;
            leg["segmentMetadata"] = metadata;
        }
    }
    Ok(())
}

fn normalize(
    scope: &Value,
    route_index: usize,
    leg_index: Option<usize>,
    retrieved_at: &str,
    requested_departure: Option<&str>,
) -> Result<(Value, Value)> {
    if !scope.is_object() {
        return Err(Error::Response);
    }
    let encoded = match scope.get("polyline") {
        None | Some(Value::Null) => None,
        Some(Value::Object(polyline)) => match polyline.get("encodedPolyline") {
            None | Some(Value::Null) => None,
            Some(Value::String(encoded)) => Some(encoded.as_str()),
            _ => return Err(Error::Response),
        },
        _ => return Err(Error::Response),
    };
    let point_count = encoded.map_or(Ok(0), count_points)?;
    let intervals = match scope.get("travelAdvisory") {
        None | Some(Value::Null) => None,
        Some(Value::Object(advisory)) => match advisory.get("speedReadingIntervals") {
            None | Some(Value::Null) => None,
            Some(Value::Array(intervals)) => Some(intervals),
            _ => return Err(Error::Response),
        },
        _ => return Err(Error::Response),
    };
    let mut segments = Vec::new();
    let mut partial = false;
    let mut previous_end = 0;
    if let Some(intervals) = intervals {
        for interval in intervals {
            let interval = interval.as_object().ok_or(Error::Response)?;
            let start = match interval.get("startPolylinePointIndex") {
                None => 0,
                Some(value) => value.as_u64().ok_or(Error::Response)?,
            };
            let end = interval
                .get("endPolylinePointIndex")
                .and_then(Value::as_u64)
                .ok_or(Error::Response)?;
            // A route with M vertices has M-1 edges, and the final edge ends at
            // vertex M-1. Google uses that vertex as the final boundary index.
            let last_index = point_count.checked_sub(1).ok_or(Error::Response)? as u64;
            if start >= end || end > last_index || start < previous_end {
                return Err(Error::Response);
            }
            if start != previous_end {
                partial = true;
            }
            let category = match interval.get("speed") {
                None | Some(Value::Null) => Value::Null,
                Some(Value::String(speed)) => Value::String(speed.clone()),
                _ => return Err(Error::Response),
            };
            if !matches!(category.as_str(), Some("NORMAL" | "SLOW" | "TRAFFIC_JAM")) {
                partial = true;
            }
            segments.push(json!({
                "startPolylinePointIndex":start,
                "endPolylinePointIndex":end,
                "trafficCategory":category,
                "roadCategory":null,
                "speedLimit":null,
                "trafficObservedAt":null
            }));
            previous_end = end;
        }
    }
    let status = if segments.is_empty() {
        "NOT_RETURNED"
    } else if partial || previous_end != point_count.saturating_sub(1) as u64 {
        "PARTIAL"
    } else {
        "AVAILABLE"
    };
    let mut metadata = json!({
        "polylineScope":if leg_index.is_some() { "LEG" } else { "ROUTE" },
        "routeIndex":route_index,
        "indexConvention":"START_INCLUSIVE_END_EXCLUSIVE",
        "retrievedAt":retrieved_at,
        "requestedDepartureTime":requested_departure,
        "trafficSource":"GOOGLE_ROUTES",
        "trafficStatus":status,
        "roadCategoryStatus":"NOT_PROVIDED_BY_ROUTES",
        "speedLimitStatus":"NOT_PROVIDED_BY_ROUTES"
    });
    if let Some(leg_index) = leg_index {
        metadata["legIndex"] = json!(leg_index);
    }
    Ok((Value::Array(segments), metadata))
}

/// Validate the encoded Google polyline and count vertices in one bounded pass.
///
/// Only running integer coordinates are retained. No geometry is resampled or
/// allocated, so provider interval indices keep their original meaning.
fn count_points(encoded: &str) -> Result<usize> {
    let bytes = encoded.as_bytes();
    if bytes.len() > MAX_ENCODED_BYTES {
        return Err(Error::Response);
    }
    let mut cursor = 0;
    let mut latitude = 0_i64;
    let mut longitude = 0_i64;
    let mut count = 0;
    while cursor < bytes.len() {
        latitude += delta(bytes, &mut cursor)?;
        longitude += delta(bytes, &mut cursor)?;
        if !(-9_000_000..=9_000_000).contains(&latitude)
            || !(-18_000_000..=18_000_000).contains(&longitude)
        {
            return Err(Error::Response);
        }
        count += 1;
    }
    Ok(count)
}

fn delta(bytes: &[u8], cursor: &mut usize) -> Result<i64> {
    let mut value = 0_u32;
    // Polyline components are signed 32-bit integers encoded in five-bit
    // groups. Seven groups suffice; the seventh may contain only two bits.
    for group in 0..7 {
        let byte = bytes.get(*cursor).copied().ok_or(Error::Response)?;
        *cursor += 1;
        if !(b'?'..=b'~').contains(&byte) {
            return Err(Error::Response);
        }
        let chunk = u32::from(byte - b'?');
        let payload = chunk & 0x1f;
        if group == 6 && payload > 3 {
            return Err(Error::Response);
        }
        value |= payload << (group * 5);
        if chunk & 0x20 == 0 {
            return Ok(i64::from(value >> 1) ^ -i64::from(value & 1));
        }
    }
    Err(Error::Response)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RETRIEVED: &str = "2026-11-01T12:34:56Z";
    // Google's five-vertex traffic-polyline example ends its last interval at 4.
    const GOOGLE: &str = "}boeF~zbjVAg@EmB`GWHlD";

    fn route(polyline: &str, intervals: Value) -> Value {
        json!({
            "polyline":{"encodedPolyline":polyline},
            "travelAdvisory":{"speedReadingIntervals":intervals,"tollInfo":{"estimatedPrice":[{"currencyCode":"EUR"}]}}
        })
    }

    #[test]
    fn normalizes_google_example_and_keeps_provider_fields_exact() {
        let mut route = route(
            GOOGLE,
            json!([
                {"endPolylinePointIndex":1,"speed":"NORMAL"},
                {"startPolylinePointIndex":1,"endPolylinePointIndex":2,"speed":"SLOW"},
                {"startPolylinePointIndex":2,"endPolylinePointIndex":4,"speed":"NORMAL"}
            ]),
        );
        let native = route.clone();
        enrich(&mut route, 2, RETRIEVED, Some("2026-11-01T13:00:00Z")).unwrap();
        assert_eq!(route["polyline"], native["polyline"]);
        assert_eq!(route["travelAdvisory"], native["travelAdvisory"]);
        assert_eq!(
            route["segments"][0],
            json!({"startPolylinePointIndex":0,"endPolylinePointIndex":1,"trafficCategory":"NORMAL","roadCategory":null,"speedLimit":null,"trafficObservedAt":null})
        );
        assert_eq!(route["segments"][2]["endPolylinePointIndex"], 4);
        assert_eq!(
            route["segmentMetadata"],
            json!({"polylineScope":"ROUTE","routeIndex":2,"indexConvention":"START_INCLUSIVE_END_EXCLUSIVE","retrievedAt":RETRIEVED,"requestedDepartureTime":"2026-11-01T13:00:00Z","trafficSource":"GOOGLE_ROUTES","trafficStatus":"AVAILABLE","roadCategoryStatus":"NOT_PROVIDED_BY_ROUTES","speedLimitStatus":"NOT_PROVIDED_BY_ROUTES"})
        );
    }

    #[test]
    fn route_and_leg_geometry_have_independent_indices_and_availability() {
        let mut route = route(GOOGLE, json!([{"endPolylinePointIndex":4,"speed":"SLOW"}]));
        route["legs"] = json!([
            {"polyline":{"encodedPolyline":"????"},"travelAdvisory":{"speedReadingIntervals":[{"endPolylinePointIndex":1,"speed":"TRAFFIC_JAM"}]}},
            {"polyline":{"encodedPolyline":"??????"}}
        ]);
        enrich(&mut route, 7, RETRIEVED, None).unwrap();
        assert_eq!(route["segmentMetadata"]["trafficStatus"], "AVAILABLE");
        assert_eq!(route["legs"][0]["segments"][0]["endPolylinePointIndex"], 1);
        assert_eq!(route["legs"][0]["segmentMetadata"]["polylineScope"], "LEG");
        assert_eq!(route["legs"][0]["segmentMetadata"]["routeIndex"], 7);
        assert_eq!(route["legs"][0]["segmentMetadata"]["legIndex"], 0);
        assert_eq!(route["legs"][1]["segmentMetadata"]["legIndex"], 1);
        assert_eq!(
            route["legs"][1]["segmentMetadata"]["trafficStatus"],
            "NOT_RETURNED"
        );
        assert_eq!(route["legs"][1]["segments"], json!([]));
        assert!(route["segmentMetadata"]["requestedDepartureTime"].is_null());
        assert!(route["segmentMetadata"].get("legIndex").is_none());
    }

    #[test]
    fn gaps_and_missing_or_unknown_categories_stay_partial_without_filling() {
        for intervals in [
            json!([{"startPolylinePointIndex":1,"endPolylinePointIndex":4,"speed":"NORMAL"}]),
            json!([{"endPolylinePointIndex":3,"speed":"NORMAL"}]),
            json!([{"endPolylinePointIndex":1,"speed":"NORMAL"},{"startPolylinePointIndex":2,"endPolylinePointIndex":4,"speed":"SLOW"}]),
            json!([{"endPolylinePointIndex":4}]),
            json!([{"endPolylinePointIndex":4,"speed":null}]),
            json!([{"endPolylinePointIndex":4,"speed":"SPEED_UNSPECIFIED"}]),
            json!([{"endPolylinePointIndex":4,"speed":"FUTURE_GOOGLE_CATEGORY"}]),
        ] {
            let expected_len = intervals.as_array().unwrap().len();
            let expected_category = intervals[0].get("speed").cloned().unwrap_or(Value::Null);
            let mut route = route(GOOGLE, intervals);
            enrich(&mut route, 0, RETRIEVED, None).unwrap();
            assert_eq!(route["segmentMetadata"]["trafficStatus"], "PARTIAL");
            assert_eq!(route["segments"].as_array().unwrap().len(), expected_len);
            assert_eq!(route["segments"][0]["trafficCategory"], expected_category);
            assert!(route["segments"][0]["speedLimit"].is_null());
            assert!(route["segments"][0]["trafficObservedAt"].is_null());
        }
    }

    #[test]
    fn no_traffic_is_reported_explicitly_even_without_geometry() {
        for mut route in [
            json!({}),
            json!({"polyline":{"encodedPolyline":""}}),
            route(GOOGLE, json!([])),
            route(GOOGLE, Value::Null),
        ] {
            enrich(&mut route, 0, RETRIEVED, None).unwrap();
            assert_eq!(route["segments"], json!([]));
            assert_eq!(route["segmentMetadata"]["trafficStatus"], "NOT_RETURNED");
        }
    }

    #[test]
    fn rejects_overlap_order_invalid_boundaries_and_corrupt_interval_types() {
        for intervals in [
            json!([{"endPolylinePointIndex":3,"speed":"NORMAL"},{"startPolylinePointIndex":2,"endPolylinePointIndex":4,"speed":"SLOW"}]),
            json!([{"startPolylinePointIndex":2,"endPolylinePointIndex":4,"speed":"NORMAL"},{"endPolylinePointIndex":1,"speed":"SLOW"}]),
            json!([{"endPolylinePointIndex":5,"speed":"NORMAL"}]),
            json!([{"startPolylinePointIndex":-1,"endPolylinePointIndex":4}]),
            json!([{"endPolylinePointIndex":-1}]),
            json!([{"startPolylinePointIndex":4,"endPolylinePointIndex":4}]),
            json!([{"startPolylinePointIndex":3,"endPolylinePointIndex":2}]),
            json!([{"startPolylinePointIndex":null,"endPolylinePointIndex":4}]),
            json!([{"endPolylinePointIndex":1.5}]),
            json!([{"endPolylinePointIndex":4,"speed":1}]),
            json!([{"speed":"NORMAL"}]),
            json!([null]),
            json!({"endPolylinePointIndex":4}),
        ] {
            let mut route = route(GOOGLE, intervals);
            let before = route.clone();
            assert!(matches!(
                enrich(&mut route, 0, RETRIEVED, None),
                Err(Error::Response)
            ));
            assert_eq!(route, before);
        }
    }

    #[test]
    fn invalid_leg_does_not_partially_enrich_route() {
        let mut route = route(
            GOOGLE,
            json!([{"endPolylinePointIndex":4,"speed":"NORMAL"}]),
        );
        route["legs"] = json!([{"polyline":{"encodedPolyline":"????"},"travelAdvisory":{"speedReadingIntervals":[{"endPolylinePointIndex":4,"speed":"NORMAL"}]}}]);
        let original = route.clone();
        assert!(matches!(
            enrich(&mut route, 0, RETRIEVED, None),
            Err(Error::Response)
        ));
        assert_eq!(route, original);
    }

    #[test]
    fn validates_polyline_encoding_in_one_pass_without_resampling() {
        // Standard Google encoding example: three distinct WGS84 points.
        assert_eq!(count_points("_p~iF~ps|U_ulLnnqC_mqNvxq`@").unwrap(), 3);
        assert_eq!(count_points(GOOGLE).unwrap(), 5);
        assert_eq!(count_points("????").unwrap(), 2);
        assert_eq!(count_points("").unwrap(), 0);
        for encoded in [
            "?",
            "_",
            "????_",
            "!??",
            "é",
            "~~~~~~~~?",
            "~~~~~~D?",
            "_gsia@?",
        ] {
            assert!(
                matches!(count_points(encoded), Err(Error::Response)),
                "{encoded:?}"
            );
        }
        let oversized = "?".repeat(MAX_ENCODED_BYTES + 1);
        assert!(matches!(count_points(&oversized), Err(Error::Response)));
    }

    #[test]
    fn intervals_require_usable_geometry_and_invalid_native_shapes_fail() {
        for mut route in [
            route("", json!([{"endPolylinePointIndex":1,"speed":"NORMAL"}])),
            route("??", json!([{"endPolylinePointIndex":1,"speed":"NORMAL"}])),
            route("_", json!([])),
            json!({"travelAdvisory":{"speedReadingIntervals":[{"endPolylinePointIndex":1}]}}),
            json!({"polyline":{"encodedPolyline":42}}),
            json!({"polyline":[]}),
            json!({"travelAdvisory":[]}),
            json!({"legs":{}}),
            json!({"legs":[null]}),
            json!([]),
        ] {
            assert!(matches!(
                enrich(&mut route, 0, RETRIEVED, None),
                Err(Error::Response)
            ));
        }
    }
}
