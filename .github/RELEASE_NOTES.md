# v0.4.0 — 1 November 2026

The standalone Rust CLI and MCP server now provide **42 tools across 20
server-side Google API products**. The nine-language manual documents **329
parameter paths**, including defaults, nested fields, authentication and service
restrictions. CLI, MCP stdio and Streamable HTTP share the same implementations.

## New tools

Nineteen additions extend the previous 23-tool catalog:

- **Places and geography:** Autocomplete (New), radio/WiFi Geolocation, restricted
  Roads Speed Limits, Solar building insights/data layers/GeoTIFF, and separate
  Aerial View lookup and render tools.
- **Imagery:** Map Tiles sessions and 2D/3D/Street View resources, Street View
  Static, and Street View Publish photo/sequence operations.
- **Cloud resources:** Maps Datasets and Route Optimization, including native
  payloads, explicit pagination and asynchronous job status.
- **Local conditions:** Air Quality forecast/history/heatmaps, Pollen heatmaps,
  Weather history/public alerts, and experimental minute forecasts/weather maps.

Binary results preserve MIME and cache metadata. MCP displays supported images;
CLI `--output` saves original bytes. Autocomplete session tokens can also be
passed to the concluding Place Details or Address Validation call.

## Route geometry and traffic

- Directions and route planning accept independent `polylineQuality` selection:
  `OVERVIEW` or `HIGH_QUALITY`, also exposed as `--polyline-quality`. Ordinary calls
  retain Google's overview default; traffic calls default to high quality. An
  explicit quality overrides either default. Geometry is preserved without
  interpolation.
- Opt-in `trafficSegments: true` / `--traffic-segments` returns native traffic
  categories with normalized segments and availability/provenance metadata.
  Route and leg indices refer to their respective polylines. Gaps and missing
  categories remain explicit; missing traffic is never labeled normal.
- Road category, posted speed limit and provider observation time remain `null`
  because Routes does not supply them. `retrievedAt` is the local retrieval time;
  requested departure time is recorded separately. No implicit Roads lookup is
  performed.
- Traffic segments require driving and request Google's **Compute Routes
  Enterprise SKU**. Higher geometry quality alone does not request traffic
  intervals. Review Google pricing before enabling traffic calculations.

## Authentication and provider limits

- Solar and Map Tiles support API keys or OAuth. Datasets and Route Optimization
  require OAuth with appropriate resource-project and quota-project permissions.
- Street View Publish requires explicit consenting end-user OAuth credentials;
  service-account IAM does not replace consent. Media uploads are bounded to
  **8 MiB**; resumable uploads are not implemented.
- Mutations are advertised conservatively and are not automatically retried.
  Native results retain attribution and processing status.
- Speed Limits requires Google's restricted **Asset Tracking** entitlement.
  Map Tiles is licensed for visualization, not machine interpretation, image
  analysis, geodata extraction or offline use. Aerial View supports covered US
  locations and prohibits downloading, storing or caching videos.
- Weather minute forecasts and weather maps are **pre-GA**, with additional
  attribution, caching and usage restrictions. Regional coverage, EEA terms,
  billing and quotas continue to apply across services; API enablement alone
  does not grant every capability or content right.

## Verification

- **99 offline tests and one doctest passed**; 31 live tests remain opt-in and
  ignored by default.
- One live geometry comparison increased provider points from **267 to 3,110**
  while preserving the same **56,928 m** route distance. A live traffic check
  returned **16 route and 16 leg intervals**, including normal, slow and jam
  categories. These observations are examples, not coverage or accuracy promises.
- Live checks did **not** establish universal access: Speed Limits was blocked
  by entitlement, Aerial View by API enablement, and Datasets/Route Optimization
  by IAM. Publish was not exercised without consenting user credentials; public
  writes were not part of smoke testing.

## Install

```sh
cargo install mcp-gmap-rs --version 0.4.0 --locked
```

Release archives target **Linux x86_64, macOS Apple Silicon and macOS Intel**.
Verify downloads against `SHA256SUMS`. Archives include documentation, deployment
examples, the agent skill, configuration example and licenses. macOS executables
are not code-signed or notarized.
