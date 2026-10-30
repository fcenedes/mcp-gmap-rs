# mcp-gmap-rs

A standalone Rust executable for Google Maps: 
**CLI**, **MCP stdio**, and **MCP Streamable HTTP**. 

[![CI](https://github.com/fcenedes/mcp-gmap-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/fcenedes/mcp-gmap-rs/actions/workflows/ci.yml)

Prebuilt executables for Linux and macOS (Apple Silicon and Intel) are
published on the [Releases page](https://github.com/fcenedes/mcp-gmap-rs/releases).
Each archive includes documentation and licenses; use `SHA256SUMS` to verify it.
See [release automation](docs/releases.md) for the CI and publishing process.

## Build and install

Install a stable Rust toolchain (Rust 1.88 or newer; validated with 1.94.1), then:

```sh
cargo build --release --locked
./target/release/mcp-gmap-rs --help

# Or install the executable in your Cargo bin directory:
cargo install --path . --locked
```

`target/release/mcp-gmap-rs` is the executable. It needs no source checkout or
language runtime after building. HTTPS uses the operating system certificate
store; the host still needs working CA certificates and network access to Google.

## Configure Google access

```sh
export GOOGLE_MAPS_API_KEY='your-key'
```

Enable billing and the Google APIs used by your tools: Places API (New),
Geocoding API, Routes API, Elevation API, Time Zone API, Maps Static API,
Weather API, and Air Quality API. You only need APIs for tools you actually call.
Calls can incur charges; composite tools make multiple API requests. Availability
and travel-mode coverage depend on Google and the region.

The executable reads environment variables; it does **not** automatically load
`.env` files. See [.env.example](.env.example). API keys stay in process
configuration and are never accepted as MCP tool arguments or HTTP query parameters.

| Variable | Default | Purpose |
| --- | --- | --- |
| `GOOGLE_MAPS_API_KEY` | unset | Server/CLI credential for Google API calls |
| `GOOGLE_MAPS_LANGUAGE` | `en` | Google response language |
| `GOOGLE_MAPS_TIMEOUT` | `30` | Timeout per HTTP request, 1–600 seconds |
| `GOOGLE_MAPS_ENABLED_TOOLS` | all | Comma-separated `maps_*` tool names, or `*` |
| `MCP_HTTP_TOKEN` | unset | Bearer token for HTTP clients; required for non-loopback binds |
| `RUST_LOG` | `warn` | Diagnostic log filter; logs go to stderr |

`--language`, `--timeout`, and `--enabled-tools` override their environment values.
Unknown tool names in an allowlist fail startup. Tool discovery and input-schema
inspection work without an API key; actual API calls need one.

## CLI

All commands print JSON to stdout and errors to stderr, with a nonzero exit status
on failure. `--output` writes JSON to a file, or raw PNG for `static-map`.

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
headers. TLS termination and external identity/OAuth are deployment concerns;
this executable implements a configured bearer token, not an OAuth provider.
It uses one Google key per process; run separate instances for separate Google
credentials. Request bodies are limited to 1 MiB. Ctrl-C and SIGTERM shut down
the server and active HTTP sessions.

## Tools

| MCP tool | CLI command |
| --- | --- |
| `maps_geocode` | `geocode` |
| `maps_reverse_geocode` | `reverse-geocode` |
| `maps_search_places` | `search-places` |
| `maps_search_nearby` | `search-nearby` |
| `maps_place_details` | `place-details` |
| `maps_directions` | `directions` |
| `maps_distance_matrix` | `distance-matrix` |
| `maps_elevation` | `elevation` |
| `maps_timezone` | `timezone` |
| `maps_weather` | `weather` |
| `maps_air_quality` | `air-quality` |
| `maps_static_map` | `static-map` |
| `maps_batch_geocode` | `batch-geocode` |
| `maps_explore_area` | `explore-area` |
| `maps_plan_route` | `plan-route` |
| `maps_compare_places` | `compare-places` |
| `maps_search_along_route` | `search-along-route` |
| `maps_local_rank_tracker` | `local-rank-tracker` |

Place data keeps Google field names and attribution metadata. Display review
authors/source links, photo author attribution, and AI disclosures with the
associated content, following [Google's Places policies](https://developers.google.com/maps/documentation/places/web-service/policies).
MCP map calls return inline PNG image content; CLI map calls print base64 JSON
unless `--output` is provided.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

The default tests use a local Google API fixture and exercise all 18 tools, CLI
subprocesses, stdio initialization and tool calls, plus HTTP sessions, auth, origin
validation, and image responses. They make no live Google calls and require no key.

There are also 18 opt-in live acceptance tests in `tests/live.rs`, one for each
tool. They check returned data, photo URLs, PNG dimensions, optimized route
endpoints, matrix entries, all nine rank-grid searches, and nested composite
errors. The weather test covers current conditions and daily/hourly forecasts.
These tests require `GOOGLE_MAPS_API_KEY` exported in the environment, enabled
APIs, billing and network access, and make billable Google requests:

```sh
cargo test --locked --test live -- --ignored --test-threads=1
```

Live tests are ignored in ordinary `cargo test` and CI runs. Run a single tool
with, for example, `cargo test --locked --test live live_maps_weather -- --ignored`.
See the [validation record](docs/validation.md) for the checks run and live-test status.

MIT licensed. 
