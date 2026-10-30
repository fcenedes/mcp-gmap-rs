use chrono::DateTime;
use serde_json::{Value, json};

use crate::client::{Error, MapsClient, Result};

#[derive(Clone, Debug)]
pub(crate) struct ResolvedLocation {
    pub original: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub place_id: Option<String>,
}

impl ResolvedLocation {
    pub fn waypoint(&self) -> Value {
        match &self.place_id {
            Some(id) => json!({"placeId":id}),
            None => {
                json!({"location":{"latLng":{"latitude":self.latitude,"longitude":self.longitude}}})
            }
        }
    }

    pub fn display(&self) -> Value {
        json!({"originalName":self.original,"address":self.address,"latitude":self.latitude,"longitude":self.longitude,"placeId":self.place_id})
    }
}

pub(crate) fn coordinates(input: &str) -> Result<Option<(f64, f64)>> {
    let Some((lat, lng)) = input.split_once(',') else {
        return Ok(None);
    };
    let (Ok(lat), Ok(lng)) = (lat.trim().parse::<f64>(), lng.trim().parse::<f64>()) else {
        return Ok(None);
    };
    check_coordinates(lat, lng)?;
    Ok(Some((lat, lng)))
}

pub(crate) fn check_coordinates(latitude: f64, longitude: f64) -> Result<()> {
    if !latitude.is_finite()
        || !longitude.is_finite()
        || !(-90.0..=90.0).contains(&latitude)
        || !(-180.0..=180.0).contains(&longitude)
    {
        return Err(Error::Input(
            "coordinates must have latitude -90..90 and longitude -180..180".into(),
        ));
    }
    Ok(())
}

impl MapsClient {
    pub async fn resolve(&self, input: &str) -> Result<ResolvedLocation> {
        if let Some((latitude, longitude)) = coordinates(input)? {
            // Never move an exact coordinate to the nearest geocoded address/place.
            return Ok(ResolvedLocation {
                original: input.into(),
                address: input.into(),
                latitude,
                longitude,
                place_id: None,
            });
        }
        match self.geocode(input).await {
            Ok(data) => resolved(
                input,
                &data["formatted_address"],
                &data["location"]["lat"],
                &data["location"]["lng"],
                &data["place_id"],
            ),
            Err(Error::Api(message)) if message == "no location found" => {
                let places = self
                    .search_text(
                        &json!({"textQuery":input,"pageSize":1,"languageCode":self.language}),
                    )
                    .await?;
                let place = places.first().ok_or_else(|| {
                    Error::Api("no location found using Geocoding or Places".into())
                })?;
                resolved(
                    input,
                    &place["formattedAddress"],
                    &place["location"]["latitude"],
                    &place["location"]["longitude"],
                    &place["id"],
                )
            }
            Err(error) => Err(error),
        }
    }

    pub async fn compute_route(
        &self,
        stops: &[ResolvedLocation],
        options: &Value,
        optimize: bool,
    ) -> Result<Value> {
        let mode = travel_mode(options);
        if stops.len() < 2 || stops.len() > 27 {
            return Err(Error::Input("routes require 2-27 stops".into()));
        }
        if mode == "TRANSIT" && stops.len() > 2 {
            return Err(Error::Input(
                "transit routes do not support intermediate stops".into(),
            ));
        }
        let mut body = route_options(options)?;
        body["languageCode"] = json!(self.language);
        body["origin"] = stops[0].waypoint();
        body["destination"] = stops[stops.len() - 1].waypoint();
        if stops.len() > 2 {
            body["intermediates"] = json!(
                stops[1..stops.len() - 1]
                    .iter()
                    .map(ResolvedLocation::waypoint)
                    .collect::<Vec<_>>()
            );
        }
        if optimize && stops.len() > 3 && mode != "TRANSIT" {
            body["optimizeWaypointOrder"] = json!(true);
        }
        let data = self
            .post(
                "routes.googleapis.com",
                "/directions/v2:computeRoutes",
                &body,
                Some(ROUTE_MASK),
            )
            .await?;
        let route = data["routes"]
            .as_array()
            .and_then(|r| r.first())
            .ok_or_else(|| Error::Api("no route found for the selected travel mode".into()))?;
        let meters = route["distanceMeters"].as_f64().unwrap_or(0.);
        let seconds = duration_seconds(&route["duration"]);
        Ok(
            json!({"routes":data["routes"],"summary":route["description"],"total_distance":{"value":meters,"text":distance_text(meters)},"total_duration":{"value":seconds,"text":duration_text(seconds)},"optimizedIntermediateWaypointIndex":route["optimizedIntermediateWaypointIndex"]}),
        )
    }

    pub async fn matrix(
        &self,
        origins: &[String],
        destinations: &[String],
        options: &Value,
    ) -> Result<Value> {
        let limit = if travel_mode(options) == "TRANSIT" {
            100
        } else {
            625
        };
        if origins.is_empty()
            || destinations.is_empty()
            || origins.len() * destinations.len() > limit
        {
            return Err(Error::Input(format!(
                "route matrix requires 1-{limit} origin/destination pairs"
            )));
        }
        let mut body = route_options(options)?;
        body["languageCode"] = json!(self.language);
        let modifiers = body
            .as_object_mut()
            .and_then(|o| o.remove("routeModifiers"));
        let origin_waypoints = origins
            .iter()
            .map(|s| {
                waypoint_input(s).map(|w| {
                    let mut origin = json!({"waypoint":w});
                    if let Some(m) = &modifiers {
                        origin["routeModifiers"] = m.clone();
                    }
                    origin
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let destination_waypoints = destinations
            .iter()
            .map(|s| waypoint_input(s).map(|w| json!({"waypoint":w})))
            .collect::<Result<Vec<_>>>()?;
        let addresses = origin_waypoints
            .iter()
            .chain(destination_waypoints.iter())
            .filter(|w| w["waypoint"].get("address").is_some())
            .count();
        if addresses > 50 {
            return Err(Error::Input(
                "route matrix supports at most 50 address/place-ID waypoints".into(),
            ));
        }
        body["origins"] = json!(origin_waypoints);
        body["destinations"] = json!(destination_waypoints);
        let elements = self
            .post(
                "routes.googleapis.com",
                "/distanceMatrix/v2:computeRouteMatrix",
                &body,
                Some("originIndex,destinationIndex,distanceMeters,duration,status,condition"),
            )
            .await?;
        let elements = elements.as_array().ok_or(Error::Response)?;
        let mut distances = vec![vec![Value::Null; destinations.len()]; origins.len()];
        let mut durations = distances.clone();
        let mut errors = Vec::new();
        let mut seen = vec![vec![false; destinations.len()]; origins.len()];
        for element in elements {
            // Protobuf JSON can omit zero-valued indices.
            let i = element["originIndex"].as_u64().unwrap_or(0) as usize;
            let j = element["destinationIndex"].as_u64().unwrap_or(0) as usize;
            if i >= origins.len() || j >= destinations.len() || seen[i][j] {
                return Err(Error::Response);
            }
            seen[i][j] = true;
            if element["status"]["code"].as_i64().unwrap_or(0) != 0
                || element["condition"] != "ROUTE_EXISTS"
            {
                errors.push(json!({"originIndex":i,"destinationIndex":j,"status":element["status"],"condition":element["condition"]}));
                continue;
            }
            let meters = element["distanceMeters"].as_f64().unwrap_or(0.);
            let seconds = duration_seconds(&element["duration"]);
            distances[i][j] = json!({"value":meters,"text":distance_text(meters)});
            durations[i][j] = json!({"value":seconds,"text":duration_text(seconds)});
        }
        for (i, row) in seen.iter().enumerate() {
            for (j, present) in row.iter().enumerate() {
                if !present {
                    errors.push(json!({"originIndex":i,"destinationIndex":j,"error":"missing response element"}));
                }
            }
        }
        Ok(
            json!({"origin_addresses":origins,"destination_addresses":destinations,"distances":distances,"durations":durations,"errors":errors}),
        )
    }
}

fn resolved(
    input: &str,
    address: &Value,
    lat: &Value,
    lng: &Value,
    id: &Value,
) -> Result<ResolvedLocation> {
    let latitude = lat.as_f64().ok_or(Error::Response)?;
    let longitude = lng.as_f64().ok_or(Error::Response)?;
    check_coordinates(latitude, longitude)?;
    Ok(ResolvedLocation {
        original: input.into(),
        address: address.as_str().unwrap_or(input).into(),
        latitude,
        longitude,
        place_id: id.as_str().filter(|s| !s.is_empty()).map(str::to_owned),
    })
}

fn waypoint_input(input: &str) -> Result<Value> {
    Ok(match coordinates(input)? {
        Some((latitude, longitude)) => {
            json!({"location":{"latLng":{"latitude":latitude,"longitude":longitude}}})
        }
        None => json!({"address":input}),
    })
}

pub(crate) fn travel_mode(options: &Value) -> &'static str {
    match options["mode"].as_str().unwrap_or("driving") {
        "walking" => "WALK",
        "bicycling" => "BICYCLE",
        "transit" => "TRANSIT",
        _ => "DRIVE",
    }
}

pub(crate) fn route_options(options: &Value) -> Result<Value> {
    let mode = travel_mode(options);
    let avoid_tolls = options["avoid_tolls"].as_bool().unwrap_or(false);
    let avoid_highways = options["avoid_highways"].as_bool().unwrap_or(false);
    if (avoid_tolls || avoid_highways) && mode != "DRIVE" {
        return Err(Error::Input(
            "avoid_tolls and avoid_highways require driving mode".into(),
        ));
    }
    if options.get("arrival_time").is_some() && options.get("departure_time").is_some() {
        return Err(Error::Input(
            "provide departure_time or arrival_time, not both".into(),
        ));
    }
    if options.get("arrival_time").is_some() && mode != "TRANSIT" {
        return Err(Error::Input("arrival_time requires transit mode".into()));
    }
    let mut body = json!({"travelMode":mode,"languageCode":"en"});
    if mode == "DRIVE" {
        body["routingPreference"] = json!("TRAFFIC_AWARE");
    }
    if avoid_tolls || avoid_highways {
        body["routeModifiers"] = json!({"avoidTolls":avoid_tolls,"avoidHighways":avoid_highways});
    }
    for (source, target) in [
        ("departure_time", "departureTime"),
        ("arrival_time", "arrivalTime"),
    ] {
        if let Some(time) = options[source].as_str() {
            let date = DateTime::parse_from_rfc3339(time).map_err(|_| {
                Error::Input(format!(
                    "{source} must be RFC 3339 with a timezone, e.g. 2027-01-01T09:00:00Z"
                ))
            })?;
            body[target] = json!(date.to_rfc3339());
        }
    }
    Ok(body)
}

pub(crate) fn duration_seconds(value: &Value) -> f64 {
    value
        .as_str()
        .and_then(|s| s.strip_suffix('s'))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.)
}
pub(crate) fn distance_text(meters: f64) -> String {
    if meters >= 1000. {
        format!("{:.1} km", meters / 1000.)
    } else {
        format!("{meters} m")
    }
}
pub(crate) fn duration_text(seconds: f64) -> String {
    format!("{} min", (seconds / 60.).round())
}

const ROUTE_MASK: &str = "routes.distanceMeters,routes.duration,routes.description,routes.polyline,routes.legs,routes.optimizedIntermediateWaypointIndex,routes.warnings";
