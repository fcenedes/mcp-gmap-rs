use std::{
    io::{self, Read, Write},
    time::Duration,
};

use anyhow::Context;
use clap::{Arg, ArgAction, Command};
use serde_json::{Value, json};
use tracing_subscriber::EnvFilter;

use crate::{
    catalog::Catalog,
    client::MapsClient,
    server::{self, HttpOptions},
    tools::{EngineState, Output},
};

pub(crate) async fn run() -> anyhow::Result<()> {
    let catalog = Catalog::new(None)?;
    let matches = command(&catalog).get_matches();
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .try_init()
        .ok();
    let filter = matches.get_one::<String>("enabled-tools");
    let catalog = Catalog::new(filter.map(String::as_str))?;
    let client = MapsClient::new(
        std::env::var("GOOGLE_MAPS_API_KEY").ok(),
        Duration::from_secs(
            *matches
                .get_one::<u64>("timeout")
                .context("missing timeout")?,
        ),
        matches
            .get_one::<String>("language")
            .context("missing language")?
            .clone(),
    )?;
    let engine = EngineState { client, catalog };
    match matches.subcommand() {
        None => server::serve_stdio(engine).await,
        Some(("serve", args)) => match args.subcommand() {
            Some(("stdio", _)) | None => server::serve_stdio(engine).await,
            Some(("http", http)) => {
                let token = std::env::var("MCP_HTTP_TOKEN")
                    .ok()
                    .filter(|s| !s.is_empty());
                if token.as_ref().is_some_and(|s| s.trim().is_empty()) {
                    anyhow::bail!("MCP_HTTP_TOKEN cannot be whitespace");
                }
                server::serve_http(
                    engine,
                    HttpOptions {
                        listen: http
                            .get_one::<String>("listen")
                            .context("missing listen address")?
                            .parse()
                            .context("listen must be an IP:port socket address")?,
                        token,
                        allowed_hosts: http
                            .get_many::<String>("allowed-host")
                            .map(|s| s.cloned().collect())
                            .unwrap_or_default(),
                        allowed_origins: http
                            .get_many::<String>("allowed-origin")
                            .map(|s| s.cloned().collect())
                            .unwrap_or_default(),
                    },
                )
                .await
            }
            _ => anyhow::bail!("unknown transport"),
        },
        Some(("tools", _)) => print_json(&json!(
            engine
                .catalog
                .iter()
                .map(|d| d.tool.clone())
                .collect::<Vec<_>>()
        )),
        Some(("schema", args)) => {
            let name = normalize(args.get_one::<String>("tool").context("missing tool")?);
            print_json(&json!(engine.catalog.get(&name)?.tool.input_schema))
        }
        Some(("call", args)) => {
            let name = normalize(args.get_one::<String>("tool").context("missing tool")?);
            let input = read_args(args.get_one::<String>("args").context("missing args")?)?;
            let output = engine.call(&name, input).await?;
            write_output(output, args.get_one::<String>("output"))
        }
        Some((name, args)) => {
            let tool = normalize(name);
            let definition = engine.catalog.get(&tool)?;
            let properties = definition.tool.input_schema["properties"]
                .as_object()
                .context("invalid schema")?;
            let mut input = serde_json::Map::new();
            for (key, schema) in properties {
                if let Some(value) = args.get_one::<String>(key) {
                    let value = match schema["type"].as_str() {
                        Some("string") => Value::String(value.clone()),
                        _ => serde_json::from_str(value).with_context(|| {
                            format!(
                                "--{} requires a JSON {}",
                                flag(key),
                                schema["type"].as_str().unwrap_or("value")
                            )
                        })?,
                    };
                    input.insert(key.clone(), value);
                }
            }
            let output = engine.call(&tool, Value::Object(input)).await?;
            write_output(output, args.get_one::<String>("output"))
        }
    }
}

fn command(catalog: &Catalog) -> Command {
    let mut command=Command::new("mcp-gmap-rs").version(env!("CARGO_PKG_VERSION"))
        .about("Google Maps CLI and MCP server (stdio or Streamable HTTP)")
        .after_help("No command starts MCP stdio. Set GOOGLE_MAPS_API_KEY for API calls. Objects and arrays use JSON. Example: mcp-gmap-rs geocode --address 'Paris, France'")
        .arg(Arg::new("enabled-tools").long("enabled-tools").env("GOOGLE_MAPS_ENABLED_TOOLS").global(true).help("Comma-separated MCP tool names (default: all)"))
        .arg(Arg::new("timeout").long("timeout").env("GOOGLE_MAPS_TIMEOUT").global(true).default_value("30").value_parser(clap::value_parser!(u64).range(1..=600)).help("Timeout per Google HTTP request in seconds"))
        .arg(Arg::new("language").long("language").env("GOOGLE_MAPS_LANGUAGE").global(true).default_value("en").help("Google response language code"))
        .subcommand(Command::new("serve").about("Launch an MCP server (defaults to stdio)")
            .subcommand(Command::new("stdio").about("MCP over standard input/output"))
            .subcommand(Command::new("http").about("MCP Streamable HTTP at /mcp")
                .arg(Arg::new("listen").long("listen").default_value("127.0.0.1:3000").help("Bind IP:port (use [::1]:3000 for IPv6)"))
                .arg(Arg::new("allowed-host").long("allowed-host").action(ArgAction::Append).help("Allowed HTTP Host; repeat for remote deployment. Defaults to loopback hosts"))
                .arg(Arg::new("allowed-origin").long("allowed-origin").action(ArgAction::Append).help("Allowed browser origin; repeat. Present Origin headers are rejected by default"))))
        .subcommand(Command::new("tools").about("List enabled tools and JSON schemas; no API key needed"))
        .subcommand(Command::new("schema").about("Print a tool's input JSON schema").arg(Arg::new("tool").required(true)))
        .subcommand(Command::new("call").about("Call a tool with JSON arguments; --args - reads stdin")
            .arg(Arg::new("tool").required(true)).arg(Arg::new("args").long("args").required(true)).arg(output_arg()));
    for definition in catalog.iter() {
        let tool = &definition.tool;
        let mut sub = Command::new(cli_name(&tool.name))
            .about(tool.description.as_deref().unwrap_or_default().to_owned())
            .arg(output_arg());
        if let Some(properties) = tool.input_schema["properties"].as_object() {
            for (key, schema) in properties {
                let required = tool.input_schema["required"]
                    .as_array()
                    .is_some_and(|r| r.iter().any(|r| r == key));
                let mut arg = Arg::new(key.clone())
                    .long(flag(key))
                    .required(required)
                    .allow_hyphen_values(true)
                    .value_name(match schema["type"].as_str() {
                        Some("object" | "array") => "JSON",
                        Some("boolean") => "BOOL",
                        Some("number" | "integer") => "NUMBER",
                        _ => "TEXT",
                    });
                if schema["type"] == "boolean" {
                    arg = arg
                        .num_args(0..=1)
                        .default_missing_value("true")
                        .allow_hyphen_values(false)
                        .value_parser(["true", "false"]);
                }
                sub = sub.arg(arg);
            }
        }
        command = command.subcommand(sub);
    }
    command
}

fn output_arg() -> Arg {
    Arg::new("output")
        .long("output")
        .short('o')
        .help("Write JSON or raw static-map PNG to a file; default: JSON on stdout")
}
fn cli_name(name: &str) -> String {
    name.strip_prefix("maps_").unwrap_or(name).replace('_', "-")
}
fn normalize(name: &str) -> String {
    if name.starts_with("maps_") {
        name.into()
    } else {
        format!("maps_{}", name.replace('-', "_"))
    }
}
fn flag(name: &str) -> String {
    let mut result = String::new();
    for ch in name.chars() {
        if ch.is_ascii_uppercase() {
            result.push('-');
            result.push(ch.to_ascii_lowercase());
        } else if ch == '_' {
            result.push('-');
        } else {
            result.push(ch);
        }
    }
    result
}

fn read_args(input: &str) -> anyhow::Result<Value> {
    if input == "-" {
        let mut buffer = String::new();
        io::stdin()
            .take(1024 * 1024 + 1)
            .read_to_string(&mut buffer)?;
        if buffer.len() > 1024 * 1024 {
            anyhow::bail!("JSON arguments exceed 1 MiB");
        }
        serde_json::from_str(&buffer).context("invalid JSON arguments on stdin")
    } else {
        serde_json::from_str(input).context("invalid JSON arguments")
    }
}

fn write_output(output: Output, path: Option<&String>) -> anyhow::Result<()> {
    if let Some(path) = path {
        let bytes = match output {
            Output::Json(value) => serde_json::to_vec_pretty(&value)?,
            Output::Image(bytes) => bytes,
        };
        std::fs::write(path, bytes).with_context(|| format!("cannot write {path}"))?;
        Ok(())
    } else {
        print_json(&output.into_json())
    }
}

fn print_json(value: &Value) -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, value)?;
    writeln!(stdout)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_contract_is_valid() {
        command(&Catalog::new(None).unwrap()).debug_assert();
    }

    #[test]
    fn flags_support_negative_coordinates_and_boolean_switches() {
        let catalog = Catalog::new(None).unwrap();
        let args = command(&catalog)
            .try_get_matches_from([
                "mcp-gmap-rs",
                "reverse-geocode",
                "--latitude",
                "-33.8",
                "--longitude",
                "151.2",
            ])
            .unwrap();
        assert_eq!(
            args.subcommand()
                .unwrap()
                .1
                .get_one::<String>("latitude")
                .unwrap(),
            "-33.8"
        );
        let args = command(&catalog)
            .try_get_matches_from([
                "mcp-gmap-rs",
                "search-places",
                "--query",
                "coffee",
                "--open-now",
                "--min-rating",
                "4",
            ])
            .unwrap();
        assert_eq!(
            args.subcommand()
                .unwrap()
                .1
                .get_one::<String>("openNow")
                .unwrap(),
            "true"
        );
    }
}
