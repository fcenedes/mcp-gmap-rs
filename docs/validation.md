# Validation record

## Independent geometry-quality validation: 2026-11-01

Directions and route planning accept an explicit polylineQuality choice,
OVERVIEW or HIGH_QUALITY, independently of traffic segments. Explicit selection
wins; omitted quality retains Google’s overview default for ordinary calls and
high quality for traffic-segment calls. The current catalog has 42 tools and
329 parameter paths. All 99 offline tests pass, with 31 live tests opt-in.

Fixture checks cover both qualities on both tools without traffic, walking
routes, explicit overview with traffic/optimized stops, invalid enum values
before resolution, original provider geometry and CLI flag/JSON parity.

Two API-key driving requests for the same California endpoints returned the
same 56,928 m route distance: OVERVIEW had 267 vertices (1,031 encoded bytes),
HIGH_QUALITY had 3,110 vertices (7,688 encoded bytes). Neither response included
normalized traffic segments. These counts describe that observed response, not
a promised density ratio or lane-level accuracy. No Roads call or interpolation
was used. The repeatable high-quality live test is
`live_maps_directions_high_quality_without_traffic_intervals`.

## Opt-in traffic segment validation: 2026-11-01

Directions and route planning now expose `trafficSegments` (false by default),
keeping the catalog at 42 tools and expanding the reference to 327 parameter
paths. The suite has 95 passing offline tests and 30 ignored live tests.
Fixture tests cover opt-in request masks/settings, original route/leg geometry,
waypoint optimization, independent index scopes, timestamp provenance, no
implicit Roads calls, unsupported modes, missing traffic, Google fallback and
OAuth without key fallback. Pure normalization tests cover native omitted-zero
indexes, end boundary M−1, gaps, unknown categories, invalid geometry/indexes and
atomic failure without partially enriched output.

One billable API-key check of a driving route in California returned 16 route
intervals and 16 leg intervals, including NORMAL, SLOW and TRAFFIC_JAM. Traffic
status was AVAILABLE, raw advisory and staticDuration remained present, and
roadCategory/speedLimit/trafficObservedAt remained null. This validates the
adapter against current Google output, not a guarantee of future traffic
conditions. The opt-in repeatable test is
`live_maps_directions_traffic_segments`; Google bills traffic-on-polyline at the
Compute Routes Enterprise tier. No speed-limit or additional Roads request was
made. See [the generic segment contract](route-segments.md).

## v0.4.0 service expansion: 2026-11-01

The source catalog now exposes 42 tools across 20 server-side API families and
329 documented parameter paths. The credential-free reference validator exercises
every example through the executable and checks the missing-key or OAuth-required
boundary. The 99 offline tests and compile-checked documentation example pass locally.
Strict Clippy and rustdoc with docs.rs configuration pass; the Cargo package
audit excludes local credentials and includes the expanded manuals/skill.
New offline fixtures exercise native payloads, verbs, user-scoped OAuth,
write retries, upload bytes, binary MIME/cache headers and fixed-origin resources.
The nine-language manual builds and verifies 135 pages, 329 parameter paths per
language and 11,148 internal links/assets. Prior browser checks at 320px found all
42 tool anchors and no document overflow in all nine locales, including Arabic
and Hebrew RTL. Earlier records below describe their dated releases, not this
expanded catalog.

Small live read-only checks used the supplied restricted key and service-account
credentials; no datasets, render requests, uploads, publications or optimization
jobs were created. The repeatable probes are `.github/scripts/live-services.py`.

| Live scenario | Result |
| --- | --- |
| API key: Autocomplete and Geolocation | Passed |
| API key: Solar building insights and Map Tiles 3D root | Passed |
| API key: Street View metadata and JPEG | Passed; JPEG MIME and binary response |
| API key: Air Quality forecast/history/heatmap and Pollen heatmap | Passed; PNG tiles returned |
| API key: Weather history, public alerts, experimental minute forecast/map | Passed; experimental endpoints remain pre-GA despite live access |
| Roads Speed Limits | Blocked: Google 403, speed limits unavailable for this project; restricted Asset Tracking entitlement is required |
| Aerial View metadata | Blocked: Google 403, `aerialview.googleapis.com` disabled/not previously used in the key project |
| OAuth without API key: Solar and Map Tiles 3D root | Passed |
| OAuth: Datasets list | Blocked: `mapsplatformdatasets.datasets.list` IAM permission denied |
| OAuth: Route Optimization validation only | Blocked: `routeoptimization.locations.use` IAM permission denied |
| Street View Publish | Not run live: supplied service account cannot provide end-user consent; all operations use fixtures |
| Aerial render, dataset mutations, optimization jobs and public publishing | Not run live; explicit mutations use fixtures |

Summary: 14/16 API-key probe scenarios passed; 2/4 OAuth probe scenarios passed.
Blocked probes are access limitations, not successful live verification. Contract
fixtures are not a substitute for testing remote mutations on a consenting account.

## Documentation validation: v0.3.1

On 2026-10-31, the complete tool reference was checked against executable
discovery: all 23 tools and 120 parameter paths have descriptions and omission
behavior. Tables derive types, required fields, accepted values and bounds from
the schemas; defaults, results and cross-field restrictions were reviewed against
the implementation. All 23 example argument objects pass validation and reach
the expected missing-key boundary with credentials stripped and dotenv disabled.
These checks make no Google requests and run in Linux/macOS CI.

The patch changes documentation and its generation/distribution checks, with no
tool runtime changes. The feature acceptance below remains the v0.3.0 record;
billable live tests were not repeated for this documentation-only patch.

## Latest feature validation: v0.3.0

On 2026-10-31, the v0.3.0 feature/skill commit passed
[all eight CI jobs](https://github.com/fcenedes/mcp-gmap-rs/actions/runs/41198737996):
Linux/macOS tests, strict Clippy, Rust 1.88, stable/nightly docs, package auditing
and Linux amd64 container/TLS checks. The suite has 53 offline tests, 23 opt-in
API-key tests, six opt-in OAuth tests and one compile-checked documentation example.
All 29 live scenarios and all twelve `doctor --live` API probes passed in the
local acceptance runs. Both Kubernetes examples pass strict schema validation;
no cluster deployment was performed. Public Rust documentation coverage is 100%.

The detailed v0.3.0 and agent-skill records are below. Earlier sections are dated
historical snapshots, including failures subsequently resolved; their old tool
counts and preparation status do not describe the current version. Release
publication and hosted docs.rs availability are verified after the final tag.

## Historical validation: v0.1.0

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
cargo test --locked --test live live_maps_ -- --ignored --test-threads=1
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

## Layered configuration and container preparation

Validated locally on 2026-10-31 on macOS arm64 with Docker Desktop Linux arm64.
The eight configuration subprocess tests cover default-file discovery, explicit
file selection/opt-out, dotenv/environment/CLI precedence, timeout validation,
credential-safe parse errors, empty/whitespace tokens, and effective HTTP
bind/auth/host/origin settings. The existing 21 offline tests and the Rust
example also pass; live Google tests remain opt-in and were not rerun for this
change. Rust 1.88 checks all targets and features successfully.

The Dockerfile built a Linux arm64 image. Its smoke checks passed CLI discovery,
MCP stdio initialization/discovery, authenticated Streamable HTTP, mounted dotenv
with environment overrides, unauthenticated health probes and graceful SIGTERM.
These checks ran as UID/GID 10001 with a read-only root filesystem, dropped
capabilities and privilege escalation disabled. The image contains the system CA
bundle and no bundled `.env`. Build inputs are allowlisted by `.dockerignore`.

Strict stable and nightly documentation builds passed. Cargo packaging compiled
and the file audit included the Dockerfile, Kubernetes example, guide and
licenses while excluding local credentials. Kubeconform v0.8.0 validated all
three Kubernetes resources with strict schema checking. No workload was deployed
to a Kubernetes cluster. Linux amd64 container checks are configured in CI but
have not been run on GitHub for this change yet. No container image or new crate
version was published as part of these local checks.

## OAuth and Google Cloud setup preparation

Validated locally on 2026-10-31. The default offline run passes 36 tests and
ignores 22 live tests: the original 18 API-key scenarios plus four new OAuth
scenarios for Places, Routes, Weather and Air Quality. At preparation time, no
live OAuth request or service-account key creation had been performed.

Local token/API fixtures verify OAuth Authorization and quota-project headers
on all four supported API families, including Places details/photos; absence of
API keys from OAuth requests; cached token reuse; explicit key use on the current
Geocoding v3, Elevation, Time Zone and Static Maps endpoints; operation without a
key on supported endpoints; bounded token waits; billing-project errors; and
credential/token redaction. The refresh fixture uses an authorized-user ADC file
with synthetic credentials, while a malformed service-account file checks safe
failure and lazy loading. At preparation time, Google service-account JWT
exchange, real GKE metadata and external federation had not been accepted against
live infrastructure; the service-account results below supersede that boundary.

The official Google auth SDK is pinned to 1.15.0 to preserve Rust 1.88 support;
the selected dependency graph passes the minimum-version check. Stable/nightly
strict docs builds, the Rust example, Clippy, package compilation/auditing and
Linux arm64 Docker build/smoke checks pass. Container discovery also succeeds in
OAuth mode with an unavailable credential file, confirming no token is fetched
for discovery. The Cloud setup guide covers billing, all eight API service IDs,
restricted keys, service-account creation, consumer IAM permission, mounted JSON
keys, local ADC impersonation and GKE workload identity. These remain local,
changes prepared for v0.2.0.

## Live service-account OAuth acceptance

On 2026-10-31 a user-provided service-account JSON key was used with its project
as the explicit OAuth quota project. `sa_key.json` is untracked and explicitly
ignored by Git and Docker; local permissions are 0600. Cargo's package file list
excludes it. Credential contents were not printed or committed.

All four opt-in live OAuth tests passed with the API key removed from each
process:

| API | Live check | Result |
| --- | --- | --- |
| Places (New) | Text search returned a nonempty place ID | Passed |
| Routes | Walking route returned a positive distance | Passed |
| Weather | Current conditions returned a numeric temperature | Passed |
| Air Quality | Current conditions returned air quality indices | Passed |

The first run passed Air Quality; Places, Routes and Weather returned HTTP 403
identifying the missing `serviceusage.services.use` permission. After the user
granted Service Usage Consumer on the quota project, the next four-test run
passed Air Quality and Routes. Places and Weather still returned the permission
error, then passed individual retries after a 45-second wait, consistent with
IAM propagation. These results confirm real service-account OAuth token exchange
and access to all four API families in the tested project; no runtime code fix
was needed.

The checks cover small Paris-based scenarios. Live service-account token expiry
and refresh over an hour, GKE metadata authentication and external federation
remain untested against live infrastructure. Cached token reuse and refresh
request handling remain covered by the local token/API fixtures.

## v0.3.0 production features and additional APIs

Validated on 2026-10-31. The offline suite passes 53 tests, with 29 live tests
ignored by default, plus one compile-checked Rust documentation example. The
five added tools have local request/response fixtures and live API-key checks.

All 23 API-key tool scenarios passed. The first full run passed 22 and received
an explicit API-disabled 403 for Address Validation. After the user enabled that
service in the key's project, the focused Address Validation rerun passed.
All six service-account OAuth tests passed with the API key removed, including
the added Address Validation and Places Aggregate scenarios. `doctor --live`
also passed representative probes for all twelve Google API families in use.
No real credential values were printed, packaged or committed.

Local tests verify TLS/mTLS handshakes, rejection of untrusted or missing client
certificates, bearer authentication, native HTTPS MCP initialization, protected
metrics, request limits, SIGHUP certificate replacement, failed-reload retention
and graceful SIGTERM. Fixtures cover API request shape, OAuth versus key routing,
parallel per-invocation budgets, retry/backoff/Retry-After handling, overall
deadlines, metrics, diagnostic redaction and offline/live doctor behavior.

Strict Clippy, formatting, Rust 1.88 checks, stable/nightly rustdoc, workflow lint,
Cargo package compilation and package/credential auditing passed locally. Public
Rust documentation coverage is 100% for both items and eligible examples. An
LLVM coverage run reports the following line coverage for the new modules:

| Module | Covered lines | Percentage |
| --- | --- | --- |
| Additional Google tools | 113/113 | 100.00% |
| Doctor | 109/122 | 89.34% |
| Metrics | 64/68 | 94.12% |
| Limits and budgets | 55/57 | 96.49% |
| Native TLS | 41/46 | 89.13% |

Coverage is an offline-test measurement, not a claim of complete branch coverage.
The package contains the production guide and both deployment examples while
excluding `.env`, `sa_key.json`, TLS keys and generated output. The service-account
file remains ignored, untracked and mode 0600.

Kubeconform strictly validates all three base Kubernetes resources and both
Gateway API resources without skipped schemas. No Kubernetes workload or gateway
was deployed. Local Docker Desktop was unresponsive. The isolated Linux amd64
[CI container job](https://github.com/fcenedes/mcp-gmap-rs/actions/runs/41197569281/job/124526718430)
built the image and passed native TLS, protected metrics, rate limits, doctor,
SIGHUP and SIGTERM checks in addition to the existing CLI/stdio/HTTP cases.
The first CI run also passed Linux/macOS tests, both documentation builds, package
verification and Rust 1.88 checks. Its Rust 1.99 Clippy deprecation was corrected
using a compatible compare-and-exchange loop; strict local Clippy on 1.99 passes.
No v0.3.0 release or container-registry publication was made during preparation.

## Agent skill packaging

The `skills/mcp-gmap/` skill and its installed local copy pass the skill-creator
frontmatter/structure validator. Fifteen discovery, MCP-argument and CLI examples
were exercised against the executable with dotenv loading disabled, Google
credentials removed and API-key mode explicitly selected. Discovery succeeded;
data calls reached the expected missing-key boundary after argument validation.
No live Google requests were made for this skill check.

Cargo packaging compiled successfully and includes all four skill files. The
release packager was exercised in a temporary source fixture using the current
debug executable: version/discovery checks passed, the archive contained identical
skill files, and local credentials were excluded. That fixture was removed; it
was not a published release binary. Strict rustdoc also passes with the skill
source link and installation guidance.
