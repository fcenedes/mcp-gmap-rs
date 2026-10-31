//! Additional read-only Google Maps services. All responses retain source metadata.

use serde_json::{Value, json};

use crate::{
    client::{Error, MapsClient, Result},
    tools::text,
};

impl MapsClient {
    pub async fn validate_address(&self, args: &Value) -> Result<Value> {
        if args["enableUspsCass"] == true && !matches!(text(args, "regionCode"), "US" | "PR") {
            return Err(Error::Input(
                "enableUspsCass requires regionCode US or PR".into(),
            ));
        }
        let mut address = json!({"addressLines":args["addressLines"]});
        for key in ["regionCode", "locality", "administrativeArea", "postalCode"] {
            if let Some(value) = args.get(key) {
                address[key] = value.clone();
            }
        }
        self.post("addressvalidation.googleapis.com", "/v1:validateAddress",
            &json!({"address":address,"enableUspsCass":args["enableUspsCass"].as_bool().unwrap_or(false)}), None).await
    }

    pub async fn pollen(&self, args: &Value) -> Result<Value> {
        self.get(
            "pollen.googleapis.com",
            "/v1/forecast:lookup",
            vec![
                ("location.latitude".into(), args["latitude"].to_string()),
                ("location.longitude".into(), args["longitude"].to_string()),
                (
                    "days".into(),
                    args["days"].as_u64().unwrap_or(1).to_string(),
                ),
                ("pageSize".into(), "5".into()),
                ("languageCode".into(), self.language.to_string()),
                (
                    "plantsDescription".into(),
                    args["plantsDescription"]
                        .as_bool()
                        .unwrap_or(false)
                        .to_string(),
                ),
            ],
        )
        .await
    }

    pub async fn area_insights(&self, args: &Value) -> Result<Value> {
        if let (Some(min), Some(max)) = (args["minRating"].as_f64(), args["maxRating"].as_f64())
            && min > max
        {
            return Err(Error::Input("minRating cannot exceed maxRating".into()));
        }
        if let Some(points) = args
            .pointer("/area/customArea/polygon/coordinates")
            .and_then(Value::as_array)
            && !["latitude", "longitude"].iter().all(|axis| {
                points.first().and_then(|point| point[*axis].as_f64())
                    == points.last().and_then(|point| point[*axis].as_f64())
            })
        {
            return Err(Error::Input(
                "polygon coordinates must form a closed ring".into(),
            ));
        }
        let mut filter = json!({"locationFilter":args["area"],"typeFilter":{"includedTypes":args["includedTypes"]}});
        if let Some(value) = args.get("excludedTypes") {
            filter["typeFilter"]["excludedTypes"] = value.clone();
        }
        for key in ["operatingStatus", "priceLevels"] {
            if let Some(value) = args.get(key) {
                filter[key] = value.clone();
            }
        }
        for key in ["minRating", "maxRating"] {
            if let Some(value) = args.get(key) {
                if filter.get("ratingFilter").is_none() {
                    filter["ratingFilter"] = json!({});
                }
                filter["ratingFilter"][key] = value.clone();
            }
        }
        let mut insights = vec!["INSIGHT_COUNT"];
        if args["includePlaces"].as_bool().unwrap_or(false) {
            insights.push("INSIGHT_PLACES");
        }
        self.post(
            "areainsights.googleapis.com",
            "/v1:computeInsights",
            &json!({"insights":insights,"filter":filter}),
            None,
        )
        .await
    }

    pub async fn roads(&self, args: &Value, snap: bool) -> Result<Value> {
        let key = if snap { "path" } else { "points" };
        let points = args[key].as_array().ok_or(Error::Response)?;
        let mut query = vec![(
            key.into(),
            points
                .iter()
                .map(|point| format!("{},{}", point["latitude"], point["longitude"]))
                .collect::<Vec<_>>()
                .join("|"),
        )];
        if snap {
            query.push((
                "interpolate".into(),
                args["interpolate"].as_bool().unwrap_or(false).to_string(),
            ));
        }
        self.get(
            "roads.googleapis.com",
            if snap {
                "/v1/snapToRoads"
            } else {
                "/v1/nearestRoads"
            },
            query,
        )
        .await
    }
}
