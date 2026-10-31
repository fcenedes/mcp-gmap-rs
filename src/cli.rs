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
    doctor, server,
    settings::Settings,
    tools::{EngineState, Output},
};

pub(crate) async fn run() -> anyhow::Result<()> {
    let catalog = Catalog::new(None)?;
    let matches = command(&catalog).get_matches();
    let settings = Settings::load(&matches)?;
    let logging = tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(
            EnvFilter::try_new(&settings.log_filter).unwrap_or_else(|_| EnvFilter::new("warn")),
        );
    if settings.log_format == "json" {
        logging.json().try_init().ok();
    } else {
        logging.try_init().ok();
    }
    let catalog = Catalog::new(Some(&settings.enabled_tools))?;
    let mut client = MapsClient::new(
        settings.api_key.clone(),
        Duration::from_secs(settings.timeout),
        settings.language.clone(),
    )?;
    client.oauth = settings.oauth.clone();
    client.policy = settings.policy.clone();
    let engine = EngineState { client, catalog };
    match matches.subcommand() {
        Some(("doctor", args)) => {
            let report = doctor::report(&engine, &settings, args.get_flag("live")).await;
            let healthy = report["ok"] == true;
            print_json(&report)?;
            if !healthy {
                anyhow::bail!("doctor found failed checks; see JSON report");
            }
            Ok(())
        }
        None => server::serve_stdio(engine).await,
        Some(("serve", args)) => match args.subcommand() {
            Some(("stdio", _)) | None => server::serve_stdio(engine).await,
            Some(("http", _)) => server::serve_http(engine, settings.http_options()?).await,
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
        .after_help("No command starts MCP stdio. Set GOOGLE_MAPS_API_KEY, or select --auth-mode oauth for supported APIs with service accounts/ADC. Objects and arrays use JSON. Example: mcp-gmap-rs geocode --address 'Paris, France'")
        .arg(Arg::new("env-file").long("env-file").global(true).conflicts_with("no-env-file").help("Load this dotenv file (or MCP_ENV_FILE); default: optional .env in the working directory"))
        .arg(Arg::new("no-env-file").long("no-env-file").global(true).action(ArgAction::SetTrue).help("Disable dotenv file loading, including MCP_ENV_FILE"))
        .arg(Arg::new("auth-mode").long("auth-mode").global(true).value_parser(["api-key", "oauth"]).help("Override GOOGLE_MAPS_AUTH_MODE (default: api-key); OAuth uses ADC/service accounts or service-specific end-user credentials"))
        .arg(Arg::new("enabled-tools").long("enabled-tools").global(true).help("Override GOOGLE_MAPS_ENABLED_TOOLS; comma-separated MCP names (default: all)"))
        .arg(Arg::new("timeout").long("timeout").global(true).value_parser(clap::value_parser!(u64).range(1..=600)).help("Override GOOGLE_MAPS_TIMEOUT; seconds per request (default: 30)"))
        .arg(Arg::new("language").long("language").global(true).help("Override GOOGLE_MAPS_LANGUAGE (default: en)"))
        .arg(Arg::new("log-format").long("log-format").global(true).value_parser(["text","json"]).help("Override MCP_LOG_FORMAT (default: text); diagnostics always go to stderr"))
        .arg(Arg::new("max-requests").long("max-requests").global(true).value_parser(clap::value_parser!(u32).range(1..=10000)).help("Google HTTP attempt budget per tool call, including retries (default: 200)"))
        .arg(Arg::new("max-retries").long("max-retries").global(true).value_parser(clap::value_parser!(u32).range(0..=5)).help("Transient Google request retries within --timeout (default: 2)"))
        .arg(Arg::new("tool-timeout").long("tool-timeout").global(true).value_parser(clap::value_parser!(u32).range(1..=3600)).help("Overall tool deadline, including authentication, queueing and retries (default: 120 seconds)"))
        .subcommand(Command::new("doctor").about("Check local configuration; --live also makes billable Google API probes")
            .arg(Arg::new("live").long("live").action(ArgAction::SetTrue).help("Opt in to billable checks of each API used by enabled tools")))
        .subcommand(Command::new("serve").about("Launch an MCP server (defaults to stdio)")
            .subcommand(Command::new("stdio").about("MCP over standard input/output"))
            .subcommand(Command::new("http").about("MCP Streamable HTTP at /mcp")
                .arg(Arg::new("listen").long("listen").help("Override MCP_HTTP_LISTEN; IP:port (default: 127.0.0.1:3000)"))
                .arg(Arg::new("tls-cert").long("tls-cert").help("PEM server certificate chain (MCP_TLS_CERT); requires --tls-key or MCP_TLS_KEY"))
                .arg(Arg::new("tls-key").long("tls-key").help("PEM server private key file (MCP_TLS_KEY)"))
                .arg(Arg::new("tls-client-ca").long("tls-client-ca").help("PEM CA bundle; require client certificates (MCP_TLS_CLIENT_CA)"))
                .arg(Arg::new("rate-limit").long("rate-limit").value_parser(clap::value_parser!(u32).range(0..=1000000)).help("Authenticated /mcp HTTP requests per minute per process, burst of the same size; 0 disables (default: 120)"))
                .arg(Arg::new("metrics").long("metrics").num_args(0..=1).default_missing_value("true").value_parser(["true","false"]).help("Enable protected Prometheus /metrics; overrides MCP_METRICS_ENABLED (default: false)"))
                .arg(Arg::new("allowed-host").long("allowed-host").action(ArgAction::Append).help("Replace MCP_HTTP_ALLOWED_HOSTS; repeat for multiple hosts. Defaults to loopback hosts"))
                .arg(Arg::new("allowed-origin").long("allowed-origin").action(ArgAction::Append).help("Replace MCP_HTTP_ALLOWED_ORIGINS; repeat. Present Origin headers are rejected by default"))))
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
                if let Some(description) = schema["description"].as_str() {
                    arg = arg.help(description.to_owned());
                }
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
        .help("Write JSON or raw binary response to a file; default: JSON on stdout")
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
            .take(12 * 1024 * 1024 + 1)
            .read_to_string(&mut buffer)?;
        if buffer.len() > 12 * 1024 * 1024 {
            anyhow::bail!("JSON arguments exceed 12 MiB");
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
            Output::Binary(value) => value.data,
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
