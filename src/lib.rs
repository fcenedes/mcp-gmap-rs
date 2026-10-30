#![doc = include_str!("../docs/crate.md")]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]

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
/// This is the executable's entry point. It reads the process's arguments and
/// environment; with no arguments, it starts the MCP stdio server. Run it inside
/// a Tokio runtime. See the [crate guide](crate) for commands and configuration.
///
/// Diagnostics go to stderr. Stdout carries CLI JSON or MCP messages, depending
/// on the selected command. Argument parsing can exit the process for help,
/// version information, or invalid command-line syntax.
///
/// # Errors
///
/// Returns an error for invalid configuration, Google API failures, output-file
/// failures, or MCP transport failures. Batch and composite commands can return
/// successful output containing per-item errors; inspect their result fields.
///
/// # Example
///
/// This example is compiled without starting a server or calling Google:
///
/// ```no_run
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     mcp_gmap_rs::run().await
/// }
/// ```
pub async fn run() -> anyhow::Result<()> {
    cli::run().await
}
