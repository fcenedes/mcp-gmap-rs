# Google Maps CLI and MCP server

`mcp-gmap-rs` provides 42 Google Maps tools through one Rust executable:
a JSON CLI, an MCP stdio server, and an MCP Streamable HTTP server. All three
interfaces share argument validation and Google API execution.

This crate is primarily distributed as an executable. Its public Rust entry point
is [`run`], which drives the same command-line application inside a Tokio runtime.
The underlying Google clients are implementation details, not a public Rust SDK.

## Install

Version 0.4.0 includes the complete 42-tool catalog below. Install from crates.io:

```sh
cargo install mcp-gmap-rs --version 0.4.0 --locked
```

For an existing source checkout, use `cargo install --path . --locked`.
Rust 1.88 or newer is required to build from source. Prebuilt Linux and macOS
executables are available from the repository's GitHub Releases page.

## Configuration

Configure Google API credentials in the environment or an optional `.env` in the
working directory before making Google requests. API-key mode is the default;
OAuth is opt-in. Configuration precedence is defaults →
`.env` → environment variables → CLI flags. Discovery and schema
inspection work without a key. API keys are process configuration and are not
accepted as tool arguments.

Use `--env-file /etc/mcp-gmap/runtime.env` or `MCP_ENV_FILE` for another path;
the flag wins. `--no-env-file` disables file loading. Only the working directory's
default `.env` may be absent; explicit paths must exist, and malformed/unreadable
files fail startup without printing their contents. Parent directories are not
searched. Files are parsed without changing the process environment.

| Variable | Default | Purpose |
| --- | --- | --- |
| `GOOGLE_MAPS_API_KEY` | unset | Credential for Google API requests |
| `GOOGLE_MAPS_AUTH_MODE` | `api-key` | `api-key` or `oauth`; `--auth-mode` overrides it |
| `GOOGLE_APPLICATION_CREDENTIALS` | ADC discovery | Path to a service-account or supported ADC JSON credential file |
| `GOOGLE_CLOUD_QUOTA_PROJECT` | unset | Required consumer project for OAuth Google API calls |
| `GOOGLE_MAPS_LANGUAGE` | `en` | Google response language |
| `GOOGLE_MAPS_TIMEOUT` | `30` | Timeout per Google request, in seconds |
| `GOOGLE_MAPS_ENABLED_TOOLS` | all | Comma-separated `maps_*` names, or `*` |
| `MCP_HTTP_TOKEN` | unset | HTTP bearer token; required outside loopback |
| `MCP_HTTP_LISTEN` | `127.0.0.1:3000` | HTTP bind IP:port |
| `MCP_HTTP_ALLOWED_HOSTS` | loopback hosts | Comma-separated HTTP Host allowlist |
| `MCP_HTTP_ALLOWED_ORIGINS` | none | Comma-separated browser origins with explicit ports |
| `RUST_LOG` | `warn` | Diagnostic logging to stderr |

`--language`, `--timeout`, `--enabled-tools`, and HTTP `--listen` override the
corresponding values. Repeated `--allowed-host` / `--allowed-origin` flags replace
the entire corresponding list. Empty environment values override file values;
empty optional credentials/lists clear them, while an empty timeout is invalid.
Application settings are read once at startup, including mounted Secret files.
TLS certificate/key/client-CA files can be reloaded with SIGHUP; see Production
configuration below for the additional TLS, limit, metric and logging settings.

For project billing, API enablement, key creation and service-account setup, see
the bundled `docs/google-cloud-setup.md` or the
[repository setup guide](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/google-cloud-setup.md).
OAuth mode uses Google's official Rust auth library, the `cloud-platform` scope,
cached/refreshed access tokens and a quota-project header for Places (New), Routes,
Weather, Air Quality, Address Validation, Places Aggregate, Solar and Map Tiles.
Datasets and Route Optimization require OAuth with operation-specific IAM.
Street View Publish requires explicit authorized_user credentials with consent
for the separate `streetviewpublish` scope; service-account credentials are
rejected for publishing. Other current endpoints (Geocoding v3, Elevation, Time
Zone, Static Maps, Pollen, Roads, Geolocation, Aerial View and Street View Static) still
use `GOOGLE_MAPS_API_KEY`; composite flows may need it.
With no credential file selected, ADC can discover local credentials or use the
attached service/workload account. Credential loading is lazy for discovery.
Tokens are not MCP client credentials: the HTTP server still uses `MCP_HTTP_TOKEN`.

The repository includes a multi-stage Dockerfile and Kubernetes example. The
container runs as UID/GID 10001 and defaults to `serve http` on `0.0.0.0:3000`,
requiring `MCP_HTTP_TOKEN`. CLI commands and `serve stdio` are also available by
overriding its command. See the bundled `docs/containers.md` guide.

Enable the Google APIs used by your commands: Places API (New), Geocoding API,
Routes API, Elevation API, Time Zone API, Maps Static API, Weather API, Air Quality
API, Address Validation API, Pollen API, Places Aggregate API and Roads API.
Google requests can incur charges. Composite tools make multiple
requests, with up to five concurrent Google requests per process.

## CLI

```sh
mcp-gmap-rs tools
mcp-gmap-rs schema search-nearby
mcp-gmap-rs geocode --address 'Paris, France'
mcp-gmap-rs search-places --query 'coffee in Paris' --open-now
mcp-gmap-rs call maps_geocode --args '{"address":"Paris, France"}'
mcp-gmap-rs static-map --center 'Paris' --zoom 12 --output paris.png
```

Tool calls print JSON to stdout and failures to stderr. Failures produce a
nonzero exit status. `doctor` prints its JSON report even on failed checks.
`--output` writes JSON to a file, or raw bytes for binary results.
Object and array flags take JSON; boolean flags accept `true` or `false`, and a
bare boolean flag means `true`. Use `COMMAND --help` for its input flags and
`schema COMMAND` for its JSON Schema.

Batch and composite results can contain per-item errors even when a command
returns successfully. Inspect those errors as well as its exit status. Times use
RFC 3339 with a timezone, except `timezone`'s optional `timestamp`, which uses
Unix milliseconds. Exact coordinate waypoints are preserved during routing.

## MCP transports

Start the stdio server with:

```sh
mcp-gmap-rs serve stdio
```

Running the executable without arguments also starts stdio. Standard input and
output carry newline-delimited MCP JSON-RPC; diagnostics go to stderr.

Start Streamable HTTP with:

```sh
mcp-gmap-rs serve http --listen 127.0.0.1:3000
```

The MCP endpoint is `http://127.0.0.1:3000/mcp`. `GET /health` checks process
liveness only. HTTP validates Host and Origin headers. For a non-loopback bind,
configure `MCP_HTTP_TOKEN` and allowed hosts/origins using the server's flags;
clients send `Authorization: Bearer <token>`. External HTTPS can use native TLS
or a TLS gateway; see Production configuration below.
All MCP clients share the process's configured Google credentials and request
budgets. Inbound MCP bearer tokens are separate from Google API keys/OAuth.

## Tool catalog

The [Tool reference](#tool-reference) below includes each tool's parameter tables,
defaults, restrictions, result descriptions and equivalent MCP/CLI examples.

| MCP name | CLI command | Capability |
| --- | --- | --- |
| [`maps_geocode`](#maps_geocode) | `geocode` | Address to coordinates |
| [`maps_validate_address`](#maps_validate_address) | `validate-address` | Address validation and corrections |
| [`maps_pollen`](#maps_pollen) | `pollen` | Daily pollen forecasts |
| [`maps_area_insights`](#maps_area_insights) | `area-insights` | Place counts and filtered area insights |
| [`maps_snap_to_roads`](#maps_snap_to_roads) | `snap-to-roads` | Match an ordered GPS path to roads |
| [`maps_nearest_roads`](#maps_nearest_roads) | `nearest-roads` | Nearest roads for independent coordinates |
| [`maps_reverse_geocode`](#maps_reverse_geocode) | `reverse-geocode` | Coordinates to address |
| [`maps_search_places`](#maps_search_places) | `search-places` | Places text search |
| [`maps_search_nearby`](#maps_search_nearby) | `search-nearby` | Nearby places by type |
| [`maps_place_details`](#maps_place_details) | `place-details` | Details, reviews and optional photo URLs |
| [`maps_directions`](#maps_directions) | `directions` | Route and navigation instructions |
| [`maps_distance_matrix`](#maps_distance_matrix) | `distance-matrix` | Distances and durations across locations |
| [`maps_elevation`](#maps_elevation) | `elevation` | Geographic elevation |
| [`maps_timezone`](#maps_timezone) | `timezone` | Timezone offsets and local time |
| [`maps_weather`](#maps_weather) | `weather` | Current, daily and hourly weather |
| [`maps_air_quality`](#maps_air_quality) | `air-quality` | Air quality indices and health advice |
| [`maps_static_map`](#maps_static_map) | `static-map` | PNG maps, markers and paths |
| [`maps_batch_geocode`](#maps_batch_geocode) | `batch-geocode` | Up to 50 addresses per call |
| [`maps_explore_area`](#maps_explore_area) | `explore-area` | Categorized places with details |
| [`maps_plan_route`](#maps_plan_route) | `plan-route` | Multi-stop routing and optimization |
| [`maps_compare_places`](#maps_compare_places) | `compare-places` | Place comparison with optional travel distance |
| [`maps_search_along_route`](#maps_search_along_route) | `search-along-route` | Places near a route polyline |
| [`maps_local_rank_tracker`](#maps_local_rank_tracker) | `local-rank-tracker` | Geographic search-rank grid |

| [`maps_speed_limits`](#maps_speed_limits) | `speed-limits` | Restricted Asset Tracking speed limits |
| [`maps_autocomplete`](#maps_autocomplete) | `autocomplete` | Places predictions and session tokens |
| [`maps_geolocate`](#maps_geolocate) | `geolocate` | Radio/WiFi device geolocation |
| [`maps_solar`](#maps_solar) | `solar` | Building insights, data layers and GeoTIFF |
| [`maps_aerial_view`](#maps_aerial_view) | `aerial-view` | US video lookup and metadata |
| [`maps_render_aerial_video`](#maps_render_aerial_video) | `render-aerial-video` | Explicit asynchronous video rendering |
| [`maps_map_tiles`](#maps_map_tiles) | `map-tiles` | 2D, 3D and Street View visualization assets |
| [`maps_streetview`](#maps_streetview) | `streetview` | Static panorama metadata and JPEG |
| [`maps_streetview_publish`](#maps_streetview_publish) | `streetview-publish` | End-user photo/sequence publishing |
| [`maps_datasets`](#maps_datasets) | `datasets` | OAuth dataset lifecycle and GCS import |
| [`maps_route_optimization`](#maps_route_optimization) | `route-optimization` | Native fleet models, validation and jobs |
| [`maps_air_quality_forecast`](#maps_air_quality_forecast) | `air-quality-forecast` | Hourly air quality forecast |
| [`maps_air_quality_history`](#maps_air_quality_history) | `air-quality-history` | Previous 30 days of air quality |
| [`maps_air_quality_heatmap`](#maps_air_quality_heatmap) | `air-quality-heatmap` | Air quality PNG tile |
| [`maps_pollen_heatmap`](#maps_pollen_heatmap) | `pollen-heatmap` | Pollen PNG tile |
| [`maps_weather_history`](#maps_weather_history) | `weather-history` | Previous 24 hours of weather |
| [`maps_weather_alerts`](#maps_weather_alerts) | `weather-alerts` | Public weather alerts |
| [`maps_weather_minute_forecast`](#maps_weather_minute_forecast) | `weather-minute-forecast` | Experimental minute precipitation |
| [`maps_weather_map`](#maps_weather_map) | `weather-map` | Experimental precipitation PNG tile |

Version 0.4.0 includes the expanded service and routing catalog.
Mutating tools include conservative MCP write/destructive hints and never retry writes automatically.
Native request body objects preserve Google API fields, validated by Google; wrapper arguments are strict.

MCP JSON results include text and structured content. Static maps return MCP PNG
image content; new PNG/JPEG/WebP binaries also preserve provider cache headers.
Other binary assets use MIME/base64 JSON; CLI `--output` writes raw bytes.
Uploads are limited to 8 MiB decoded, request JSON to 12 MiB and responses to 16 MiB.
Place results preserve Google field names and source metadata. Display review
authors, photo attribution, source links, and AI disclosures with their associated
content, following [Google's Places policies](https://developers.google.com/maps/documentation/places/web-service/policies).

## Optional traffic segments

Directions and route planning accept `polylineQuality: "HIGH_QUALITY"` or
`"OVERVIEW"` independently of traffic. An explicit quality wins; omitted quality
keeps Google's overview default unless traffic segments request high quality.
Higher quality returns more geometry and can increase response size and latency.
The executable preserves provider points without interpolation.

Directions and route planning accept `trafficSegments: true` for driving.
This opts into Google traffic-on-polyline calculations and higher Compute Routes
Enterprise billing. Normalized route/leg segments preserve Google point
boundaries and traffic categories. Native geometry/advisories remain available.
Road category, posted speed limit and traffic observation time are `null` because
Routes does not provide them; `retrievedAt` is only the local fetch timestamp.
There are no application-specific speed multipliers or automatic Roads calls.
See the [segment contract](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/route-segments.md)
for field names, independent index scopes, boundary reconstruction and examples.

## Documentation and tests

The source package and release archives include `skills/mcp-gmap/`, a reusable
agent skill for MCP and CLI usage. It covers capability discovery, tool selection,
partial failures, attribution and authentication. See the
[skill entry point](https://docs.rs/crate/mcp-gmap-rs/latest/source/skills/mcp-gmap/SKILL.md)
and the README's installation instructions. Installing the skill does not configure
an MCP connection or supply Google credentials.

The public Rust API is checked for missing documentation and broken symbol links.
The example on [`run`] is compile-checked without making Google requests. The
offline tests exercise all 42 tools and both MCP transports. An opt-in live suite
requires a key, enabled Google APIs, billing and network access.

For the full CLI guide and release procedures, consult the README and documentation
files included in the crate's source package.
