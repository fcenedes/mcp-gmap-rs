# Getting started

Version 0.4.0 provides 42 tools through one executable, available from the terminal or an MCP client. This manual documents installation, Google access and the complete tool interface.

Build from the repository checkout:

```sh
cargo install --path . --locked
```

## Install

Prebuilt Linux x86_64, macOS Apple Silicon and macOS Intel binaries are available in [GitHub Releases](https://github.com/fcenedes/mcp-gmap-rs/releases). Download the archive for your machine, verify it against `SHA256SUMS`, extract it and put `mcp-gmap-rs` on your `PATH`. Linux binaries target glibc 2.35 or newer. macOS binaries are not signed or notarized.

With Rust 1.88 or newer, install from crates.io:

```sh
cargo install mcp-gmap-rs --version 0.4.0 --locked
mcp-gmap-rs --version
```

No Rust, Python or JavaScript runtime is needed after the executable is built. The host needs CA certificates, DNS and outbound HTTPS access to Google.

## Configure access

[Create a restricted key and enable services](./google-cloud), or configure [service-account OAuth](./authentication). Store credentials in an ignored `.env` in your working directory:

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

Environment variables override the file. Keep credentials outside source control and image layers. The working directory matters: parent directories are not searched for `.env`.

## Discover, check, call

Discovery and offline diagnostics do not make Google requests:

```sh
mcp-gmap-rs tools
mcp-gmap-rs schema search-places
mcp-gmap-rs doctor
```

Then make your first potentially billable call:

```sh
mcp-gmap-rs search-places --query 'coffee near Gare de Lyon, Paris'
```

The result is JSON. Keep the returned place IDs for follow-up details rather than resolving the same names repeatedly. Read [CLI usage](./cli) for automation or [MCP integration](./mcp) to connect an agent. [Recipes](./recipes) show how to combine tools; the [complete reference](../reference/tools) documents every parameter, default and result.
