# Changelog

## 0.3.0 - 2026-10-31

- Include a reusable agent skill for MCP/CLI usage, with capability discovery, tool selection, credential boundaries and partial-result handling.
- Add Address Validation, Pollen, Places Aggregate, Snap to Roads and Nearest Roads tools across CLI and both MCP transports (23 tools total).
- Extend service-account/ADC OAuth to Address Validation and Places Aggregate.
- Serve native TLS 1.2/1.3 with optional client certificates and atomic SIGHUP certificate/CA reload; retain the last good configuration on failure.
- Add per-invocation Google request budgets, overall tool deadlines and bounded transient retries with Retry-After/backoff handling.
- Add a process-wide authenticated HTTP request rate limit, opt-in protected Prometheus metrics and structured JSON diagnostic events.
- Add offline `doctor` diagnostics and explicitly opted-in live API probes.
- Document production settings and provide an optional Kubernetes HTTPS Gateway/HTTPRoute example, with schema validation and TLS/container smoke checks in CI.

## 0.2.0 - 2026-10-31

- Support service-account/ADC OAuth for Places (New), Routes, Weather and Air Quality, with key authentication retained for current legacy endpoints.
- Document Google project billing, API enablement, restricted API keys, service-account JSON keys and keyless Kubernetes/GKE authentication.
- Load an optional dotenv file with environment-variable and CLI overrides using config-rs.
- Add explicit file selection, file-loading opt-out and environment-based HTTP settings.
- Add a non-root Docker image, container smoke checks and a Kubernetes deployment example.

## 0.1.1 - 2026-10-30

- Add a docs.rs landing guide covering all 18 tools and the public Rust entry point.
- Configure all-feature Linux documentation builds and compile-checked Rust examples.
- Enforce strict stable/nightly rustdoc checks in CI and verify docs before crate publication.
- Update base64 to 0.23.1.

## 0.1.0 - 2026-10-30

- Standalone Rust CLI and MCP server with stdio and Streamable HTTP transports.
- Eighteen Google Maps tools for places, geocoding, routing, elevation, timezone,
  weather, air quality, static maps, batch operations and local rank tracking.
- Shared input validation, bounded HTTP concurrency, request timeouts, safe
  credential handling and Google attribution metadata.
- Offline protocol/API fixtures and an opt-in live acceptance suite.
- CI for Linux and macOS, Rust 1.88 compatibility checks, and verified
  Cargo packaging.
- Release archives for Linux x86_64 and macOS Apple Silicon / Intel,
  with SHA-256 checksums.
