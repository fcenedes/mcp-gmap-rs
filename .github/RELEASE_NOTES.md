Documentation and packaging update for the standalone Rust CLI and MCP server.

- Add a docs.rs guide covering all 18 tools, CLI use, MCP transports and the public Rust entry point.
- Configure all-feature Linux documentation builds and compile-checked examples.
- Enforce stable/nightly documentation checks and publication preflight checks.
- Update base64 to 0.23.1.

Download the archive for your platform and verify it against `SHA256SUMS`:

- Linux x86_64: `x86_64-unknown-linux-gnu` (glibc 2.35 or newer).
- macOS Apple Silicon: `aarch64-apple-darwin`.
- macOS Intel: `x86_64-apple-darwin`.

Archives contain the executable, README, changelog, configuration example,
documentation and licenses. Export `GOOGLE_MAPS_API_KEY` to call Google APIs;
discovery and `--help` work without a key. See the bundled README for setup.

The macOS binaries are not notarized or code-signed.

Once the crate has been published to crates.io, installation from source is:

```sh
cargo install mcp-gmap-rs --locked
```

All 18 tools passed local live acceptance scenarios. Automated release checks
use offline tests and do not require Google credentials.
