# Production operations

This guide covers v0.3.x. All Google tools share
the same implementation across CLI, MCP stdio and Streamable HTTP.

## Native TLS and mutual TLS

The HTTP listener can serve HTTPS directly using rustls and TLS 1.2/1.3:

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3443 \
  --tls-cert /etc/mcp-gmap/tls/tls.crt \
  --tls-key /etc/mcp-gmap/tls/tls.key \
  --allowed-host maps.example.com:3443
```

Supply `MCP_HTTP_TOKEN` separately through the environment or a Secret. Clients
connect to `https://maps.example.com:3443/mcp`, verify the server certificate,
and send `Authorization: Bearer <token>`. Certificate files contain a PEM leaf
certificate followed by its intermediate chain; the private key must match.
The executable does not issue certificates or implement ACME. Use your PKI,
cert-manager, or a TLS gateway for issuance and renewal.

Add `--tls-client-ca /etc/mcp-gmap/tls/client-ca.pem` to require a client
certificate signed by that CA. Native mTLS applies to every path, including
`/health` and `/metrics`, and does not replace the HTTP bearer token. Any valid
client identity under the configured trust roots is accepted at the TLS layer;
there are no per-certificate tool permissions or revocation-list checks.

On Linux/macOS, replace the files atomically, then send `SIGHUP` to reload the
certificate, key and optional client CA together. Existing connections keep their
TLS state; new handshakes use the replacement. If loading fails, the previous
configuration stays active and an error is logged without printing key material.
Mounted Kubernetes Secret files can be reloaded this way after their projection
updates. Other settings still require a process restart.

For UID/GID 10001 in the container, mount keys read-only with permissions that
allow that identity to read them; use a suitable group and `fsGroup` for projected
Secrets. With native TLS, Kubernetes HTTP probes must use `scheme: HTTPS`.
Kubelet HTTP probes cannot provide an mTLS client certificate; use a TCP probe
for that deployment, understanding that it checks the listener, not TLS or Google.

## Kubernetes TLS gateway

[the Gateway manifest](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/gateway.yaml) provides an HTTPS Gateway and HTTPRoute for the base
Deployment in [the base Kubernetes manifest](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml). Install the Gateway API CRDs and a
conformant controller first. Replace `your-gateway-class` and `maps.example.com`,
and provision a `kubernetes.io/tls` Secret named `mcp-gmap-tls` in the same namespace.
The example uses stable Gateway/HTTPRoute v1 fields and HTTPRoute timeouts
supported since Gateway API v1.2. Check your controller's timeout support.

```sh
kubectl create secret tls mcp-gmap-tls --cert=/secure/fullchain.pem --key=/secure/key.pem
kubectl set env deployment/mcp-gmap-rs \
  MCP_HTTP_ALLOWED_HOSTS='maps.example.com,mcp-gmap-rs:3000,localhost:3000,127.0.0.1:3000'
kubectl apply -f deploy/gateway.yaml
kubectl get gateway,httproute
```

Persist the corresponding allowed-host setting in the ConfigMap or deployment
source too. Add actual browser origins with `MCP_HTTP_ALLOWED_ORIGINS` when needed.
The route exposes only `/mcp`; health and metrics stay on the private Service.
Authorization and MCP session/protocol headers must pass through unchanged.
Disable response buffering/caching and configure streaming/idle timeouts in the
chosen controller. The route disables request/backend-request deadlines; configure
appropriate connection/idle limits at the gateway for your clients.

This configuration terminates TLS at the gateway and uses HTTP to the ClusterIP
backend. Use native TLS plus a controller-supported BackendTLSPolicy or a mesh if
the gateway-to-pod connection must also be encrypted. Restrict direct pod/Service
access with your cluster's network policy. TLS does not provide MCP OAuth login:
the executable uses a shared bearer token and one Google identity per process.

See the upstream [Gateway TLS guide](https://gateway-api.sigs.k8s.io/guides/user-guides/tls/)
and [HTTPRoute timeout guide](https://gateway-api.sigs.k8s.io/guides/http-timeouts/).

## Limits, retries and deadlines

| Variable | Default | Flag | Meaning |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | Google HTTP attempts per tool invocation, 1–10000; includes retries |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | Additional attempts per Google request, 0–5 |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | Overall tool deadline in seconds, 1–3600 |
| `MCP_HTTP_RATE_LIMIT` | `120` | HTTP `--rate-limit` | Authenticated `/mcp` HTTP requests per minute per process; 0 disables |
| `MCP_METRICS_ENABLED` | `false` | HTTP `--metrics [true\|false]` | Enable protected Prometheus `/metrics` |
| `MCP_LOG_FORMAT` | `text` | `--log-format text\|json` | Diagnostic event format on stderr |
| `MCP_TLS_CERT` | unset | HTTP `--tls-cert` | PEM server certificate chain file |
| `MCP_TLS_KEY` | unset | HTTP `--tls-key` | Matching PEM server private key file |
| `MCP_TLS_CLIENT_CA` | unset | HTTP `--tls-client-ca` | PEM CA bundle; require client certificates |

These settings follow defaults → optional dotenv → environment → flags. TLS
certificate and key must be supplied together. Empty environment TLS paths clear
their file values. `GOOGLE_MAPS_TIMEOUT` bounds each Google HTTP request including
semaphore queueing, retries, backoff and response reading; OAuth acquisition has
its own bound using the same timeout. The overall tool deadline includes both.

Each invocation has a fresh request budget shared across all its concurrent
branches. Five Google API requests can execute concurrently per process. Once
the budget is exhausted, further attempts fail locally. Composites may return
successful output containing per-item budget/errors; inspect those fields. This
limits request count, not currency: different Google requests/fields have different
prices, and a matrix request can bill multiple elements.

Read-only Google requests retry HTTP 429, 500, 502, 503 and 504 and network failures.
Retries honor `Retry-After` seconds or HTTP dates, otherwise use exponential
backoff with jitter. They stay within the request deadline and invocation budget.
Other HTTP errors, including 400/401/403, are not retried. No authentication
failure silently switches to API-key mode. Set retries to zero to make a single
attempt. Google-internal legacy JSON error statuses are reported without retries.

The HTTP limiter is a token bucket with a burst capacity equal to its per-minute
rate and continuous refill. It covers initialization, POST, GET and DELETE on
`/mcp`, across all sessions. Unauthorized requests do not consume the bucket.
Rejections return HTTP 429 with `Retry-After`. Health and metric scrapes are exempt.
The limiter is global to one process, not per user/IP; it does not trust forwarded
IP headers. Use gateway authentication and per-client limits for shared deployments.

## Metrics and diagnostic events

```sh
mcp-gmap-rs --log-format json serve http --metrics --rate-limit 120
```

`GET /metrics` uses the same bearer token as `/mcp` when configured. On an
unauthenticated loopback listener it is also unauthenticated. It is disabled by
default, and native mTLS applies when enabled. Expose it only to trusted scrapers.
The Prometheus text endpoint exports:

- `mcp_gmap_tool_calls_total{tool,outcome}` for completed tool calls;
- `mcp_gmap_tool_duration_seconds{tool,outcome}` as a histogram;
- `mcp_gmap_google_requests_total{api,status}` for Google HTTP attempts, including retries;
- `mcp_gmap_http_rate_limited_total` for rejected HTTP requests.

Labels contain built-in tool names, API hostnames, HTTP status codes and fixed
outcomes. They never contain addresses, coordinates, credentials, tokens or session
IDs. A composite with per-item errors may count as a successful top-level call;
inspect its result and the upstream status metrics. Metrics reset on restart.

Set `RUST_LOG=info` to see tool completion events with name, success and elapsed
time. Arguments and Google response bodies are not included. The listener's startup
address is printed separately on stderr. Stdout remains CLI JSON or MCP traffic.

## Doctor

```sh
mcp-gmap-rs doctor
# Explicit opt-in: makes potentially billable requests.
mcp-gmap-rs doctor --live
# Check only the API dependencies of selected tools:
mcp-gmap-rs --enabled-tools maps_pollen,maps_area_insights doctor --live
```

Without `--live`, doctor checks local credential configuration, HTTP settings and
TLS file/key matching without making network requests. It does not prove key
validity, IAM, API enablement, certificate trust/hostname/expiry or ADC access.
ADC discovery is reported as `not_tested` until a live request. Doctor never
prints credential values or their JSON. A JSON report goes to stdout; a failed
check produces exit status 1. Parse `checks[].status`: `pass`, `fail` or `not_tested`.

Live mode makes one representative tool invocation for each distinct Google API
used by enabled tools, with normal retry/deadline/budget settings. It avoids running
expensive composites. Routes use exact coordinates. Roads uses nearest-road lookup.
Probes use public Paris landmarks, not user-provided addresses. Success means Google
accepted that probe, not that every operation or geographic region is available.


## Shutdown and configuration changes

Ctrl-C and SIGTERM shut down the HTTP server and its active MCP sessions. The
Kubernetes example gives the process 30 seconds. Other application configuration,
including environment-injected Secret changes, requires a restart; only native TLS
certificate/key/client-CA files support SIGHUP reload. Sessions and metrics are
in memory and are lost on restart. Run a single replica or configure session
affinity; clients must reconnect when their owning process exits.

See [Deployment](./deployment) for container permissions, probes and runtime
credential mounts, and [Configuration reference](../reference/configuration) for
every setting and flag.
