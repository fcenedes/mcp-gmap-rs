# mcp-gmap-rs

A standalone Rust executable for Google Maps: 
**CLI**, **MCP stdio**, and **MCP Streamable HTTP**. 

- 42 tools across 20 server-side Google API families, including fleet optimization, datasets, imagery and environmental data.
- Explicit license, geographic, preview and credential restrictions in tool discovery and the manual.
- API keys or service-account/ADC OAuth for supported APIs.
- Native TLS/mTLS, request limits, Prometheus metrics and access diagnostics.
- Docker/Kubernetes examples and a portable agent skill for MCP or CLI use.

[![CI](https://github.com/fcenedes/mcp-gmap-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/fcenedes/mcp-gmap-rs/actions/workflows/ci.yml)
[![docs.rs](https://docs.rs/mcp-gmap-rs/badge.svg)](https://docs.rs/mcp-gmap-rs/latest/mcp_gmap_rs/)

**[Read the full manual](https://fcenedes.github.io/mcp-gmap-rs/)** —
[English](https://fcenedes.github.io/mcp-gmap-rs/),
[Français](https://fcenedes.github.io/mcp-gmap-rs/fr/),
[Italiano](https://fcenedes.github.io/mcp-gmap-rs/it/),
[Deutsch](https://fcenedes.github.io/mcp-gmap-rs/de/),
[Español](https://fcenedes.github.io/mcp-gmap-rs/es/),
[简体中文](https://fcenedes.github.io/mcp-gmap-rs/zh/),
[日本語](https://fcenedes.github.io/mcp-gmap-rs/ja/),
[العربية](https://fcenedes.github.io/mcp-gmap-rs/ar/),
[עברית](https://fcenedes.github.io/mcp-gmap-rs/he/).
Installation, Google Cloud setup, authentication, CLI/MCP, production deployment,
all 21 configuration settings, and detailed references for every tool.

The [docs.rs guide](https://docs.rs/mcp-gmap-rs/latest/mcp_gmap_rs/) contains the
tool catalog, CLI/MCP setup, and public Rust entry-point documentation. Hosted
documentation becomes available after a crate version is published to crates.io.

Prebuilt executables for Linux and macOS (Apple Silicon and Intel) are
published on the [Releases page](https://github.com/fcenedes/mcp-gmap-rs/releases).
Each archive includes documentation and licenses; use `SHA256SUMS` to verify it.
See [release automation](docs/releases.md) for the CI and publishing process.

The 42-tool catalog describes the current **unreleased source**. The published
0.3.1 crate has 23 tools; build from this checkout to use the additions.
See [API coverage and provider limits](docs/service-research/coverage.md).

## Build and install

Install Rust 1.88 or newer. CI checks the minimum and current stable toolchains;
release binaries are built with Rust 1.94.1. Install this version from crates.io:

```sh
cargo install mcp-gmap-rs --version 0.3.1 --locked
```

Or build from a source checkout:

```sh
cargo build --release --locked
./target/release/mcp-gmap-rs --help

# Or install the executable in your Cargo bin directory:
cargo install --path . --locked
```

`target/release/mcp-gmap-rs` is the executable. It needs no source checkout or
language runtime after building. Outbound HTTPS uses the operating system
certificate store; the host needs working CA certificates and network access to Google.

## Configure Google access

Follow the [Google Cloud setup guide](docs/google-cloud-setup.md) for project
billing, API enablement, restricted API-key creation and service-account OAuth
with JSON credentials, local ADC or Kubernetes/GKE workload identity.

```sh
export GOOGLE_MAPS_API_KEY='your-key'
```

Enable billing and the Google APIs used by your tools: Places API (New),
Geocoding API, Routes API, Elevation API, Time Zone API, Maps Static API,
Weather API, Air Quality API, Address Validation API, Pollen API, Places Aggregate
API and Roads API for the original catalog. The expanded source also uses
Geolocation, Solar, Aerial View, Map Tiles, Street View Static, Maps Platform
Datasets, Route Optimization and Street View Publish. You only need APIs for
tools you actually call. The setup guide lists exact service IDs and credential
requirements.
Calls can incur charges; composite tools make multiple API requests. Availability
and travel-mode coverage depend on Google and the region.

Configuration is layered: **defaults → optional `.env` → environment variables →
CLI flags**. The default file is `.env` in the working directory; parent directories
are not searched. See [.env.example](.env.example). API keys stay in process
configuration and are never accepted as MCP tool arguments or MCP HTTP query parameters.

```sh
cp .env.example .env
# Edit .env, then run normally:
mcp-gmap-rs tools
# Or select a mounted configuration file:
mcp-gmap-rs --env-file /etc/mcp-gmap/runtime.env serve http
# Use only environment variables and flags:
mcp-gmap-rs --no-env-file serve http
```

`MCP_ENV_FILE` also selects a file; `--env-file` takes precedence. An explicitly
selected file must exist. A missing default `.env` is allowed; unreadable or
malformed files fail startup without printing their contents. `--no-env-file`
disables all file loading. Files are parsed without modifying the process environment.

| Variable | Default | Purpose |
| --- | --- | --- |
| `GOOGLE_MAPS_API_KEY` | unset | Server/CLI credential for Google API calls |
| `GOOGLE_MAPS_AUTH_MODE` | `api-key` | `api-key` or `oauth`; OAuth is used for Places (New), Routes, Weather, Air Quality, Address Validation and Places Aggregate |
| `GOOGLE_APPLICATION_CREDENTIALS` | ADC discovery | Path to service-account or other supported ADC JSON credentials |
| `GOOGLE_CLOUD_QUOTA_PROJECT` | unset | Required Maps billing/quota project ID or number for OAuth requests |
| `GOOGLE_MAPS_LANGUAGE` | `en` | Google response language |
| `GOOGLE_MAPS_TIMEOUT` | `30` | Timeout per HTTP request, 1–600 seconds |
| `GOOGLE_MAPS_ENABLED_TOOLS` | all | Comma-separated `maps_*` tool names, or `*` |
| `MCP_HTTP_TOKEN` | unset | Bearer token for HTTP clients; required for non-loopback binds |
| `MCP_HTTP_LISTEN` | `127.0.0.1:3000` | HTTP bind address; the Docker image defaults to `0.0.0.0:3000` |
| `MCP_HTTP_ALLOWED_HOSTS` | loopback hosts | Comma-separated allowed HTTP Host values |
| `MCP_HTTP_ALLOWED_ORIGINS` | none | Comma-separated browser origins; specify ports explicitly |
| `RUST_LOG` | `warn` | Diagnostic log filter; logs go to stderr |
| `MCP_LOG_FORMAT` | `text` | `text` or `json` diagnostic events |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | Google HTTP attempt budget per tool call, including retries |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | Additional transient attempts per Google request |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | Overall tool deadline in seconds |
| `MCP_HTTP_RATE_LIMIT` | `120` | Authenticated MCP HTTP requests/minute per process; `0` disables |
| `MCP_METRICS_ENABLED` | `false` | Enable protected Prometheus `/metrics` |
| `MCP_TLS_CERT` | unset | PEM server certificate chain file |
| `MCP_TLS_KEY` | unset | Matching PEM server private key file |
| `MCP_TLS_CLIENT_CA` | unset | PEM client CA bundle; require client certificates |

`--auth-mode`, `--language`, `--timeout`, and `--enabled-tools` override their environment values.
HTTP `--listen`, `--allowed-host`, and `--allowed-origin` do the same. Repeated
allowlist flags replace the corresponding file/environment list as a whole.
An explicitly empty environment value also overrides the file; it can clear an
optional token or allowlist, but an empty timeout is invalid. Configuration is
read at startup; restart the process after changing application settings or
Kubernetes Secrets. TLS certificate/key/client-CA files can be reloaded with
SIGHUP; see the [production guide](docs/production.md).
Unknown tool names in an allowlist fail startup. Tool discovery and input-schema
inspection work without credentials; actual API calls need credentials supported
by the selected endpoints.

In OAuth mode, keep `GOOGLE_MAPS_API_KEY` for the current Geocoding v3, Elevation,
Time Zone, Static Maps, Pollen, Roads, Geolocation, Aerial View and Street View
Static endpoints if used. Solar and Map Tiles also support OAuth. Datasets and
Route Optimization require OAuth/IAM. Street View Publish requires an explicit
end-user OAuth credential file with `streetviewpublish` consent; a service account
cannot publish user imagery. Composite/location-resolution flows
may need both credential types. OAuth-supported calls send a cached/refreshed
access token and quota project, with no API key; an OAuth failure does not retry
using a key. Google OAuth configures outbound Google access independently of
the MCP server's `MCP_HTTP_TOKEN`.

For TLS/mTLS, metrics, limits and diagnostics, see the [production guide](docs/production.md).
For containers and Kubernetes Secrets/ConfigMaps, see the
[Docker and Kubernetes guide](docs/containers.md) and [Dockerfile](Dockerfile).

## CLI

Tool calls print JSON to stdout and errors to stderr, with a nonzero exit status
on failure. `doctor` also prints its JSON report when checks fail. `--output`
writes JSON to a file, or raw image/raster/dataset/3D bytes for binary results.
Binary JSON output includes `mimeType`, base64 data and provider cache headers;
MCP PNG/JPEG/WebP results use image content.

```sh
mcp-gmap-rs tools
mcp-gmap-rs schema search-nearby
mcp-gmap-rs geocode --address 'Paris, France'
mcp-gmap-rs reverse-geocode --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs search-places --query 'coffee in Paris' --open-now --min-rating 4
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 1000
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --max-photos 1
mcp-gmap-rs directions --origin 'Paris' --destination 'Versailles' --mode driving
mcp-gmap-rs plan-route --stops '["Eiffel Tower","Louvre","Montmartre","Notre Dame, Paris"]'
mcp-gmap-rs batch-geocode --addresses '["Paris","London","Rome"]'
mcp-gmap-rs static-map --center 'Paris' --zoom 12 --output paris.png
mcp-gmap-rs validate-address --address-lines '["5 Avenue Anatole France"]' --region-code FR
mcp-gmap-rs pollen --latitude 48.8584 --longitude 2.2945 --days 3
mcp-gmap-rs area-insights \
  --area '{"circle":{"latLng":{"latitude":48.8584,"longitude":2.2945},"radius":500}}' \
  --included-types '["restaurant"]' --min-rating 4
mcp-gmap-rs snap-to-roads \
  --path '[{"latitude":-35.27801,"longitude":149.12958},{"latitude":-35.28032,"longitude":149.12907}]'
mcp-gmap-rs nearest-roads --points '[{"latitude":48.8584,"longitude":2.2945}]'
mcp-gmap-rs doctor
```

Use `COMMAND --help` for flags. Object/array flags take JSON; boolean flags accept
`true` or `false`, and a bare boolean flag means `true`. Times use RFC 3339 with a
timezone. The `timezone` tool's optional `timestamp` uses Unix **milliseconds**.

The generic interface accepts the same JSON as MCP, using either the MCP name or
the CLI command name:

```sh
mcp-gmap-rs call maps_geocode --args '{"address":"Paris"}'
printf '%s' '{"address":"Paris"}' | mcp-gmap-rs call geocode --args -
```

Batch and composite results can contain per-item errors alongside useful data.
Inspect those fields as well as the command's exit status. These operations limit
concurrent Google requests to five per process and preserve input order.

## MCP stdio

```sh
mcp-gmap-rs serve stdio
# Running with no arguments does the same thing:
mcp-gmap-rs
```

Standard input/output carries newline-delimited MCP JSON-RPC only. Diagnostics
go to stderr. A generic MCP client configuration is:

```json
{
  "mcpServers": {
    "google-maps": {
      "command": "/absolute/path/to/mcp-gmap-rs",
      "args": ["serve", "stdio"],
      "env": { "GOOGLE_MAPS_API_KEY": "your-key" }
    }
  }
}
```

This is an MCP server configuration, not an assistant-specific plugin.

## MCP Streamable HTTP

```sh
mcp-gmap-rs serve http --listen 127.0.0.1:3000
```

Connect an MCP client to **`http://127.0.0.1:3000/mcp`**. The official
[Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk) implements protocol
negotiation, JSON-RPC, SSE streaming, and legacy MCP sessions. The endpoint supports
POST, GET, and DELETE as appropriate to the negotiated protocol. `GET /health`
is process liveness only; it does not call Google or validate credentials.

For authenticated access through a TLS reverse proxy:

```sh
export MCP_HTTP_TOKEN='a-long-random-secret'
mcp-gmap-rs serve http --listen 0.0.0.0:3000 \
  --allowed-host maps.example.com \
  --allowed-origin https://app.example.com:443
```

Clients send `Authorization: Bearer <MCP_HTTP_TOKEN>`. The server defaults to
loopback Host values and rejects any present Origin header unless explicitly
allowed. Repeat `--allowed-host` / `--allowed-origin` for additional values.
The proxy must forward a permitted Host and preserve authorization and MCP/SSE
headers. Native TLS/mTLS and a Kubernetes TLS gateway example are documented in the
[production guide](docs/production.md), together with rate limits, request budgets,
metrics, JSON logs and `doctor`. External identity/OAuth remains a deployment concern;
this executable authenticates HTTP clients with a configured bearer token.
It uses one Google key per process; run separate instances for separate Google
credentials. Request bodies are limited to 12 MiB. Ctrl-C and SIGTERM shut down
the server and active HTTP sessions.

To serve HTTPS directly, supply a matching certificate/key and the runtime bearer
token. Set `RUST_LOG=info` to see tool completion events:

```sh
mcp-gmap-rs --log-format json serve http --listen 0.0.0.0:3443 \
  --tls-cert /etc/mcp-gmap/tls/tls.crt --tls-key /etc/mcp-gmap/tls/tls.key \
  --allowed-host maps.example.com:3443 --metrics --rate-limit 120
```

Add `--tls-client-ca /etc/mcp-gmap/tls/client-ca.pem` to require client certificates.
For diagnostics, `doctor` is offline; `doctor --live` opts into billable probes
and can be scoped with `--enabled-tools`. The [production guide](docs/production.md)
documents request-budget flags, metric names, TLS renewal and deployment limits.

## Tools

The [complete tool reference](docs/tools.md) documents every tool's purpose,
parameters (including nested fields), required/optional status, defaults, types,
limits, response shape, restrictions and MCP/CLI examples.

| MCP tool | CLI command |
| --- | --- |
| [`maps_geocode`](docs/tools.md#maps_geocode) | `geocode` |
| [`maps_validate_address`](docs/tools.md#maps_validate_address) | `validate-address` |
| [`maps_pollen`](docs/tools.md#maps_pollen) | `pollen` |
| [`maps_area_insights`](docs/tools.md#maps_area_insights) | `area-insights` |
| [`maps_snap_to_roads`](docs/tools.md#maps_snap_to_roads) | `snap-to-roads` |
| [`maps_nearest_roads`](docs/tools.md#maps_nearest_roads) | `nearest-roads` |
| [`maps_reverse_geocode`](docs/tools.md#maps_reverse_geocode) | `reverse-geocode` |
| [`maps_search_places`](docs/tools.md#maps_search_places) | `search-places` |
| [`maps_search_nearby`](docs/tools.md#maps_search_nearby) | `search-nearby` |
| [`maps_place_details`](docs/tools.md#maps_place_details) | `place-details` |
| [`maps_directions`](docs/tools.md#maps_directions) | `directions` |
| [`maps_distance_matrix`](docs/tools.md#maps_distance_matrix) | `distance-matrix` |
| [`maps_elevation`](docs/tools.md#maps_elevation) | `elevation` |
| [`maps_timezone`](docs/tools.md#maps_timezone) | `timezone` |
| [`maps_weather`](docs/tools.md#maps_weather) | `weather` |
| [`maps_air_quality`](docs/tools.md#maps_air_quality) | `air-quality` |
| [`maps_static_map`](docs/tools.md#maps_static_map) | `static-map` |
| [`maps_batch_geocode`](docs/tools.md#maps_batch_geocode) | `batch-geocode` |
| [`maps_explore_area`](docs/tools.md#maps_explore_area) | `explore-area` |
| [`maps_plan_route`](docs/tools.md#maps_plan_route) | `plan-route` |
| [`maps_compare_places`](docs/tools.md#maps_compare_places) | `compare-places` |
| [`maps_search_along_route`](docs/tools.md#maps_search_along_route) | `search-along-route` |
| [`maps_local_rank_tracker`](docs/tools.md#maps_local_rank_tracker) | `local-rank-tracker` |

| [`maps_datasets`](docs/tools.md#maps_datasets) | `datasets` |
| [`maps_route_optimization`](docs/tools.md#maps_route_optimization) | `route-optimization` |
| [`maps_air_quality_forecast`](docs/tools.md#maps_air_quality_forecast) | `air-quality-forecast` |
| [`maps_air_quality_history`](docs/tools.md#maps_air_quality_history) | `air-quality-history` |
| [`maps_air_quality_heatmap`](docs/tools.md#maps_air_quality_heatmap) | `air-quality-heatmap` |
| [`maps_pollen_heatmap`](docs/tools.md#maps_pollen_heatmap) | `pollen-heatmap` |
| [`maps_weather_history`](docs/tools.md#maps_weather_history) | `weather-history` |
| [`maps_weather_alerts`](docs/tools.md#maps_weather_alerts) | `weather-alerts` |
| [`maps_weather_minute_forecast`](docs/tools.md#maps_weather_minute_forecast) | `weather-minute-forecast` |
| [`maps_weather_map`](docs/tools.md#maps_weather_map) | `weather-map` |
| [`maps_map_tiles`](docs/tools.md#maps_map_tiles) | `map-tiles` |
| [`maps_streetview`](docs/tools.md#maps_streetview) | `streetview` |
| [`maps_streetview_publish`](docs/tools.md#maps_streetview_publish) | `streetview-publish` |
| [`maps_speed_limits`](docs/tools.md#maps_speed_limits) | `speed-limits` |
| [`maps_autocomplete`](docs/tools.md#maps_autocomplete) | `autocomplete` |
| [`maps_geolocate`](docs/tools.md#maps_geolocate) | `geolocate` |
| [`maps_solar`](docs/tools.md#maps_solar) | `solar` |
| [`maps_aerial_view`](docs/tools.md#maps_aerial_view) | `aerial-view` |
| [`maps_render_aerial_video`](docs/tools.md#maps_render_aerial_video) | `render-aerial-video` |

Place data keeps Google field names and attribution metadata. Display review
authors/source links, photo author attribution, and AI disclosures with the
associated content, following [Google's Places policies](https://developers.google.com/maps/documentation/places/web-service/policies).
MCP PNG/JPEG/WebP calls return inline image content and binary metadata; CLI binary calls print MIME/base64 JSON
unless `--output` is provided.


## Traffic segments

Set `trafficSegments: true` on `maps_directions` or `maps_plan_route` for
polyline-aligned traffic categories and explicit provenance/availability metadata:

```sh
mcp-gmap-rs directions --origin '37.422,-122.084' \
  --destination '37.7749,-122.4194' --traffic-segments
```

This opt-in requires driving and triggers Google's higher-priced Compute Routes
Enterprise SKU. Traffic categories are not measured speeds or posted limits.
Road classification, posted speed limits and provider observation times remain
`null` because Routes does not return them. A separate `retrievedAt` is the local
fetch time. No application-specific multipliers or implicit Roads requests are
applied. See [the segment contract and examples](docs/route-segments.md).

## Agent skill

The reusable [mcp-gmap skill](skills/mcp-gmap/SKILL.md) teaches agents to choose
the appropriate Maps tools, use MCP or CLI, inspect partial failures, preserve
attribution and handle authentication without exposing credentials. It discovers
available tools before using version-specific capabilities. It supplies instructions;
the executable or MCP connection must be configured separately.

The source package and release archives include `skills/mcp-gmap/`. To install
it for Codex from a checkout or extracted release archive:

```sh
mkdir -p "${CODEX_HOME:-$HOME/.codex}/skills"
cp -R skills/mcp-gmap "${CODEX_HOME:-$HOME/.codex}/skills/"
```

Invoke it as `$mcp-gmap`, for example: “Use $mcp-gmap to find cafes near Gare de
Lyon and compare walking routes.” For other agents, place the same folder in their
supported skill directory or load its `SKILL.md` and linked references. The body
is agent-independent; `agents/openai.yaml` adds optional Codex UI metadata.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

The [complete tool reference](docs/tools.md) is generated from the executable's
schemas and reviewed usage notes. To update it from a repository checkout:

```sh
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs --check --verify-examples
```

These example checks run without credentials or Google calls; CI verifies that
all tools and parameter paths remain documented.

The default tests use local Google API fixtures and exercise all 42 tools, CLI
subprocesses, stdio initialization and tool calls, plus HTTP sessions, auth, origin
validation, and image responses. They make no live Google calls and require no key.

The original suite also has 23 opt-in API-key live acceptance tests in `tests/live.rs`, one for each
tool. They check returned data, photo URLs, PNG dimensions, optimized route
endpoints, matrix entries, all nine rank-grid searches, and nested composite
errors. The weather test covers current conditions and daily/hourly forecasts.
These tests require `GOOGLE_MAPS_API_KEY` exported in the environment, enabled
APIs, billing and network access, and make billable Google requests:

```sh
cargo test --locked --test live live_maps_ -- --ignored --test-threads=1
```

Six additional opt-in tests exercise OAuth on Places, Routes, Weather, Air
Quality, Address Validation and Places Aggregate with the API key removed. See the [OAuth validation instructions](docs/google-cloud-setup.md#4-verify-and-troubleshoot).

Live tests are ignored in ordinary `cargo test` and CI runs. Run a single tool
with, for example, `cargo test --locked --test live live_maps_weather -- --ignored`.
See the [validation record](docs/validation.md) for the checks run and live-test status.

After local verification, run `cargo clean` to reclaim generated build output.
Subsequent source builds will recompile it.

MIT licensed. 
