# Development and releases

The runtime is a standalone Rust project. CLI, MCP stdio and Streamable HTTP share the same tool implementations. The documentation website is separate authoring/build tooling; end users need only the compiled executable.

## Build and check

Clone [the repository](https://github.com/fcenedes/mcp-gmap-rs), then use its lockfile:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
cargo test --doc --all-features --locked
cargo build --release --locked
```

Rust 1.88 is the declared minimum; release binaries use Rust 1.94.1. Default tests use local Google/token fixtures and require no credentials or network access to Google. CI also checks Linux/macOS, minimum Rust, stable/nightly rustdoc, source packaging, Docker/MCP/TLS smoke scenarios and Kubernetes manifests.

After local verification, `cargo clean` reclaims generated build output; subsequent builds recompile it.

## Keep the reference complete

The canonical tool guide combines actual executable schemas with reviewed defaults, results and behavioral notes:

```sh
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs --check --verify-examples
```

The checker strips credentials, disables dotenv loading and validates all examples at the missing-key boundary without contacting Google. Update the reviewed JSON notes when parameters or behavior change.

Live tests are opt-in and potentially billable. With an explicitly configured test key, run `cargo test --locked --test live live_maps_ -- --ignored --test-threads=1`. The six OAuth tests use the `live_oauth_` filter and remove the key from child processes. Never commit credentials or enable live tests in ordinary CI.

## Publish a version

Update the Cargo version, lockfile, changelog, install examples and release notes together. Push and wait for CI, then create an annotated matching `vX.Y.Z` tag. The Release workflow checks the exact tag, builds three native targets, verifies archives/checksums and publishes the staged draft.

The separate **Publish crate** workflow requires that published GitHub release. Dispatch it for the same tag. Store its scoped `CARGO_REGISTRY_TOKEN` in the `crates-io` GitHub environment, never in this repository. docs.rs builds automatically after crates.io publication; a GitHub release alone does not update hosted Rust documentation. Verify the actual hosted build after uploading.

See the [release procedures](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/releases.md), [validation record](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/validation.md) and [public Rust entry point](https://docs.rs/mcp-gmap-rs/latest/mcp_gmap_rs/fn.run.html).
