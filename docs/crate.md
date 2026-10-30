# Google Maps CLI and MCP server

`mcp-gmap-rs` provides 18 Google Maps tools through one Rust executable:
a JSON CLI, an MCP stdio server, and an MCP Streamable HTTP server. All three
interfaces share argument validation and Google API execution.

This crate is primarily distributed as an executable. Its public Rust entry point
is [`run`], which drives the same command-line application inside a Tokio runtime.
The underlying Google clients are implementation details, not a public Rust SDK.

## Install

After the crate version is published on crates.io:

```sh
cargo install mcp-gmap-rs --locked
```

For an existing source checkout, use `cargo install --path . --locked`.
Rust 1.88 or newer is required to build from source. Prebuilt Linux and macOS
executables are available from the repository's GitHub Releases page.

## Configuration

Export `GOOGLE_MAPS_API_KEY` in the environment before making Google requests.
The executable does not automatically load `.env` files. Discovery and schema
inspection work without a key. API keys are process configuration and are not
accepted as tool arguments.

| Variable | Default | Purpose |
| --- | --- | --- |
| `GOOGLE_MAPS_API_KEY` | unset | Credential for Google API requests |
| `GOOGLE_MAPS_LANGUAGE` | `en` | Google response language |
| `GOOGLE_MAPS_TIMEOUT` | `30` | Timeout per Google request, in seconds |
| `GOOGLE_MAPS_ENABLED_TOOLS` | all | Comma-separated `maps_*` names, or `*` |
| `MCP_HTTP_TOKEN` | unset | HTTP bearer token; required outside loopback |
| `RUST_LOG` | `warn` | Diagnostic logging to stderr |

Enable the Google APIs used by your commands: Places API (New), Geocoding API,
Routes API, Elevation API, Time Zone API, Maps Static API, Weather API, and Air
Quality API. Google requests can incur charges. Composite tools make multiple
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

CLI commands print JSON to stdout and failures to stderr. Failures produce a
nonzero exit status. `--output` writes JSON to a file, or raw PNG for a static map.
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
clients send `Authorization: Bearer <token>`. External HTTPS requires a TLS proxy.
The server uses one configured Google key per process.

## Tool catalog

| MCP name | CLI command | Capability |
| --- | --- | --- |
| `maps_geocode` | `geocode` | Address to coordinates |
| `maps_reverse_geocode` | `reverse-geocode` | Coordinates to address |
| `maps_search_places` | `search-places` | Places text search |
| `maps_search_nearby` | `search-nearby` | Nearby places by type |
| `maps_place_details` | `place-details` | Details, reviews and optional photo URLs |
| `maps_directions` | `directions` | Route and navigation instructions |
| `maps_distance_matrix` | `distance-matrix` | Distances and durations across locations |
| `maps_elevation` | `elevation` | Geographic elevation |
| `maps_timezone` | `timezone` | Timezone offsets and local time |
| `maps_weather` | `weather` | Current, daily and hourly weather |
| `maps_air_quality` | `air-quality` | Air quality indices and health advice |
| `maps_static_map` | `static-map` | PNG maps, markers and paths |
| `maps_batch_geocode` | `batch-geocode` | Up to 50 addresses per call |
| `maps_explore_area` | `explore-area` | Categorized places with details |
| `maps_plan_route` | `plan-route` | Multi-stop routing and optimization |
| `maps_compare_places` | `compare-places` | Place comparison with optional travel distance |
| `maps_search_along_route` | `search-along-route` | Places near a route polyline |
| `maps_local_rank_tracker` | `local-rank-tracker` | Geographic search-rank grid |

MCP JSON results include text and structured content. Static maps return MCP PNG
image content; CLI map results use base64 JSON unless written with `--output`.
Place results preserve Google field names and source metadata. Display review
authors, photo attribution, source links, and AI disclosures with their associated
content, following [Google's Places policies](https://developers.google.com/maps/documentation/places/web-service/policies).

## Documentation and tests

The public Rust API is checked for missing documentation and broken symbol links.
The example on [`run`] is compile-checked without making Google requests. The
offline tests exercise all 18 tools and both MCP transports. An opt-in live suite
requires a key, enabled Google APIs, billing and network access.

For the full CLI guide and release procedures, consult the README and documentation
files included in the crate's source package.
