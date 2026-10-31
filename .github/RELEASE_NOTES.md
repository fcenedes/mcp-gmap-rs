Complete parameter documentation for all 23 Google Maps tools, available in the
bundled `docs/tools.md` guide and the docs.rs landing page.

- Describe each tool’s purpose, parameters and nested object fields, types,
  required status, defaults, accepted values, limits and behavioral restrictions.
- Document response shapes, partial-error handling, authentication dependencies,
  timestamps, units and result/coverage limitations.
- Provide MCP argument examples and equivalent CLI calls for every tool.
- Generate parameter tables from the executable’s actual schemas and reviewed
  behavior notes. CI checks for stale output and validates examples without
  credentials or Google requests.

Tool behavior is unchanged from v0.3.0. TLS/mTLS, request controls, metrics,
service-account OAuth and the bundled agent skill remain available.

Archives include Linux x86_64 or macOS Apple Silicon/Intel executables,
documentation, deployment examples, the agent skill, configuration example and
licenses. Verify downloads against SHA256SUMS. macOS binaries are not notarized
or code-signed.

```sh
cargo install mcp-gmap-rs --version 0.3.1 --locked
```
