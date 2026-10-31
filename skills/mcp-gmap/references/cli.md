# CLI invocation and access diagnostics

Use the executable on PATH or the path supplied by the user. These commands do
not require a source checkout. Discovery/help make no Google requests:

```sh
mcp-gmap-rs --version
mcp-gmap-rs --help
mcp-gmap-rs tools
mcp-gmap-rs schema search-nearby
```

`tools` reports enabled tools and their input schemas. Inspect this output instead
of assuming that the newest source release is installed. If a malformed local
dotenv file blocks discovery, `mcp-gmap-rs --no-env-file tools` can inspect the
environment-only selection; it may differ from the intended configured selection.

## Make structured calls

The generic `call` interface uses the same argument names as MCP and avoids
guessing the CLI's flag spelling. Pass JSON as a distinct process argument when
using a subprocess API. For shell use, single-quote literal JSON or send
programmatically serialized JSON on stdin; do not interpolate user text into a
shell command. Both logical MCP names and CLI command names are accepted.

```sh
mcp-gmap-rs call maps_search_places --args '{"query":"coffee near Gare de Lyon, Paris","openNow":true,"minRating":4}'
mcp-gmap-rs directions --origin '48.8584,2.2945' --destination '48.8606,2.3376' --mode walking
mcp-gmap-rs search-nearby --center '{"value":"48.8584,2.2945","isCoordinates":true}' --keyword cafe --radius 500
mcp-gmap-rs weather --latitude 48.8584 --longitude 2.2945 --type forecast_daily --forecast-days 3
mcp-gmap-rs static-map --center '48.8584,2.2945' --zoom 14 --output area.png
```

Object/array flags take JSON. Boolean flags accept `true`/`false`; a bare boolean
flag means `true`. Inspect `COMMAND --help` for available flags. `--output` writes
JSON for ordinary tools and raw PNG for `static-map`.

CLI success writes JSON to stdout. Errors go to stderr with a nonzero exit code.
Batch/composite JSON can contain per-item errors even on exit 0. `doctor` is a
special case: it prints its JSON report on stdout even when checks fail.

## Use the configured identity

Configuration precedence is **defaults → working-directory `.env` → process
environment → CLI flags**. Parent directories are not searched for dotenv files.
Select a mounted file with `--env-file PATH` or `MCP_ENV_FILE`; disable file
loading with `--no-env-file`. Do not dump environment variables or credential files
to discover whether a key is present. Relative credential paths are relative to
the process's working directory.

| Purpose | Configuration |
| --- | --- |
| Google API key | `GOOGLE_MAPS_API_KEY`; default auth mode is `api-key` |
| Google OAuth | `GOOGLE_MAPS_AUTH_MODE=oauth`, `GOOGLE_CLOUD_QUOTA_PROJECT`, and an optional `GOOGLE_APPLICATION_CREDENTIALS` JSON path; otherwise ADC discovery |
| MCP client authentication | Configured by the MCP host; remote clients send the server's `MCP_HTTP_TOKEN` as a bearer token |

In this implementation OAuth covers Places (New), Routes, Weather and Air Quality,
plus Address Validation and Places Aggregate when available. Geocoding v3,
Elevation, Time Zone, Static Maps, Pollen and Roads use the API key. Named-location
resolution and some composite flows can therefore require both credential types.
Do not infer that every enabled Google API supports this executable's OAuth path.
OAuth failures never silently fall back to a key.

Credentials are supplied out of band by the user or deployment. Google service
accounts need permission to consume services in the quota project; enabled APIs,
key API restrictions and public egress-IP restrictions are separate checks.

## Diagnose only what the task needs

On versions that provide it:

```sh
mcp-gmap-rs doctor
```

Default doctor is offline. Read `checks[].status` (`pass`, `fail`, `not_tested`)
and the command's exit status. It checks local configuration, not live API access.
ADC discovery may remain `not_tested`; `/health` is only process liveness.

Use `doctor --live` only when live access diagnosis is in scope: it issues
potentially billable probes. Scope it to relevant tools rather than probing every
API for a single failed operation:

```sh
mcp-gmap-rs --enabled-tools maps_pollen doctor --live
```

Distinguish an MCP HTTP 401 (client bearer token) from a Google 401/403 (outbound
credentials, enabled services, restrictions or IAM). After the user changes the
configuration, rerun the affected call; IAM/API changes may need propagation.

On v0.3.0+, `--max-requests`, `--max-retries` and `--tool-timeout` control Google
attempt budgets and tool deadlines. The default HTTP limiter is process-wide,
not per client. Budget exhaustion in a composite can appear as per-item errors.
Do not raise these limits to hide repeated failures. Use the installed `--help`
and the project's production guide for configuration work explicitly requested
by the user.

Invoking the executable without a subcommand starts the MCP stdio server, so
always give a subcommand for a one-shot query. A configured MCP host can instead
launch `mcp-gmap-rs serve stdio`; remote hosts connect to the configured `/mcp`
Streamable HTTP endpoint. Check server TLS certificates and preserve configured
host/origin restrictions. Native mTLS, where configured, also requires a client
certificate; it does not replace the MCP bearer token.
