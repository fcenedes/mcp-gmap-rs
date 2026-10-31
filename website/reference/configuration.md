# Configuration reference

These 21 application settings apply to CLI, MCP stdio and Streamable HTTP as appropriate. Values follow **defaults → optional dotenv → environment → flags**. See [configuration](../guide/configuration) for file selection and override semantics.

Global flags go before a tool or `serve`; HTTP-only flags go after `serve http`. A dash in the flag column means there is no corresponding CLI flag: inject that value through the environment or selected dotenv file.

## Google access and tool execution

| Environment variable | Default | CLI flag | Meaning and accepted values |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_API_KEY` | Unset | — | Restricted Google API key. Required in `api-key` mode and for this implementation’s key-only endpoints, including when OAuth mode is selected. Never a tool argument. |
| `GOOGLE_MAPS_AUTH_MODE` | `api-key` | `--auth-mode` | `api-key` or `oauth`. OAuth supports Places (New), Routes, Weather, Air Quality, Address Validation and Places Aggregate. |
| `GOOGLE_APPLICATION_CREDENTIALS` | ADC discovery | — | Readable service-account or supported ADC JSON file path. Relative paths use the working directory. Leave unset for local ADC or metadata discovery. |
| `GOOGLE_CLOUD_QUOTA_PROJECT` | Unset | — | Required consumer billing/quota project ID or number for OAuth calls. The service account needs `serviceusage.services.use` in that project. |
| `GOOGLE_MAPS_LANGUAGE` | `en` | `--language` | Language code for language-aware Google responses. Independent of this manual’s language selector. |
| `GOOGLE_MAPS_TIMEOUT` | `30` | `--timeout` | Per-request timeout in seconds, integer 1–600. Includes queueing, retries, backoff and response reads; OAuth acquisition has its own bound using the same value. |
| `GOOGLE_MAPS_ENABLED_TOOLS` | All (`*`) | `--enabled-tools` | Comma-separated `maps_*` names or `*`. Unknown names fail startup. Disabled tools are absent from discovery and cannot be called. |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | Google HTTP attempt budget per tool invocation, integer 1–10000. Shared across composite branches; retries consume attempts. |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | Additional transient attempts per Google request, integer 0–5. Zero means one initial attempt. Retries remain within timeout and budget. |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | Overall tool deadline in seconds, integer 1–3600. Includes credential acquisition, queueing, API calls and retries. |

Five Google HTTP requests can execute concurrently per process. The attempt budget limits requests rather than currency; one matrix request can bill multiple elements. Some composites return per-item errors when a branch exhausts its budget or deadline. See [production limits](../guide/production#limits-retries-and-deadlines).

## HTTP listener and access

| Environment variable | Default | HTTP flag | Meaning and accepted values |
| --- | --- | --- | --- |
| `MCP_HTTP_TOKEN` | Unset | — | Shared bearer token for MCP clients. Required for non-loopback binds. Separate from Google credentials and still required with native mTLS. |
| `MCP_HTTP_LISTEN` | `127.0.0.1:3000` | `--listen` | Listener IP address and port. The Docker image sets the environment value to `0.0.0.0:3000`. |
| `MCP_HTTP_ALLOWED_HOSTS` | Loopback hosts | `--allowed-host` | Comma-separated accepted HTTP Host values. Repeat the flag for several values; flags replace the whole file/environment list. Include the relevant port. |
| `MCP_HTTP_ALLOWED_ORIGINS` | None | `--allowed-origin` | Comma-separated allowed browser origins. Present Origin headers are rejected unless allowed. Specify ports explicitly; repeated flags replace the list. |
| `MCP_HTTP_RATE_LIMIT` | `120` | `--rate-limit` | Authenticated `/mcp` requests per minute per process, integer 0–1000000. Zero disables. Burst capacity equals the rate; continuous refill. |
| `MCP_METRICS_ENABLED` | `false` | `--metrics [true\|false]` | Enable Prometheus `/metrics`. A bare `--metrics` means true. Uses the same bearer credential when configured; native mTLS also applies. |
| `MCP_TLS_CERT` | Unset | `--tls-cert` | PEM server leaf certificate plus intermediate chain. Must be supplied together with a matching private key. |
| `MCP_TLS_KEY` | Unset | `--tls-key` | PEM server private key path. Keep readable by the runtime identity and out of source/images. |
| `MCP_TLS_CLIENT_CA` | Unset | `--tls-client-ca` | PEM client CA bundle enabling mandatory client certificates on every listener path. Does not replace the bearer token. |

The HTTP token bucket is global to the process, not per user/IP. Unauthorized requests do not consume it; health and metrics are exempt. Rejections return 429 with `Retry-After`. The request-body limit is 12 MiB. See [MCP access](../guide/mcp#bind-addresses-and-access) and [TLS operations](../guide/production#native-tls-and-mutual-tls).

## Diagnostic events

| Environment variable | Default | CLI flag | Meaning and accepted values |
| --- | --- | --- | --- |
| `RUST_LOG` | `warn` | — | Diagnostic log filter. `info` enables tool completion events; arguments and Google response bodies are not logged. |
| `MCP_LOG_FORMAT` | `text` | `--log-format` | `text` or `json`. Diagnostics always go to stderr, leaving stdout for CLI JSON or MCP traffic. |

## File selection and output controls

File selection happens before the application layers above. `MCP_ENV_FILE` chooses a dotenv path; global `--env-file PATH` takes precedence. An explicitly selected file must exist. `--no-env-file` disables all dotenv loading and conflicts with `--env-file`. Without a selection, only the working directory’s optional `.env` is read.

An explicitly empty environment value still overrides a file value. It can clear an optional token, TLS path or allowlist; it does not make an empty numeric setting valid. Loaded file values do not mutate the process environment.

`doctor --live` opts into potentially billable probes; ordinary `doctor` is offline. Tool `--output PATH` saves JSON, or raw PNG for `static-map`. These are command controls, not additional environment settings.

Application settings and credential-file changes require a restart. On Linux/macOS, SIGHUP can reload native TLS certificate/key/client-CA files together; failed reloads preserve the previous TLS configuration. See [configuration](../guide/configuration) and [production operations](../guide/production).
