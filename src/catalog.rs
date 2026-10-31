use std::sync::Arc;

use rmcp::model::{Tool, ToolAnnotations};
use serde_json::{Value, json};

use crate::client::{Error, Result};

pub(crate) struct ToolDefinition {
    pub tool: Tool,
    validator: jsonschema::Validator,
}

impl ToolDefinition {
    pub fn validate(&self, input: &Value) -> Result<()> {
        if let Some(error) = self.validator.iter_errors(input).next() {
            // Do not echo user input or credentials back in validation errors.
            return Err(Error::Input(format!(
                "invalid arguments at {} (schema constraint {}); use `schema` to inspect accepted input",
                error.instance_path(),
                error.schema_path()
            )));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct Catalog(Arc<Vec<ToolDefinition>>);

impl Catalog {
    pub fn new(filter: Option<&str>) -> Result<Self> {
        let mut definitions = Vec::new();
        let mut builtins = builtin_definitions();
        builtins.extend(crate::geo_services::definitions());
        builtins.extend(crate::media_services::definitions());
        builtins.extend(crate::cloud_services::definitions());
        for (name, description, schema) in builtins {
            let writes = matches!(
                name,
                "maps_render_aerial_video"
                    | "maps_datasets"
                    | "maps_route_optimization"
                    | "maps_streetview_publish"
                    | "maps_map_tiles"
            );
            let destructive = matches!(name, "maps_datasets" | "maps_streetview_publish");
            let validator = jsonschema::validator_for(&schema)
                .map_err(|_| Error::Input(format!("invalid built-in schema for {name}")))?;
            let schema = schema
                .as_object()
                .cloned()
                .ok_or_else(|| Error::Input("schema must be an object".into()))?;
            definitions.push(ToolDefinition {
                tool: Tool::new(name, description, schema).with_annotations(
                    ToolAnnotations::new()
                        .read_only(!writes)
                        .destructive(destructive)
                        .idempotent(!writes)
                        .open_world(true),
                ),
                validator,
            });
        }
        if let Some(filter) = filter.map(str::trim).filter(|s| !s.is_empty() && *s != "*") {
            let names: Vec<_> = filter
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            if names.is_empty()
                || names
                    .iter()
                    .any(|name| !definitions.iter().any(|d| d.tool.name == *name))
            {
                return Err(Error::Input("GOOGLE_MAPS_ENABLED_TOOLS contains an unknown tool or empty selection; use `tools` to list names".into()));
            }
            definitions.retain(|d| names.contains(&d.tool.name.as_ref()));
        }
        Ok(Self(Arc::new(definitions)))
    }

    pub fn iter(&self) -> impl Iterator<Item = &ToolDefinition> {
        self.0.iter()
    }

    pub fn get(&self, name: &str) -> Result<&ToolDefinition> {
        self.0
            .iter()
            .find(|d| d.tool.name == name)
            .ok_or_else(|| Error::Input(format!("unknown or disabled tool: {name}")))
    }
}

pub(crate) fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object", "properties":properties, "required":required, "additionalProperties":false})
}

pub(crate) fn string() -> Value {
    json!({"type":"string","minLength":1})
}
pub(crate) fn number(min: f64, max: f64) -> Value {
    json!({"type":"number","minimum":min,"maximum":max})
}
pub(crate) fn integer(min: u32, max: u32) -> Value {
    json!({"type":"integer","minimum":min,"maximum":max})
}
pub(crate) fn array(items: Value, min: u32, max: u32) -> Value {
    json!({"type":"array","items":items,"minItems":min,"maxItems":max})
}
pub(crate) fn boolean() -> Value {
    json!({"type":"boolean"})
}
pub(crate) fn coordinate() -> Value {
    object(
        json!({"latitude":number(-90.,90.),"longitude":number(-180.,180.)}),
        &["latitude", "longitude"],
    )
}
fn travel() -> Value {
    json!({"type":"string","enum":["driving","walking","bicycling","transit"]})
}

fn builtin_definitions() -> Vec<(&'static str, &'static str, Value)> {
    let lat = number(-90., 90.);
    let lng = number(-180., 180.);
    let bias = object(
        json!({"latitude":lat,"longitude":lng,"radius":number(0.1,50000.)}),
        &["latitude", "longitude"],
    );
    let route_options = json!({"mode":travel(),"departure_time":string(),"avoid_tolls":boolean(),"avoid_highways":boolean()});
    let mut directions = route_options.clone();
    directions["origin"] = string();
    directions["destination"] = string();
    directions["arrival_time"] = string();
    directions["trafficSegments"] = boolean();
    let mut matrix = route_options.clone();
    matrix["origins"] = array(string(), 1, 50);
    matrix["destinations"] = array(string(), 1, 50);
    let mut plan = route_options;
    plan["stops"] = array(string(), 2, 27);
    plan["optimize"] = boolean();
    plan["trafficSegments"] = boolean();
    vec![
        (
            "maps_geocode",
            "Convert an address or landmark to coordinates.",
            object(json!({"address":string()}), &["address"]),
        ),
        (
            "maps_validate_address",
            "Validate and standardize a postal address; returns Google's verdict, corrections, components and geocode. A verdict is evidence, not a delivery guarantee.",
            object(
                json!({
                    "addressLines":array(string(),1,10),
                    "regionCode":{"type":"string","pattern":"^[A-Z]{2}$"},
                    "locality":string(),"administrativeArea":string(),"postalCode":string(),
                    "enableUspsCass":boolean(),"sessionToken":string()
                }),
                &["addressLines"],
            ),
        ),
        (
            "maps_pollen",
            "Get 1-5 days of pollen forecasts, indices and health recommendations for exact coordinates. Coverage varies by location and season.",
            object(
                json!({"latitude":lat,"longitude":lng,"days":integer(1,5),"plantsDescription":boolean()}),
                &["latitude", "longitude"],
            ),
        ),
        (
            "maps_area_insights",
            "Count places within a circle, region or polygon using Places Aggregate. Optional place IDs are available only for counts of 100 or less; no automatic details requests.",
            object(
                json!({
                    "area":area_schema(),
                    "includedTypes":array(string(),1,50),"excludedTypes":array(string(),1,50),
                    "includePlaces":boolean(),"minRating":number(1.,5.),"maxRating":number(1.,5.),
                    "operatingStatus":array(json!({"type":"string","enum":["OPERATING_STATUS_OPERATIONAL","OPERATING_STATUS_PERMANENTLY_CLOSED","OPERATING_STATUS_TEMPORARILY_CLOSED"]}),1,3),
                    "priceLevels":array(json!({"type":"string","enum":["PRICE_LEVEL_FREE","PRICE_LEVEL_INEXPENSIVE","PRICE_LEVEL_MODERATE","PRICE_LEVEL_EXPENSIVE","PRICE_LEVEL_VERY_EXPENSIVE"]}),1,5)
                }),
                &["area", "includedTypes"],
            ),
        ),
        (
            "maps_snap_to_roads",
            "Snap an ordered GPS path of 2-100 points to roads. Keep consecutive points within about 300 metres for good results. Optional interpolation adds road geometry.",
            object(
                json!({"path":array(coordinate(),2,100),"interpolate":boolean()}),
                &["path"],
            ),
        ),
        (
            "maps_nearest_roads",
            "Find the nearest road segments for 1-100 independent GPS points; preserves Google's originalIndex and road place IDs.",
            object(json!({"points":array(coordinate(),1,100)}), &["points"]),
        ),
        (
            "maps_reverse_geocode",
            "Convert coordinates to a street address.",
            object(
                json!({"latitude":lat,"longitude":lng}),
                &["latitude", "longitude"],
            ),
        ),
        (
            "maps_search_places",
            "Search Places (New) by text with optional location and rating filters.",
            object(
                json!({"query":string(),"locationBias":bias,"openNow":boolean(),"minRating":number(0.,5.),"includedType":string()}),
                &["query"],
            ),
        ),
        (
            "maps_search_nearby",
            "Find places around an address or exact coordinate pair, filtered by type, rating, and opening status.",
            object(
                json!({"center":object(json!({"value":string(),"isCoordinates":boolean()}), &["value"]),"keyword":string(),"radius":number(0.1,50000.),"openNow":boolean(),"minRating":number(0.,5.)}),
                &["center"],
            ),
        ),
        (
            "maps_place_details",
            "Get full place details, reviews and attribution. Optional photo URLs retain author attribution. Display source/disclosure metadata alongside content.",
            object(
                json!({"placeId":string(),"maxPhotos":integer(0,10),"sessionToken":string()}),
                &["placeId"],
            ),
        ),
        (
            "maps_directions",
            "Get directions, route geometry, navigation instructions and travel duration.",
            object(directions, &["origin", "destination"]),
        ),
        (
            "maps_distance_matrix",
            "Compute travel distances and durations for multiple origins and destinations.",
            object(matrix, &["origins", "destinations"]),
        ),
        (
            "maps_elevation",
            "Get elevation in meters for geographic coordinates.",
            object(
                json!({"locations":array(coordinate(),1,512)}),
                &["locations"],
            ),
        ),
        (
            "maps_timezone",
            "Get timezone, UTC/DST offsets and local time. timestamp is Unix milliseconds.",
            object(
                json!({"latitude":lat,"longitude":lng,"timestamp":{"type":"integer","minimum":0,"maximum":253402300799999_i64}}),
                &["latitude", "longitude"],
            ),
        ),
        (
            "maps_weather",
            "Get current weather, daily forecast or hourly forecast.",
            object(
                json!({"latitude":lat,"longitude":lng,"type":{"type":"string","enum":["current","forecast_daily","forecast_hourly"]},"forecastDays":integer(1,10),"forecastHours":integer(1,240)}),
                &["latitude", "longitude"],
            ),
        ),
        (
            "maps_air_quality",
            "Get air quality indices, optional pollutant concentrations and health recommendations.",
            object(
                json!({"latitude":lat,"longitude":lng,"includeHealthRecommendations":boolean(),"includePollutants":boolean()}),
                &["latitude", "longitude"],
            ),
        ),
        (
            "maps_static_map",
            "Fetch a PNG map with markers and paths. MCP returns an inline image; CLI returns base64 JSON or writes --output.",
            object(
                json!({"center":string(),"zoom":integer(0,21),"size":{"type":"string","pattern":"^[1-9][0-9]{0,2}x[1-9][0-9]{0,2}$"},"maptype":{"type":"string","enum":["roadmap","satellite","terrain","hybrid"]},"markers":array(string(),1,100),"path":array(string(),1,100)}),
                &[],
            ),
        ),
        (
            "maps_batch_geocode",
            "Geocode up to 50 addresses with per-address success or error and bounded concurrency.",
            object(json!({"addresses":array(string(),1,50)}), &["addresses"]),
        ),
        (
            "maps_explore_area",
            "Resolve a location and explore categories with details for top results. Use specific landmarks for trip planning.",
            object(
                json!({"location":string(),"types":array(string(),1,10),"radius":number(0.1,50000.),"topN":integer(1,20)}),
                &["location"],
            ),
        ),
        (
            "maps_plan_route",
            "Plan a route through 2-27 stops. Optimize 2-25 intermediate stops, except transit. Exact coordinates are preserved.",
            object(plan, &["stops"]),
        ),
        (
            "maps_compare_places",
            "Search and compare places with detailed information and optional driving distance from userLocation.",
            object(
                json!({"query":string(),"userLocation":coordinate(),"limit":integer(1,20)}),
                &["query"],
            ),
        ),
        (
            "maps_search_along_route",
            "Find places along a route using its encoded polyline.",
            object(
                json!({"textQuery":string(),"origin":string(),"destination":string(),"mode":travel(),"maxResults":integer(1,20)}),
                &["textQuery", "origin", "destination"],
            ),
        ),
        (
            "maps_local_rank_tracker",
            "Track search rank across a geographic grid for 1-3 keywords; returns ARP, ATRP, SoLV and per-point errors.",
            {
                let mut schema = object(
                    json!({"keyword":string(),"keywords":array(string(),1,3),"placeId":string(),"center":coordinate(),"gridSize":integer(3,7),"gridSpacing":number(100.,10000.)}),
                    &["placeId", "center"],
                );
                schema["anyOf"] = json!([{"required":["keyword"]},{"required":["keywords"]}]);
                schema
            },
        ),
    ]
}

fn area_schema() -> Value {
    let place = json!({"type":"string","pattern":"^places/[A-Za-z0-9_-]+$"});
    let mut area = object(
        json!({
            "circle":object(json!({"latLng":coordinate(),"radius":integer(23,50000)}), &["latLng","radius"]),
            "region":object(json!({"place":place}), &["place"]),
            "customArea":object(json!({"polygon":object(json!({"coordinates":array(coordinate(),4,1000)}), &["coordinates"])}), &["polygon"])
        }),
        &[],
    );
    area["oneOf"] =
        json!([{"required":["circle"]},{"required":["region"]},{"required":["customArea"]}]);
    area
}
