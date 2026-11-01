//! Bounded Google imagery retrieval and authenticated Street View publishing.
//!
//! Map Tiles content is licensed for visualization, not machine interpretation.
//! Native Google JSON preserves the attribution required by downstream viewers.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};

use crate::{
    catalog::{array, boolean, integer, number, object, string},
    client::{Error, MapsClient, Result},
    tools::Output,
};

const TILE_HOST: &str = "tile.googleapis.com";
const PUBLISH_HOST: &str = "streetviewpublish.googleapis.com";
const MAX_UPLOAD_BYTES: usize = 8 * 1024 * 1024;

fn enumeration(values: &[&str]) -> Value {
    json!({"type":"string","enum":values})
}

fn native_object() -> Value {
    json!({"type":"object","additionalProperties":true})
}

pub(crate) fn definitions() -> Vec<(&'static str, &'static str, Value)> {
    let session = object(
        json!({
            "mapType":enumeration(&["roadmap","satellite","terrain","streetview"]),
            "language":string(),"region":{"type":"string","pattern":"^[A-Z]{2}$"},
            "imageFormat":enumeration(&["jpeg","png"]),
            "scale":enumeration(&["scaleFactor1x","scaleFactor2x","scaleFactor4x"]),
            "highDpi":boolean(), "overlay":boolean(),
            "layerTypes":array(enumeration(&["layerRoadmap","layerStreetview","layerTraffic"]),1,3),
            "styles":array(native_object(),1,100)
        }),
        &["mapType", "language", "region"],
    );
    vec![
        (
            "maps_map_tiles",
            "Map Tiles sessions, 2D/3D imagery, viewport attribution and Street View tiles. Licensed for visualization only: do not use imagery for machine interpretation, analysis, extraction or offline maps. Preserve attribution and cache directives.",
            object(
                json!({
                    "operation":enumeration(&["create_session","tile_2d","viewport","root_3d","resource_3d","streetview_pano_ids","streetview_metadata","streetview_thumbnail","streetview_tile"]),
                    "sessionOptions":session,"session":string(),"zoom":integer(0,22),
                    "x":integer(0,4_194_303),"y":integer(0,4_194_303),
                    "north":number(-90.,90.),"south":number(-90.,90.),"east":number(-180.,180.),"west":number(-180.,180.),
                    "resourceUrl":string(),"panoId":string(),"latitude":number(-90.,90.),"longitude":number(-180.,180.),
                    "locations":array(object(json!({"lat":number(-90.,90.),"lng":number(-180.,180.)}), &["lat","lng"]),1,100),
                    "radius":number(0.,100_000.),"width":integer(16,600),"height":integer(16,250),
                    "yaw":number(-360.,360.),"pitch":number(-90.,90.),"fov":number(30.,120.)
                }),
                &["operation"],
            ),
        ),
        (
            "maps_streetview",
            "Get Street View Static metadata or a JPEG image by location or panorama ID. Preserve the Google attribution embedded in imagery and obey Street View usage policies.",
            object(
                json!({
                    "operation":enumeration(&["metadata","image"]),"location":string(),"panoId":string(),
                    "width":integer(1,640),"height":integer(1,640),"heading":number(0.,360.),
                    "fov":number(1.,120.),"pitch":number(-90.,90.),"radius":integer(0,100_000),
                    "source":enumeration(&["default","google","outdoor"]),"returnErrorCode":boolean(),"signature":string()
                }),
                &["operation"],
            ),
        ),
        (
            "maps_streetview_publish",
            "Manage the authenticated user's Street View photos and photo sequences, including bounded byte upload and publishing. Requires end-user OAuth consent with streetviewpublish scope; API keys and service accounts alone cannot authorize this tool. Create/update/delete operations change public imagery.",
            object(
                json!({
                    "operation":enumeration(&["list","get","start_upload","upload","create","update","delete","batch_get","batch_update","batch_delete","list_sequences","get_sequence","start_sequence_upload","create_sequence","delete_sequence"]),
                    "photoId":string(),"sequenceId":string(),"photoIds":array(string(),1,100),
                    "view":enumeration(&["BASIC","INCLUDE_DOWNLOAD_URL"]),"languageCode":string(),
                    "pageSize":integer(1,100),"pageToken":string(),"filter":string(),
                    "updateMask":string(),"photo":native_object(),"photoSequence":native_object(),
                    "updatePhotoRequests":array(native_object(),1,100),"inputType":enumeration(&["VIDEO","XDM"]),
                    "uploadUrl":string(),"dataBase64":{"type":"string","minLength":4,"maxLength":11_184_812},
                    "mimeType":enumeration(&["image/jpeg","application/octet-stream","video/mp4"])
                }),
                &["operation"],
            ),
        ),
    ]
}

pub(crate) async fn dispatch(
    client: &MapsClient,
    name: &str,
    args: &Value,
) -> Option<Result<Output>> {
    Some(match name {
        "maps_map_tiles" => map_tiles(client, args).await,
        "maps_streetview" => streetview(client, args).await,
        "maps_streetview_publish" => publish(client, args).await,
        _ => return None,
    })
}

fn input(message: impl Into<String>) -> Error {
    Error::Input(message.into())
}

fn required<'a>(args: &'a Value, key: &str) -> Result<&'a Value> {
    args.get(key)
        .filter(|v| !v.is_null())
        .ok_or_else(|| input(format!("{key} is required for this operation")))
}

fn text<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    required(args, key)?
        .as_str()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| input(format!("{key} must be a nonempty string")))
}

fn allowed(args: &Value, keys: &[&str]) -> Result<()> {
    let fields = args
        .as_object()
        .ok_or_else(|| input("arguments must be an object"))?;
    for key in fields.keys() {
        if key != "operation" && !keys.contains(&key.as_str()) {
            return Err(input(format!("{key} is not used by this operation")));
        }
    }
    Ok(())
}

fn query(args: &Value, keys: &[&str]) -> Vec<(String, String)> {
    keys.iter()
        .filter_map(|key| {
            args.get(*key).map(|v| {
                (
                    (*key).to_owned(),
                    v.as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| v.to_string()),
                )
            })
        })
        .collect()
}

fn require_keys(args: &Value, keys: &[&str]) -> Result<()> {
    for key in keys {
        required(args, key)?;
    }
    Ok(())
}

fn selector(args: &Value, location: &str, panorama: &str) -> Result<()> {
    if args.get(location).is_some() == args.get(panorama).is_some() {
        return Err(input(format!(
            "provide exactly one of {location} or {panorama}"
        )));
    }
    Ok(())
}

fn id<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    let value = text(args, key)?;
    if value.len() > 512
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
    {
        return Err(input(format!(
            "{key} must be a Google resource ID, without slashes or URL parameters"
        )));
    }
    Ok(value)
}

/// Accept only a Google-issued resource URL on its designated origin.
/// Remove API keys in returned tile URLs; the client supplies configured auth.
fn resource_url(value: &str, host: &str, prefix: &str) -> Result<(String, Vec<(String, String)>)> {
    let url = reqwest::Url::parse(value)
        .map_err(|_| input("resource URL must be an absolute Google HTTPS URL"))?;
    if url.scheme() != "https"
        || url.host_str() != Some(host)
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !url.path().starts_with(prefix)
        || url.path().contains('%')
        || value.contains('\\')
    {
        return Err(input(format!(
            "resource URL must use https://{host}{prefix} without credentials, fragments, encoded path segments or a custom port"
        )));
    }
    let pairs = url
        .query_pairs()
        .filter(|(k, _)| k != "key")
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    Ok((url.path().to_owned(), pairs))
}

async fn map_tiles(client: &MapsClient, args: &Value) -> Result<Output> {
    let op = text(args, "operation")?;
    let (path, pairs, binary) = match op {
        "create_session" => {
            allowed(args, &["sessionOptions"])?;
            let body = required(args, "sessionOptions")?;
            require_keys(body, &["mapType", "language", "region"])?;
            if body["mapType"] == "terrain"
                && !body["layerTypes"]
                    .as_array()
                    .is_some_and(|a| a.iter().any(|v| v == "layerRoadmap"))
            {
                return Err(input(
                    "terrain sessions require layerTypes containing layerRoadmap",
                ));
            }
            if body.get("styles").is_some() && body["mapType"] != "roadmap" {
                return Err(input("styles are supported only for roadmap sessions"));
            }
            return client
                .call_json(
                    "POST",
                    TILE_HOST,
                    "/v1/createSession",
                    vec![],
                    Some(body),
                    false,
                )
                .await
                .map(Output::Json);
        }
        "root_3d" => {
            allowed(args, &[])?;
            ("/v1/3dtiles/root.json".into(), vec![], false)
        }
        "resource_3d" => {
            allowed(args, &["resourceUrl"])?;
            let (path, pairs) =
                resource_url(text(args, "resourceUrl")?, TILE_HOST, "/v1/3dtiles/")?;
            let binary = !path.ends_with(".json");
            (path, pairs, binary)
        }
        "tile_2d" | "streetview_tile" => {
            allowed(
                args,
                if op == "tile_2d" {
                    &["session", "zoom", "x", "y"]
                } else {
                    &["session", "zoom", "x", "y", "panoId"]
                },
            )?;
            require_keys(args, &["session", "zoom", "x", "y"])?;
            let z = args["zoom"]
                .as_u64()
                .ok_or_else(|| input("zoom must be an integer"))?;
            let x = args["x"]
                .as_u64()
                .ok_or_else(|| input("x must be an integer"))?;
            let y = args["y"]
                .as_u64()
                .ok_or_else(|| input("y must be an integer"))?;
            let mut q = query(args, &["session"]);
            if op == "tile_2d" {
                if z > 22 || x >= (1u64 << z) || y >= (1u64 << z) {
                    return Err(input(
                        "2D tile x and y must be below 2^zoom; zoom must be 0..22",
                    ));
                }
                (format!("/v1/2dtiles/{z}/{x}/{y}"), q, true)
            } else {
                if z > 5 {
                    return Err(input("Street View tile zoom must be 0..5"));
                }
                q.push(("panoId".into(), text(args, "panoId")?.into()));
                (format!("/v1/streetview/tiles/{z}/{x}/{y}"), q, true)
            }
        }
        "viewport" => {
            let keys = ["session", "zoom", "north", "south", "east", "west"];
            allowed(args, &keys)?;
            require_keys(args, &keys)?;
            if args["north"].as_f64() <= args["south"].as_f64() {
                return Err(input(
                    "north must be greater than south; east/west may cross the antimeridian",
                ));
            }
            ("/tile/v1/viewport".into(), query(args, &keys), false)
        }
        "streetview_pano_ids" => {
            allowed(args, &["session", "locations", "radius"])?;
            text(args, "session")?;
            required(args, "locations")?;
            let mut body = json!({"locations":args["locations"]});
            if let Some(v) = args.get("radius") {
                body["radius"] = v.clone();
            }
            return client
                .call_json(
                    "POST",
                    TILE_HOST,
                    "/v1/streetview/panoIds",
                    query(args, &["session"]),
                    Some(&body),
                    true,
                )
                .await
                .map(Output::Json);
        }
        "streetview_metadata" => {
            allowed(
                args,
                &["session", "panoId", "latitude", "longitude", "radius"],
            )?;
            text(args, "session")?;
            let mut q = query(args, &["session", "panoId", "radius"]);
            if args.get("panoId").is_some() {
                if args.get("latitude").is_some()
                    || args.get("longitude").is_some()
                    || args.get("radius").is_some()
                {
                    return Err(input(
                        "panoId cannot be combined with coordinates or radius",
                    ));
                }
            } else {
                require_keys(args, &["latitude", "longitude"])?;
                q.extend([
                    ("lat".into(), args["latitude"].to_string()),
                    ("lng".into(), args["longitude"].to_string()),
                ]);
            }
            ("/v1/streetview/metadata".into(), q, false)
        }
        "streetview_thumbnail" => {
            allowed(
                args,
                &[
                    "session", "panoId", "width", "height", "yaw", "pitch", "fov",
                ],
            )?;
            require_keys(args, &["session", "panoId"])?;
            (
                "/v1/streetview/thumbnail".into(),
                query(
                    args,
                    &[
                        "session", "panoId", "width", "height", "yaw", "pitch", "fov",
                    ],
                ),
                true,
            )
        }
        _ => return Err(input("unknown Map Tiles operation")),
    };
    if binary {
        client
            .download(TILE_HOST, &path, pairs)
            .await
            .map(Output::Binary)
    } else {
        client
            .call_json("GET", TILE_HOST, &path, pairs, None, true)
            .await
            .map(Output::Json)
    }
}

async fn streetview(client: &MapsClient, args: &Value) -> Result<Output> {
    let op = text(args, "operation")?;
    selector(args, "location", "panoId")?;
    let mut q = query(args, &["location", "radius", "source", "signature"]);
    if let Some(v) = args.get("panoId") {
        q.push((
            "pano".into(),
            v.as_str()
                .ok_or_else(|| input("panoId must be a string"))?
                .into(),
        ));
    }
    match op {
        "metadata" => {
            allowed(
                args,
                &["location", "panoId", "radius", "source", "signature"],
            )?;
            client
                .call_json(
                    "GET",
                    "maps.googleapis.com",
                    "/maps/api/streetview/metadata",
                    q,
                    None,
                    true,
                )
                .await
                .map(Output::Json)
        }
        "image" => {
            allowed(
                args,
                &[
                    "location",
                    "panoId",
                    "width",
                    "height",
                    "heading",
                    "fov",
                    "pitch",
                    "radius",
                    "source",
                    "returnErrorCode",
                    "signature",
                ],
            )?;
            q.push((
                "size".into(),
                format!(
                    "{}x{}",
                    args["width"].as_u64().unwrap_or(640),
                    args["height"].as_u64().unwrap_or(400)
                ),
            ));
            q.extend(query(args, &["heading", "fov", "pitch"]));
            q.push((
                "return_error_code".into(),
                args["returnErrorCode"]
                    .as_bool()
                    .unwrap_or(true)
                    .to_string(),
            ));
            client
                .download("maps.googleapis.com", "/maps/api/streetview", q)
                .await
                .map(Output::Binary)
        }
        _ => Err(input("unknown Street View Static operation")),
    }
}

async fn publish(client: &MapsClient, args: &Value) -> Result<Output> {
    let op = text(args, "operation")?;
    let mut pairs = vec![];
    let mut body = None;
    let (method, path, read) = match op {
        "list" | "list_sequences" => {
            let keys = if op == "list" {
                vec!["view", "languageCode", "pageSize", "pageToken", "filter"]
            } else {
                vec!["pageSize", "pageToken", "filter"]
            };
            allowed(args, &keys)?;
            pairs = query(args, &keys);
            (
                "GET",
                if op == "list" {
                    "/v1/photos".into()
                } else {
                    "/v1/photoSequences".into()
                },
                true,
            )
        }
        "get" | "get_sequence" => {
            let key = if op == "get" { "photoId" } else { "sequenceId" };
            let keys = if op == "get" {
                vec![key, "view", "languageCode"]
            } else {
                vec![key, "view", "filter"]
            };
            allowed(args, &keys)?;
            let resource = if op == "get" {
                "photo"
            } else {
                "photoSequence"
            };
            pairs = query(
                args,
                if op == "get" {
                    &["view", "languageCode"]
                } else {
                    &["view", "filter"]
                },
            );
            ("GET", format!("/v1/{resource}/{}", id(args, key)?), true)
        }
        "start_upload" | "start_sequence_upload" => {
            allowed(args, &[])?;
            (
                "POST",
                if op == "start_upload" {
                    "/v1/photo:startUpload".into()
                } else {
                    "/v1/photoSequence:startUpload".into()
                },
                false,
            )
        }
        "upload" => {
            allowed(args, &["uploadUrl", "dataBase64", "mimeType"])?;
            let (path, pairs) =
                resource_url(text(args, "uploadUrl")?, PUBLISH_HOST, "/media/user/")?;
            let components: Vec<_> = path.split('/').collect();
            // UploadRef: /media/user/{account_id}/photo/{upload_reference}.
            if components.len() != 6
                || !matches!(components[4], "photo" | "photoSequence")
                || components.iter().skip(1).any(|part| part.is_empty())
            {
                return Err(input(
                    "uploadUrl must be a Street View Publish UploadRef media URL",
                ));
            }
            let encoded = text(args, "dataBase64")?;
            if encoded.len() > MAX_UPLOAD_BYTES.div_ceil(3) * 4 {
                return Err(input(
                    "upload exceeds the implementation limit of 8 MiB decoded",
                ));
            }
            let bytes = STANDARD
                .decode(encoded)
                .map_err(|_| input("dataBase64 must be standard padded base64"))?;
            if bytes.is_empty() || bytes.len() > MAX_UPLOAD_BYTES {
                return Err(input(
                    "upload must contain 1 byte to 8 MiB decoded (implementation limit)",
                ));
            }
            return client
                .upload(
                    PUBLISH_HOST,
                    &path,
                    pairs,
                    bytes,
                    args["mimeType"].as_str().unwrap_or("image/jpeg"),
                )
                .await
                .map(Output::Json);
        }
        "create" | "create_sequence" => {
            let key = if op == "create" {
                "photo"
            } else {
                "photoSequence"
            };
            allowed(
                args,
                if op == "create" {
                    &["photo"]
                } else {
                    &["photoSequence", "inputType"]
                },
            )?;
            let payload = required(args, key)?;
            let upload = text(required(payload, "uploadReference")?, "uploadUrl")?;
            resource_url(upload, PUBLISH_HOST, "/media/user/")?;
            if op == "create_sequence" {
                pairs.push(("inputType".into(), text(args, "inputType")?.into()));
            }
            body = Some(payload);
            (
                "POST",
                if op == "create" {
                    "/v1/photo".into()
                } else {
                    "/v1/photoSequence".into()
                },
                false,
            )
        }
        "update" => {
            allowed(args, &["photoId", "photo", "updateMask"])?;
            body = Some(required(args, "photo")?);
            pairs.push(("updateMask".into(), text(args, "updateMask")?.into()));
            if let Some(other) = args["photo"].pointer("/photoId/id").and_then(Value::as_str)
                && other != id(args, "photoId")?
            {
                return Err(input("photo.photoId.id must match photoId"));
            }
            ("PUT", format!("/v1/photo/{}", id(args, "photoId")?), false)
        }
        "delete" | "delete_sequence" => {
            let key = if op == "delete" {
                "photoId"
            } else {
                "sequenceId"
            };
            allowed(args, &[key])?;
            let resource = if op == "delete" {
                "photo"
            } else {
                "photoSequence"
            };
            (
                "DELETE",
                format!("/v1/{resource}/{}", id(args, key)?),
                false,
            )
        }
        "batch_get" => {
            allowed(args, &["photoIds", "view", "languageCode"])?;
            pairs = query(args, &["view", "languageCode"]);
            let ids = required(args, "photoIds")?
                .as_array()
                .ok_or_else(|| input("photoIds must be an array"))?;
            for value in ids {
                pairs.push((
                    "photoIds".into(),
                    value
                        .as_str()
                        .ok_or_else(|| input("photoIds values must be strings"))?
                        .into(),
                ));
            }
            ("GET", "/v1/photos:batchGet".into(), true)
        }
        "batch_update" | "batch_delete" => {
            let key = if op == "batch_update" {
                "updatePhotoRequests"
            } else {
                "photoIds"
            };
            allowed(args, &[key])?;
            let payload = json!({key:required(args,key)?});
            if op == "batch_update" {
                for request in payload[key]
                    .as_array()
                    .ok_or_else(|| input("updatePhotoRequests must be an array"))?
                {
                    text(request, "updateMask")?;
                    id(required(required(request, "photo")?, "photoId")?, "id")?;
                }
            }
            return client
                .call_json(
                    "POST",
                    PUBLISH_HOST,
                    if op == "batch_update" {
                        "/v1/photos:batchUpdate"
                    } else {
                        "/v1/photos:batchDelete"
                    },
                    vec![],
                    Some(&payload),
                    false,
                )
                .await
                .map(Output::Json);
        }
        _ => return Err(input("unknown Street View Publish operation")),
    };
    client
        .call_json(method, PUBLISH_HOST, &path, pairs, body, read)
        .await
        .map(Output::Json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{auth::OAuth, catalog::Catalog, tools::EngineState};
    use axum::{
        Router,
        body::Bytes,
        extract::State,
        http::{HeaderMap, Method, Uri},
        response::{IntoResponse, Response},
        routing::any,
    };
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    #[derive(Clone, Debug)]
    struct Request {
        method: Method,
        path: String,
        query: String,
        headers: HeaderMap,
        bytes: Vec<u8>,
    }

    struct Fixture {
        engine: EngineState,
        records: Arc<Mutex<Vec<Request>>>,
        task: tokio::task::JoinHandle<()>,
        _credentials: Option<tempfile::TempDir>,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    impl Fixture {
        async fn new(oauth: bool) -> Self {
            let records = Arc::new(Mutex::new(vec![]));
            let app = Router::new()
                .fallback(any(google))
                .with_state(records.clone());
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let task = tokio::spawn(async move {
                axum::serve(listener, app).await.unwrap();
            });
            let mut client = MapsClient::new(
                Some("fixture-secret-key".into()),
                Duration::from_secs(2),
                "en".into(),
            )
            .unwrap();
            client.test_base = Some(format!("http://{address}"));
            client.policy.max_retries = 2;
            let credentials = if oauth {
                let directory = tempfile::tempdir().unwrap();
                let path = directory.path().join("credentials.json");
                std::fs::write(&path,serde_json::to_vec(&json!({"type":"authorized_user","client_id":"fixture-client","client_secret":"fixture-client-secret","refresh_token":"fixture-refresh-token","token_uri":format!("http://{address}/oauth/token")})).unwrap()).unwrap();
                client.oauth = Some(OAuth::new(
                    Some(path),
                    Some("fixture-project".into()),
                    Duration::from_secs(2),
                ));
                Some(directory)
            } else {
                None
            };
            Self {
                engine: EngineState {
                    client,
                    catalog: Catalog::new(None).unwrap(),
                },
                records,
                task,
                _credentials: credentials,
            }
        }
        fn requests(&self) -> Vec<Request> {
            self.records
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.path != "/oauth/token")
                .cloned()
                .collect()
        }
        async fn call(&self, name: &str, args: Value) -> Value {
            self.engine.call(name, args).await.unwrap().into_json()
        }
    }

    async fn google(
        State(records): State<Arc<Mutex<Vec<Request>>>>,
        method: Method,
        uri: Uri,
        headers: HeaderMap,
        bytes: Bytes,
    ) -> Response {
        records.lock().unwrap().push(Request {
            method: method.clone(),
            path: uri.path().into(),
            query: uri.query().unwrap_or_default().into(),
            headers,
            bytes: bytes.to_vec(),
        });
        if uri.path() == "/oauth/token" {
            return axum::Json(json!({"access_token":"fixture-oauth-token","expires_in":3600,"token_type":"Bearer"})).into_response();
        }
        if uri.path().contains("/fail") {
            return (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                axum::Json(json!({"error":{"message":"fixture failure"}})),
            )
                .into_response();
        }
        if uri.path() == "/maps.googleapis.com/maps/api/streetview"
            || uri.path().contains("/2dtiles/")
            || uri.path().contains("/streetview/tiles/")
            || uri.path().ends_with("thumbnail")
            || uri.path().ends_with(".glb")
        {
            return (
                [
                    (
                        "content-type",
                        if uri.path().ends_with(".glb") {
                            "model/gltf-binary"
                        } else {
                            "image/jpeg"
                        },
                    ),
                    ("cache-control", "private, max-age=30"),
                    ("etag", "fixture-etag"),
                ],
                vec![255u8, 216, 255, 217],
            )
                .into_response();
        }
        if uri.path().contains("/media/user/") {
            return axum::http::StatusCode::OK.into_response();
        }
        axum::Json(json!({"method":method.as_str(),"copyright":"Fixture Google attribution","reportProblemLink":"https://example.test/report","body":serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null),"uploadUrl":"https://streetviewpublish.googleapis.com/media/user/123/photo/ref","root":{"content":{"uri":"https://tile.googleapis.com/v1/3dtiles/child.json?session=s&key=fixture-secret-key"}}})).into_response()
    }

    #[tokio::test]
    async fn map_tiles_send_native_requests_and_preserve_attribution() {
        let f = Fixture::new(false).await;
        let session = json!({"mapType":"terrain","language":"en-US","region":"US","layerTypes":["layerRoadmap"]});
        let response = f
            .call(
                "maps_map_tiles",
                json!({"operation":"create_session","sessionOptions":session}),
            )
            .await;
        assert_eq!(response["body"], session);
        assert_eq!(response["copyright"], "Fixture Google attribution");
        f.call("maps_map_tiles",json!({"operation":"viewport","session":"s","zoom":8,"north":50,"south":48,"east":-170,"west":170})).await;
        f.call("maps_map_tiles",json!({"operation":"streetview_pano_ids","session":"s","locations":[{"lat":48.8,"lng":2.3}],"radius":25})).await;
        f.call("maps_map_tiles",json!({"operation":"streetview_metadata","session":"s","latitude":48.8,"longitude":2.3})).await;
        f.call(
            "maps_map_tiles",
            json!({"operation":"streetview_metadata","session":"s","panoId":"p"}),
        )
        .await;
        let root = f
            .call("maps_map_tiles", json!({"operation":"root_3d"}))
            .await;
        assert!(!root.to_string().contains("fixture-secret-key"));
        f.call("maps_map_tiles",json!({"operation":"resource_3d","resourceUrl":"https://tile.googleapis.com/v1/3dtiles/child.json?session=s&key=discard-me"})).await;
        let records = f.requests();
        assert_eq!(records.len(), 7);
        assert_eq!(records[0].method, Method::POST);
        assert_eq!(records[0].path, "/tile.googleapis.com/v1/createSession");
        assert_eq!(records[1].path, "/tile.googleapis.com/tile/v1/viewport");
        assert!(records[1].query.contains("east=-170"));
        assert_eq!(records[2].method, Method::POST);
        assert_eq!(
            serde_json::from_slice::<Value>(&records[2].bytes).unwrap(),
            json!({"locations":[{"lat":48.8,"lng":2.3}],"radius":25})
        );
        assert!(records[3].query.contains("lat=48.8"));
        assert!(records[4].query.contains("panoId=p"));
        assert!(!records[6].query.contains("discard-me"));
    }

    #[tokio::test]
    async fn images_and_3d_resources_are_binary_and_fixed_origin() {
        let f = Fixture::new(false).await;
        for args in [
            json!({"operation":"tile_2d","session":"s","zoom":0,"x":0,"y":0}),
            json!({"operation":"streetview_tile","session":"s","panoId":"p","zoom":5,"x":2,"y":1}),
            json!({"operation":"streetview_thumbnail","session":"s","panoId":"p","width":200,"height":100,"yaw":90}),
            json!({"operation":"resource_3d","resourceUrl":"https://tile.googleapis.com/v1/3dtiles/tile.glb?session=s"}),
        ] {
            let output = f.engine.call("maps_map_tiles", args).await.unwrap();
            let Output::Binary(binary) = output else {
                panic!("expected binary response")
            };
            assert_eq!(binary.data, vec![255, 216, 255, 217]);
            assert_eq!(binary.headers["cache-control"], "private, max-age=30");
            assert_eq!(binary.headers["etag"], "fixture-etag");
        }
        let output = f
            .engine
            .call(
                "maps_streetview",
                json!({"operation":"image","location":"Paris","heading":90}),
            )
            .await
            .unwrap();
        let Output::Binary(binary) = output else {
            panic!("expected Street View JPEG")
        };
        assert_eq!(binary.mime_type, "image/jpeg");
        f.call(
            "maps_streetview",
            json!({"operation":"metadata","panoId":"p"}),
        )
        .await;
        let records = f.requests();
        assert!(records[4].query.contains("size=640x400"));
        assert!(records[4].query.contains("return_error_code=true"));
        assert!(records[5].query.contains("pano=p"));
    }

    #[tokio::test]
    async fn publish_covers_photo_and_sequence_lifecycle_with_user_oauth() {
        let f = Fixture::new(true).await;
        let upload = "https://streetviewpublish.googleapis.com/media/user/123/photo/ref";
        let operations = [
            json!({"operation":"list","pageSize":10,"view":"BASIC"}),
            json!({"operation":"get","photoId":"p1","view":"INCLUDE_DOWNLOAD_URL"}),
            json!({"operation":"start_upload"}),
            json!({"operation":"upload","uploadUrl":upload,"dataBase64":"/9j/2Q==","mimeType":"image/jpeg"}),
            json!({"operation":"create","photo":{"uploadReference":{"uploadUrl":upload},"pose":{"latLngPair":{"latitude":48.8,"longitude":2.3}}}}),
            json!({"operation":"update","photoId":"p1","updateMask":"pose.heading","photo":{"pose":{"heading":90}}}),
            json!({"operation":"delete","photoId":"p1"}),
            json!({"operation":"batch_get","photoIds":["p1","p2"],"languageCode":"en"}),
            json!({"operation":"batch_update","updatePhotoRequests":[{"photo":{"photoId":{"id":"p1"},"pose":{"heading":80}},"updateMask":"pose.heading"}]}),
            json!({"operation":"batch_delete","photoIds":["p1","p2"]}),
            json!({"operation":"list_sequences","pageSize":5}),
            json!({"operation":"get_sequence","sequenceId":"s1"}),
            json!({"operation":"start_sequence_upload"}),
            json!({"operation":"create_sequence","inputType":"VIDEO","photoSequence":{"uploadReference":{"uploadUrl":upload}}}),
            json!({"operation":"delete_sequence","sequenceId":"s1"}),
        ];
        for args in operations {
            f.call("maps_streetview_publish", args).await;
        }
        let records = f.requests();
        assert_eq!(records.len(), 15);
        for record in &records {
            assert_eq!(
                record.headers["authorization"],
                "Bearer fixture-oauth-token"
            );
            assert!(!record.query.contains("key="));
        }
        assert_eq!(records[3].method, Method::POST);
        assert_eq!(
            records[3].path,
            "/streetviewpublish.googleapis.com/media/user/123/photo/ref"
        );
        assert_eq!(records[3].bytes, vec![255, 216, 255, 217]);
        assert_eq!(records[3].headers["content-type"], "image/jpeg");
        assert_eq!(records[5].method, Method::PUT);
        assert!(records[5].query.contains("updateMask=pose.heading"));
        assert_eq!(records[6].method, Method::DELETE);
        assert!(records[7].query.contains("photoIds=p1&photoIds=p2"));
        assert_eq!(
            records[8].path,
            "/streetviewpublish.googleapis.com/v1/photos:batchUpdate"
        );
        assert!(records[13].query.contains("inputType=VIDEO"));
    }

    #[tokio::test]
    async fn invalid_operations_selectors_resources_and_uploads_never_reach_network() {
        let f = Fixture::new(true).await;
        for (name, args) in [
            (
                "maps_map_tiles",
                json!({"operation":"create_session","sessionOptions":{"mapType":"terrain","language":"en-US","region":"US"}}),
            ),
            (
                "maps_map_tiles",
                json!({"operation":"tile_2d","session":"s","zoom":1,"x":2,"y":0}),
            ),
            (
                "maps_map_tiles",
                json!({"operation":"streetview_tile","session":"s","panoId":"p","zoom":6,"x":0,"y":0}),
            ),
            (
                "maps_map_tiles",
                json!({"operation":"streetview_metadata","session":"s","panoId":"p","latitude":0,"longitude":0}),
            ),
            (
                "maps_map_tiles",
                json!({"operation":"streetview_pano_ids","session":"s","locations":[]}),
            ),
            (
                "maps_map_tiles",
                json!({"operation":"viewport","session":"s","zoom":1,"north":0,"south":1,"east":2,"west":0}),
            ),
            (
                "maps_map_tiles",
                json!({"operation":"resource_3d","resourceUrl":"https://evil.test/v1/3dtiles/root.json"}),
            ),
            (
                "maps_map_tiles",
                json!({"operation":"resource_3d","resourceUrl":"https://tile.googleapis.com/v1/3dtiles/%2fsecret"}),
            ),
            (
                "maps_map_tiles",
                json!({"operation":"root_3d","session":"unused"}),
            ),
            (
                "maps_streetview",
                json!({"operation":"image","location":"Paris","panoId":"p"}),
            ),
            ("maps_streetview", json!({"operation":"image"})),
            (
                "maps_streetview",
                json!({"operation":"image","location":"Paris","width":641}),
            ),
            (
                "maps_streetview_publish",
                json!({"operation":"get","photoId":"../evil"}),
            ),
            (
                "maps_streetview_publish",
                json!({"operation":"update","photoId":"p1","photo":{}}),
            ),
            (
                "maps_streetview_publish",
                json!({"operation":"create","photo":{}}),
            ),
            (
                "maps_streetview_publish",
                json!({"operation":"upload","uploadUrl":"https://evil.test/media/user/123/photo/ref","dataBase64":"/9j/2Q=="}),
            ),
            (
                "maps_streetview_publish",
                json!({"operation":"upload","uploadUrl":"https://streetviewpublish.googleapis.com/media/user/123/photo/ref","dataBase64":"@@@@"}),
            ),
            (
                "maps_streetview_publish",
                json!({"operation":"upload","uploadUrl":"https://streetviewpublish.googleapis.com/media/user/123/photo/ref","dataBase64":STANDARD.encode(vec![0u8;MAX_UPLOAD_BYTES+1])}),
            ),
        ] {
            assert!(f.engine.call(name, args).await.is_err(), "{name}");
        }
        assert!(f.requests().is_empty());
    }

    #[tokio::test]
    async fn publish_mutations_are_not_retried() {
        let f = Fixture::new(true).await;
        let result = f
            .engine
            .call(
                "maps_streetview_publish",
                json!({"operation":"delete","photoId":"fail"}),
            )
            .await;
        assert!(result.is_err());
        assert_eq!(f.requests().len(), 1);
    }

    #[test]
    fn google_resource_urls_reject_credential_confusion_and_custom_origins() {
        for value in [
            "http://tile.googleapis.com/v1/3dtiles/root.json",
            "https://tile.googleapis.com:444/v1/3dtiles/root.json",
            "https://user@tile.googleapis.com/v1/3dtiles/root.json",
            "https://tile.googleapis.com.evil.test/v1/3dtiles/root.json",
            "https://tile.googleapis.com/v1/3dtiles/../other",
            "https://tile.googleapis.com/v1/3dtiles/root.json#fragment",
        ] {
            assert!(
                resource_url(value, TILE_HOST, "/v1/3dtiles/").is_err(),
                "{value}"
            );
        }
    }
}
