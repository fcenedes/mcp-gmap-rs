# Google Maps API coverage and access limits

This adapter exposes **42 tools across 20 server-side Google products** through
the same Rust CLI, MCP stdio and Streamable HTTP executable. Coverage means the
operations below have implementations and local contract fixtures. It does not
mean every Google field, geography, account entitlement or public mutation has
been tested live. Enabling a service in Cloud Console does not establish its
license, billing entitlement or user consent.

## Server-side products

| Google product | Implemented tool / operations | Remaining boundary or access requirement |
| --- | --- | --- |
| Places API (New) | Search text/nearby, details, photo URL and autocomplete; composite discovery/ranking tools | Native API has additional methods/fields beyond these wrappers. Session tokens are caller-managed and can terminate via details/address validation. |
| Routes API | Directions, route matrix, route planning and route comparison/search composites | Existing wrappers expose selected routing options, not the whole native Routes schema. Route Optimization is a separate API below. |
| Geocoding API v3 | Geocode, reverse geocode, batch geocode | v4 endpoints are not implemented; v3 calls require an API key. |
| Maps Elevation API | Point/batched elevation lookup | No dedicated encoded-polyline sampling wrapper. |
| Time Zone API | Time zone and offsets for coordinates/time | API key required. |
| Maps Static API | Static map image with marker/path options | Premium-sized imagery, a renderer and URL-signing-key management are outside the adapter. |
| Address Validation API | Validate postal address, optional USPS CASS and autocomplete session token | No feedback operation; regional/CASS eligibility and terms apply. |
| Air Quality API | Current conditions; explicit-page forecast/history; binary heatmap tiles | Forecast up to 96 hours, history up to 30 days; country/index coverage varies. |
| Weather API | Current, daily/hourly forecasts, 24-hour history, public alerts; experimental minute forecasts/map tiles | Minute/map products are pre-GA; US/EU map coverage and additional content restrictions apply. |
| Pollen API | Daily forecast and binary tree/grass/weed heatmap tiles | Country/season coverage varies; API-key authentication in this implementation. |
| Places Aggregate API | Area counts and supported place-ID insights | Count/ID and geometry limits remain Google-enforced; no arbitrary Places database export. |
| Roads API | Snap to roads, nearest roads and speed limits | **Speed limits require Asset Tracking entitlement**; no real-time/vehicle-specific accuracy guarantee. |
| Geolocation API | Native cell-tower/WiFi positioning request | IP fallback is the executable's egress location, not the agent user's location. API key required. |
| Solar API | Building insights, data layers and authenticated GeoTIFF download | Location/imagery coverage varies; expanded coverage is pre-GA. Large rasters remain bounded by the response-size limit. |
| Aerial View API | Video metadata, playback URL lookup and explicit render request | Supported US addresses; rendering is asynchronous. Video downloading/caching is not implemented or permitted by the cited policy. |
| Map Tiles API | Sessions, 2D tiles, viewport attribution, 3D root/resources, Street View panorama IDs/metadata/thumbnails/tiles | Adapter returns real media/metadata; no renderer, tile cache, offline map, bulk extraction or imagery-analysis operation. |
| Street View Static API | Panorama metadata and image with optional caller signature | API key; signature generation/premium imagery are not implemented. Panorama availability/IDs can change. |
| Maps Platform Datasets API | List, get/status, create, patch, delete, GCS import, download and paginated errors | OAuth plus operation IAM; **no public v1 listVersions method**. Get latest or @active; local multipart upload is not exposed. |
| Route Optimization API | Full native ShipmentModel optimize/validate; GCS batch; explicit experimental long-running/URI jobs; get operation | OAuth/IAM and storage access; **Google currently does not support cancel/list/delete operations**. Transport timeout does not cancel a job. |
| Street View Publish API | Photo and sequence upload initiation, bounded upload, create/get/list/update/delete and documented photo batches | **Consenting end-user OAuth**, not service-account IAM. Uploads ≤8 MiB; no resumable media transfer. Public writes are excluded from live smoke tests. |

All native JSON fields returned by the newly added service tools are preserved.
Complex native request bodies (for example ShipmentModel, Geolocation radio
observations and Street View GPS/pose/IMU metadata) are forwarded rather than
flattened into an incomplete model. The tool schema validates the wrapper and
resource paths; Google validates those native payloads. Page tokens are explicit:
one call returns one page, without hidden next-page or long-running polling loops.

## Enabled legacy APIs and client SDKs

| Enabled Google item | Treatment here |
| --- | --- |
| Directions API (legacy) | The directions CLI/MCP tool uses Routes API. No separate legacy Directions endpoint. |
| Distance Matrix API (legacy) | The distance-matrix tool uses Routes computeRouteMatrix. No legacy Distance Matrix endpoint. |
| Places API (legacy) | Places tools use Places API (New). No legacy endpoint adapter. |
| Maps Embed API | Client iframe integration; not a server-side REST tool. |
| Maps JavaScript API / Places UI Kit | Browser rendering/UI SDKs; integrate them in the consuming application. |
| Maps SDK for Android / iOS | Native application SDKs; not executable server adapters. |
| Maps 3D SDK for Android / iOS | Native rendering SDKs; Map Tiles 3D resources do not substitute for those SDKs. |
| Navigation SDK | Native turn-by-turn application SDK; not a standalone server-side tool. |

## Licensing, billing and geographical limits

- **Road speed limits:** Google's [Speed Limits documentation](https://developers.google.com/maps/documentation/roads/speed-limits)
  limits access to Asset Tracking customers (including applicable transitioned
  Premium projects). One request can bill multiple returned speed-limit entries.
  An ordinary enabled Roads API key is insufficient. The API can return missing,
  estimated or outdated limits; posted signs remain authoritative.
- **Weather experimental products:** [minute forecasts](https://developers.google.com/maps/documentation/weather/minute-forecast)
  and [weather maps](https://developers.google.com/maps/documentation/weather/weather-map)
  are pre-GA with limited support and possible incompatible changes. Their
  additional terms require attribution, limit cached content to one hour and
  prohibit recreating a service whose primary purpose is providing weather
  information. US/EU tiles only cover the documented regions.
- **Map Tiles:** [policy](https://developers.google.com/maps/documentation/tile/policies)
  restricts content to map visualization and disallows unauthorized prefetching,
  storage/indexing and extraction. Return/retain applicable Cache-Control, ETag,
  logo, provider copyright and report-problem attribution. Exposing a tile as an
  MCP image does not grant a downstream agent permission to interpret/extract
  information from it.
- **Aerial View:** [policy](https://developers.google.com/maps/documentation/aerial-view/policies)
  requires attribution and prohibits storing/downloading/caching videos. The
  adapter returns fresh playback URLs and does not fetch video bytes. US/3D
  address coverage and rendering quotas apply.
- **Street View Static/Publish:** [Static policies](https://developers.google.com/maps/documentation/streetview/policies)
  govern imagery use and attribution; [Publish authorization](https://developers.google.com/streetview/publish/authorizing)
  requires user consent, ownership and accepted 360-degree content. Service
  accounts cannot replace that consent. Pixel updates are unsupported by Google;
  metadata updates and explicit delete/create operations are distinct.
- **Solar and environmental data:** [Solar policies](https://developers.google.com/maps/documentation/solar/policies),
  [Air Quality policies](https://developers.google.com/maps/documentation/air-quality/policies),
  [Pollen policies](https://developers.google.com/maps/documentation/pollen/policies)
  and [Weather policies](https://developers.google.com/maps/documentation/weather/policies)
  govern display, attribution and storage. Solar data shown on a map must use a
  Google map and the required source attribution. Coverage is not universal.
- **Cloud resources:** [Datasets](https://developers.google.com/maps/documentation/datasets/overview)
  currently has no API usage charge, but display/storage services can have their
  own charges. [Route Optimization billing](https://developers.google.com/maps/documentation/route-optimization/usage-and-billing)
  is shipment-based with Single Vehicle Routing/Fleet Routing SKUs. API enablement
  does not grant resource IAM or GCS permissions.
- **Regional terms:** customers with EEA billing addresses can have different
  [service terms and functionality](https://developers.google.com/maps/comms/eea/faq).
  Inspect the relevant product's current agreement for the intended deployment.

The Rust package's MIT license covers this code, not Google content or API
access. The tool reference links the applicable primary sources and documents
which operations mutate public/cloud resources. The live doctor checks only
read-only probe operations; fixture verification and live verification are
reported separately.

## Primary API contract sources

Reviewed 2026-11-01. Current official discovery schemas were consulted for
Air Quality, Pollen and Weather. Google's Weather REST index lags its discovery
schema for minute/map products; the experimental guides explicitly document the
implemented endpoints.

- [Maps Datasets REST](https://developers.google.com/maps/documentation/datasets/reference/rest),
  [GCS import](https://developers.google.com/maps/documentation/datasets/create),
  [current v1 proto](https://github.com/googleapis/googleapis/blob/master/google/maps/mapsplatformdatasets/v1/maps_platform_datasets.proto).
- [Route Optimization REST](https://developers.google.com/maps/documentation/route-optimization/reference/rest),
  [unsupported operation methods](https://developers.google.com/maps/documentation/route-optimization/reference/rest/v1/operations),
  [current proto and experimental annotations](https://github.com/googleapis/googleapis/blob/master/google/maps/routeoptimization/v1/route_optimization_service.proto).
- [Air Quality v1 discovery](https://airquality.googleapis.com/$discovery/rest?version=v1),
  [Pollen v1 discovery](https://pollen.googleapis.com/$discovery/rest?version=v1),
  [Weather v1 discovery](https://weather.googleapis.com/$discovery/rest?version=v1).
