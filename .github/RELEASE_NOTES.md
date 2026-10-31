Service-account OAuth, layered configuration and container deployment for the
standalone Rust Google Maps CLI and MCP server.

- Add OAuth through Google's Rust auth library, supporting service-account JSON
  files, Application Default Credentials and workload identity, with cached token
  refresh and an explicit quota project.
- Use OAuth for Places (New), Routes, Weather and Air Quality. Current Geocoding
  v3, Elevation, Time Zone and Static Maps endpoints still use an API key; mixed
  workflows can use both credential types. OAuth failures never downgrade to a key.
- Load configuration in order: defaults → optional `.env` → environment variables
  → CLI flags. Add `--env-file`, `--no-env-file`, `--auth-mode` and environment-based
  HTTP bind/host/origin settings.
- Add a non-root multi-stage Dockerfile and Kubernetes deployment example, plus
  container smoke checks and Kubernetes schema validation in CI.
- Document Google project billing, API enablement, restricted key creation,
  service-account IAM permissions, JSON credentials, ADC and GKE workload identity.

Download the archive for your platform and verify it against `SHA256SUMS`:

- Linux x86_64: `x86_64-unknown-linux-gnu` (glibc 2.35 or newer).
- macOS Apple Silicon: `aarch64-apple-darwin`.
- macOS Intel: `x86_64-apple-darwin`.

Archives contain the executable, README, changelog, configuration example,
documentation and licenses. See the bundled README and Google Cloud setup guide
for API-key or OAuth configuration. Discovery and help work without credentials.
The source archive and Cargo package include the Dockerfile and Kubernetes example.

The macOS binaries are not notarized or code-signed.

Once the crate has been published to crates.io, installation from source is:

```sh
cargo install mcp-gmap-rs --locked
```

All 18 tools passed earlier API-key live acceptance scenarios. This release's
four live OAuth tests passed against Places, Routes, Weather and Air Quality with
the API key removed. Offline validation covers 36 tests plus a documentation
example. Automated release checks do not require Google credentials.
