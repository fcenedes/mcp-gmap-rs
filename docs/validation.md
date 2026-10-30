# Validation record

Validated locally on 2026-10-30 with Rust 1.94.1 on macOS arm64.

| Check | Result |
| --- | --- |
| `cargo build --release --locked` | Passed; standalone arm64 executable produced |
| `cargo fmt --check` | Passed |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | Passed |
| `cargo test --locked` | 21 offline tests passed; 18 opt-in live tests ignored by default |
| Opt-in live acceptance suite | All 18 live tests passed across a 16-tool run and targeted Weather / Air Quality runs |
| CLI subprocesses | Discovery, schemas, JSON stdin, configuration and error exit status passed |
| Stdio subprocess | Initialization, discovery, tool calls and EOF shutdown passed |
| Streamable HTTP | Initialization, sessions, SSE GET, calls, inline images and DELETE passed |
| Sessionless HTTP | `server/discover` and `tools/list` with current protocol metadata passed |
| HTTP access checks | Bearer auth, default Origin rejection and non-loopback token requirement passed |
| Google fixture coverage | All 18 tools exercised without live API calls |
| Credential hygiene | Local `.env` ignored by Git; key absent from tracked and nonignored deliverables |

The generated executable is `target/release/mcp-gmap-rs`. The fixture suite checks
API request construction and response handling, including coordinate preservation,
geocoding fallback, optimization ordering, matrix element failures, attribution,
request timeouts, and visible partial failures.

## Live Google acceptance tests

A user-provided local test key was used for requests from this machine. All 18
tools were exercised through the executable against Google, then checked with
the repeatable Rust acceptance tests in `tests/live.rs`:

| Operation | Observed result |
| --- | --- |
| `maps_geocode` | Passed; address lookup returned numeric coordinates |
| `maps_reverse_geocode` | Passed; coordinates returned a nonempty formatted address |
| `maps_search_places` | Passed; text search returned place IDs |
| `maps_search_nearby` | Passed; cafe search within 500 meters returned places |
| `maps_place_details` | Passed; matching place ID, photo URL and author attribution array |
| `maps_directions` | Passed; walking route returned positive distance and encoded polyline |
| `maps_distance_matrix` | Passed; one origin and two destinations returned distances and durations without element errors |
| `maps_elevation` | Passed; returned numeric elevation |
| `maps_timezone` | Passed; returned Europe/Paris and a parseable local timestamp |
| `maps_weather` | Passed; current temperature, one-day forecast and one-hour forecast |
| `maps_air_quality` | Passed; returned indices, pollutants and health recommendations |
| `maps_static_map` | Passed; returned a valid PNG signature with requested 320x240 dimensions |
| `maps_batch_geocode` | Passed; both input addresses returned successful results |
| `maps_explore_area` | Passed; cafe category included details for its top result |
| `maps_plan_route` | Passed; four-stop optimized route preserved endpoints and returned three legs |
| `maps_compare_places` | Passed; returned place details and driving distance from the supplied location |
| `maps_search_along_route` | Passed; cafe search returned a place and route polyline |
| `maps_local_rank_tracker` | Passed; all nine grid-point searches succeeded and returned rank metrics |
| Live geocode over MCP stdio | Passed; returned successful structured content with numeric latitude |
| Live geocode over MCP Streamable HTTP | Passed; returned successful structured content with numeric latitude |

The first attempt was denied by the Google key's source-IP restriction. After
the user updated the restriction, the initial smoke tests succeeded. An expanded
batch request initially had one per-address IP denial; the subsequent live Rust
test passed both addresses. Weather and Air Quality initially returned API-disabled
errors; both passed after the user enabled those APIs. No runtime implementation
changes were needed for these live checks.

The live tests check nested errors as well as exit status, so a composite tool
with per-item failures does not count as a pass. The 18 successful live tests
were run as 16 tests together, followed by separate Weather and Air Quality tests
after API enablement. The default offline suite remains independent of credentials.

To repeat all live tests with `GOOGLE_MAPS_API_KEY` exported:

```sh
cargo test --locked --test live -- --ignored --test-threads=1
```

These checks cover small Paris-based scenarios using one key. Other locations,
travel modes, quota limits and all optional parameter combinations have not been
exhaustively validated. The suite makes billable Google requests.

The key is never included in this report. The originating public IP and raw Google
error bodies are also intentionally excluded from repository documentation.
See [Google's key restriction guidance](https://developers.google.com/maps/api-security-best-practices#restrict-your-api-keys)
for matching server-side keys to the source address observed by Google.

The local semantic index could not be built because its Python SQLite connection
lacks `enable_load_extension`. Direct source inspection was used; this did not
prevent Cargo builds or tests.

## docs.rs preparation after v0.1.0

The documentation changes prepared for v0.1.1 add explicit docs.rs metadata,
a standalone CLI/MCP guide, and a compile-only Rust example. Stable Rust and
nightly 1.101.0 built the public docs locally with `DOCS_RS=1` and warnings treated
as errors. Rustdoc measured 2 of 2 public items documented (100%) and 1 of 1
eligible items with examples (100%). This is public-API coverage, not private
implementation documentation coverage. The example compiled on both toolchains
without making Google requests. The Cargo package includes `docs/crate.md` and
still passes compilation and file auditing.

The registry index and docs.rs endpoint returned HTTP 404 during the initial check;
hosted docs were not yet available. docs.rs builds published crates, not GitHub
tags or unreleased `main` changes. Version v0.1.1 includes these
changes; the existing v0.1.0 tag and archives have not been modified. CI now builds
docs on Linux with stable and nightly, reports coverage, and checks examples.
