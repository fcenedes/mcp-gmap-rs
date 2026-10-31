Native HTTPS, production controls and five additional Google Maps tools for the
standalone Rust CLI and MCP server.

- Add `maps_validate_address`, `maps_pollen`, `maps_area_insights`,
  `maps_snap_to_roads` and `maps_nearest_roads`, available through CLI, MCP stdio
  and Streamable HTTP. There are now 23 tools.
- Extend Google service-account/ADC OAuth to Address Validation and Places
  Aggregate. Pollen and Roads use this executable's configured API key.
- Add native TLS 1.2/1.3, optional client-certificate authentication, and atomic
  SIGHUP reload of certificates, keys and client CAs.
- Bound Google HTTP attempts per invocation, tool wall time and transient retries.
  HTTP rate limiting returns 429 with Retry-After; Prometheus metrics are opt-in
  and protected by the HTTP bearer token when configured.
- Add JSON diagnostic events and offline `doctor` checks. `doctor --live` makes
  explicitly opted-in billable API probes.
- Include production configuration documentation and a Kubernetes TLS Gateway
  example; Google service-account OAuth remains separate from inbound MCP auth.
- Bundle the reusable `mcp-gmap` agent skill with MCP/CLI usage guidance and
  installation instructions; no assistant plugin is required.

The default Google request budget is 200 attempts per tool call, including retries.
The default overall tool deadline is 120 seconds, with up to two transient retries
within the existing 30-second Google request timeout. The default HTTP rate is
120 authenticated MCP requests per minute per process. See the bundled production
guide for overrides, metrics, TLS/mTLS and certificate renewal.

Download the archive for your platform and verify it against `SHA256SUMS`:

- Linux x86_64: `x86_64-unknown-linux-gnu` (glibc 2.35 or newer).
- macOS Apple Silicon: `aarch64-apple-darwin`.
- macOS Intel: `x86_64-apple-darwin`.

Archives contain the executable, README, changelog, configuration example,
documentation, deployment examples, agent skill and licenses. The source archive
and Cargo package also include the Dockerfile. Discovery/help require no Google credentials.
The macOS binaries are not notarized or code-signed.

Install this version from crates.io:

```sh
cargo install mcp-gmap-rs --version 0.3.0 --locked
```

All 23 tools passed live API-key checks. Six service-account OAuth scenarios
passed with the API key removed. `doctor --live` passed probes of all 12 Google
API families used by this version. Ordinary CI uses local API/token fixtures and
synthetic TLS credentials; it makes no live Google calls.
