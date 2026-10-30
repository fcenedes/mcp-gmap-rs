use futures::{StreamExt, stream};
use serde_json::{Value, json};

use crate::{
    client::{Error, MapsClient, Result},
    routes::{ResolvedLocation, duration_seconds, route_options, travel_mode},
    tools::{circle, strings, text},
};

impl MapsClient {
    pub async fn batch_geocode(&self, addresses: &[String]) -> Value {
        let requests = addresses
            .iter()
            .map(|address| async move {
                match self.geocode(address).await {
                    Ok(data) => json!({"address":address,"success":true,"data":data}),
                    Err(error) => {
                        json!({"address":address,"success":false,"error":error.to_string()})
                    }
                }
            })
            .collect::<Vec<_>>();
        let results = stream::iter(requests).buffered(5).collect::<Vec<_>>().await;
        json!(results)
    }

    pub async fn explore(&self, a: &Value) -> Result<Value> {
        let location = self.resolve(text(a, "location")).await?;
        let types = if a.get("types").is_some() {
            strings(a, "types")
        } else {
            vec![
                "restaurant".into(),
                "cafe".into(),
                "tourist_attraction".into(),
            ]
        };
        let radius = a["radius"].as_f64().unwrap_or(1000.);
        let top = a["topN"].as_u64().unwrap_or(3) as usize;
        let mut categories = Vec::new();
        for kind in types {
            let params = json!({"center":{"value":format!("{},{}",location.latitude,location.longitude),"isCoordinates":true},"keyword":kind,"radius":radius});
            match self.nearby(&params).await {
                Ok(places) => {
                    let detailed = self.enrich(&places[..places.len().min(top)]).await;
                    categories.push(json!({"type":kind,"count":places.len(),"top":detailed}));
                }
                Err(error) => categories.push(json!({"type":kind,"error":error.to_string()})),
            }
        }
        Ok(json!({"location":location.display(),"radius":radius,"categories":categories}))
    }

    pub async fn plan(&self, a: &Value) -> Result<Value> {
        route_options(a)?;
        let names = strings(a, "stops");
        if travel_mode(a) == "TRANSIT" && names.len() > 2 {
            return Err(Error::Input(
                "transit routes do not support intermediate stops".into(),
            ));
        }
        let requests = names
            .iter()
            .map(|name| self.resolve(name))
            .collect::<Vec<_>>();
        let resolved = stream::iter(requests)
            .buffered(5)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
        let optimize = a["optimize"].as_bool().unwrap_or(true)
            && resolved.len() > 3
            && travel_mode(a) != "TRANSIT";
        let data = self.compute_route(&resolved, a, optimize).await?;
        let route = &data["routes"][0];
        let order = optimized_order(
            &resolved,
            &data["optimizedIntermediateWaypointIndex"],
            optimize,
        )?;
        let legs = route["legs"].as_array().ok_or(Error::Response)?;
        if legs.len() != order.len() - 1 {
            return Err(Error::Response);
        }
        let display_legs=legs.iter().enumerate().map(|(i,leg)|json!({"from":order[i].original,"to":order[i+1].original,"distanceMeters":leg["distanceMeters"],"durationSeconds":duration_seconds(&leg["duration"]),"steps":leg["steps"]})).collect::<Vec<_>>();
        Ok(
            json!({"mode":a["mode"].as_str().unwrap_or("driving"),"optimized":optimize,"stops":order.iter().map(|s|s.display()).collect::<Vec<_>>(),"legs":display_legs,"total_distance":data["total_distance"],"total_duration":data["total_duration"],"polyline":route["polyline"],"warnings":route["warnings"]}),
        )
    }

    pub async fn compare(&self, a: &Value) -> Result<Value> {
        let limit = a["limit"].as_u64().unwrap_or(5) as usize;
        let places = self
            .search_text(
                &json!({"textQuery":a["query"],"pageSize":limit,"languageCode":self.language}),
            )
            .await?;
        let places = &places[..places.len().min(limit)];
        let mut detailed = self.enrich(places).await;
        let mut distance_error = None;
        if let Some(location) = a.get("userLocation")
            && !places.is_empty()
        {
            let origins = vec![format!(
                "{},{}",
                location["latitude"], location["longitude"]
            )];
            let destinations = places
                .iter()
                .map(|p| {
                    format!(
                        "{},{}",
                        p["location"]["latitude"], p["location"]["longitude"]
                    )
                })
                .collect::<Vec<_>>();
            match self.matrix(&origins, &destinations, &json!({})).await {
                Ok(matrix) => {
                    for (i, place) in detailed.iter_mut().enumerate() {
                        place["distance"] = matrix["distances"][0][i].clone();
                        place["drive_time"] = matrix["durations"][0][i].clone();
                        let errors = matrix["errors"]
                            .as_array()
                            .map(|e| {
                                e.iter()
                                    .filter(|e| e["destinationIndex"].as_u64() == Some(i as u64))
                                    .cloned()
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        if !errors.is_empty() {
                            place["distance_errors"] = json!(errors);
                        }
                    }
                }
                Err(error) => distance_error = Some(error.to_string()),
            }
        }
        Ok(json!({"places":detailed,"distance_error":distance_error}))
    }

    async fn enrich(&self, places: &[Value]) -> Vec<Value> {
        let requests = places
            .iter()
            .map(|place| async move {
                let mut result = place.clone();
                match self.details(text(place, "id"), 0).await {
                    Ok(details) => result["details"] = details,
                    Err(error) => result["details_error"] = json!(error.to_string()),
                }
                result
            })
            .collect::<Vec<_>>();
        stream::iter(requests).buffered(5).collect().await
    }

    pub async fn along_route(&self, a: &Value) -> Result<Value> {
        let mut options = a.clone();
        if options.get("mode").is_none() {
            options["mode"] = json!("walking");
        }
        let origin = self.resolve(text(a, "origin")).await?;
        let destination = self.resolve(text(a, "destination")).await?;
        let route = self
            .compute_route(&[origin, destination], &options, false)
            .await?;
        let polyline = route["routes"][0]["polyline"]["encodedPolyline"]
            .as_str()
            .ok_or_else(|| Error::Api("route has no polyline".into()))?;
        let mask = format!("{},routingSummaries", crate::client::SEARCH_MASK);
        let data=self.post("places.googleapis.com","/v1/places:searchText",&json!({"textQuery":a["textQuery"],"pageSize":a["maxResults"].as_u64().unwrap_or(5),"languageCode":self.language,"searchAlongRouteParameters":{"polyline":{"encodedPolyline":polyline}}}),Some(&mask)).await?;
        Ok(
            json!({"places":data["places"].as_array().cloned().unwrap_or_default(),"routingSummaries":data["routingSummaries"],"route":route}),
        )
    }

    pub async fn rank(&self, a: &Value) -> Result<Value> {
        let size = a["gridSize"].as_u64().unwrap_or(3) as usize;
        let spacing = a["gridSpacing"].as_f64().unwrap_or(1000.);
        let lat = a["center"]["latitude"].as_f64().ok_or(Error::Response)?;
        let lng = a["center"]["longitude"].as_f64().ok_or(Error::Response)?;
        let keywords = if a.get("keywords").is_some() {
            strings(a, "keywords")
        } else {
            vec![text(a, "keyword").into()]
        };
        let target = text(a, "placeId")
            .strip_prefix("places/")
            .unwrap_or(text(a, "placeId"));
        let mut scans = Vec::new();
        for keyword in keywords {
            let half = (size as f64 - 1.) / 2.;
            let points = (0..size)
                .flat_map(|row| {
                    (0..size).map(move |col| {
                        let (latitude, longitude) = offset_point(
                            lat,
                            lng,
                            (row as f64 - half) * spacing,
                            (col as f64 - half) * spacing,
                        );
                        (row, col, latitude, longitude)
                    })
                })
                .collect::<Vec<_>>();
            let grid=stream::iter(points.into_iter().map(|(row,col,latitude,longitude)| {
                let keyword=&keyword;
                async move {
                    let body=json!({"textQuery":keyword,"pageSize":20,"languageCode":self.language,"locationBias":circle(latitude,longitude,spacing/2.)});
                    match self.search_text(&body).await {
                        Ok(places)=>{
                            let rank=places.iter().position(|p|p["id"].as_str()==Some(target)).map(|i|i+1);
                            json!({"row":row,"col":col,"lat":latitude,"lng":longitude,"rank":rank,"top3":places.iter().take(3).map(|p|json!({"id":p["id"],"displayName":p["displayName"],"googleMapsUri":p["googleMapsUri"]})).collect::<Vec<_>>()})
                        }
                        Err(error)=>json!({"row":row,"col":col,"lat":latitude,"lng":longitude,"rank":null,"error":error.to_string()}),
                    }
                }
            })).buffered(5).collect::<Vec<_>>().await;
            let valid = grid
                .iter()
                .filter(|p| p.get("error").is_none())
                .collect::<Vec<_>>();
            let ranks = valid
                .iter()
                .filter_map(|p| p["rank"].as_u64())
                .collect::<Vec<_>>();
            let arp = if ranks.is_empty() {
                None
            } else {
                Some(ranks.iter().sum::<u64>() as f64 / ranks.len() as f64)
            };
            let atrp = if valid.is_empty() {
                None
            } else {
                Some(
                    valid
                        .iter()
                        .map(|p| p["rank"].as_u64().unwrap_or(21))
                        .sum::<u64>() as f64
                        / valid.len() as f64,
                )
            };
            let solv = if valid.is_empty() {
                None
            } else {
                Some(ranks.iter().filter(|r| **r <= 3).count() as f64 / valid.len() as f64 * 100.)
            };
            scans.push(json!({"keyword":keyword,"metrics":{"arp":arp,"atrp":atrp,"solv":solv,"found_in":format!("{}/{}",ranks.len(),valid.len()),"failed_points":grid.len()-valid.len()},"grid":grid}));
        }
        Ok(
            json!({"target":{"place_id":target},"grid_size":format!("{size}x{size}"),"grid_spacing_m":spacing,"keywords":scans}),
        )
    }
}

fn optimized_order<'a>(
    stops: &'a [ResolvedLocation],
    indices: &Value,
    optimize: bool,
) -> Result<Vec<&'a ResolvedLocation>> {
    if !optimize {
        return Ok(stops.iter().collect());
    }
    let indices = indices.as_array().ok_or(Error::Response)?;
    if indices.len() != stops.len() - 2 {
        return Err(Error::Response);
    }
    let mut seen = vec![false; indices.len()];
    let mut ordered = vec![&stops[0]];
    for index in indices {
        let i = index.as_u64().ok_or(Error::Response)? as usize;
        if i >= seen.len() || seen[i] {
            return Err(Error::Response);
        }
        seen[i] = true;
        ordered.push(&stops[i + 1]);
    }
    ordered.push(&stops[stops.len() - 1]);
    Ok(ordered)
}

// A spherical offset keeps grid coordinates valid at poles and across the date line.
fn offset_point(latitude: f64, longitude: f64, north: f64, east: f64) -> (f64, f64) {
    let angle = north.hypot(east) / 6_371_000.;
    let bearing = east.atan2(north);
    let lat = latitude.to_radians();
    let lng = longitude.to_radians();
    let dest_lat = (lat.sin() * angle.cos() + lat.cos() * angle.sin() * bearing.cos())
        .clamp(-1., 1.)
        .asin();
    let dest_lng = lng
        + (bearing.sin() * angle.sin() * lat.cos()).atan2(angle.cos() - lat.sin() * dest_lat.sin());
    (
        dest_lat.to_degrees(),
        (dest_lng.to_degrees() + 180.).rem_euclid(360.) - 180.,
    )
}
