//! Google Maps tools shared by a standalone CLI and both standard MCP transports.
#![deny(missing_docs)]

mod catalog;
mod cli;
mod client;
mod composite;
mod routes;
mod server;
mod tools;

#[cfg(test)]
mod tests;

/// Parse command-line arguments and run a tool or MCP server.
///
/// With no arguments, starts the stdio MCP server. Diagnostics go to stderr.
/// Returns an error for invalid configuration, API failures, or transport failures.
pub async fn run() -> anyhow::Result<()> {
    cli::run().await
}
