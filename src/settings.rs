use std::{collections::HashMap, fs::File, io::ErrorKind, path::PathBuf};

use anyhow::{Context, bail};
use clap::ArgMatches;
use config::{Config, Environment};

use crate::auth::OAuth;
use crate::server::HttpOptions;
use crate::{policy::ClientPolicy, tls::TlsFiles};

const VARIABLES: &[(&str, &str)] = &[
    ("GOOGLE_MAPS_API_KEY", "api_key"),
    ("GOOGLE_MAPS_AUTH_MODE", "auth_mode"),
    ("GOOGLE_APPLICATION_CREDENTIALS", "credentials_file"),
    ("GOOGLE_CLOUD_QUOTA_PROJECT", "quota_project"),
    ("GOOGLE_MAPS_LANGUAGE", "language"),
    ("GOOGLE_MAPS_TIMEOUT", "timeout"),
    ("GOOGLE_MAPS_ENABLED_TOOLS", "enabled_tools"),
    ("MCP_HTTP_TOKEN", "http_token"),
    ("MCP_HTTP_LISTEN", "http_listen"),
    ("MCP_HTTP_ALLOWED_HOSTS", "http_allowed_hosts"),
    ("MCP_HTTP_ALLOWED_ORIGINS", "http_allowed_origins"),
    ("RUST_LOG", "log_filter"),
    ("MCP_LOG_FORMAT", "log_format"),
    ("GOOGLE_MAPS_MAX_REQUESTS", "max_requests"),
    ("GOOGLE_MAPS_MAX_RETRIES", "max_retries"),
    ("GOOGLE_MAPS_TOOL_TIMEOUT", "tool_timeout"),
    ("MCP_HTTP_RATE_LIMIT", "http_rate_limit"),
    ("MCP_METRICS_ENABLED", "metrics_enabled"),
    ("MCP_TLS_CERT", "tls_cert"),
    ("MCP_TLS_KEY", "tls_key"),
    ("MCP_TLS_CLIENT_CA", "tls_client_ca"),
];

pub(crate) struct Settings {
    pub api_key: Option<String>,
    pub oauth: Option<OAuth>,
    pub language: String,
    pub timeout: u64,
    pub enabled_tools: String,
    pub log_filter: String,
    pub log_format: String,
    pub policy: ClientPolicy,
    tls_cert: String,
    tls_key: String,
    tls_client_ca: String,
    http_rate_limit: u32,
    metrics_enabled: bool,
    http_listen: String,
    http_token: Option<String>,
    http_allowed_hosts: String,
    http_allowed_origins: String,
}

impl Settings {
    #[cfg(test)]
    pub fn test_defaults() -> Self {
        Self {
            api_key: None,
            oauth: None,
            language: "en".into(),
            timeout: 30,
            enabled_tools: "*".into(),
            log_filter: "warn".into(),
            log_format: "text".into(),
            policy: ClientPolicy::default(),
            tls_cert: String::new(),
            tls_key: String::new(),
            tls_client_ca: String::new(),
            http_rate_limit: 120,
            metrics_enabled: false,
            http_listen: "127.0.0.1:3000".into(),
            http_token: None,
            http_allowed_hosts: String::new(),
            http_allowed_origins: String::new(),
        }
    }

    pub fn load(matches: &ArgMatches) -> anyhow::Result<Self> {
        let file = dotenv_values(matches)?;
        let mut environment = HashMap::new();
        for &(name, _) in VARIABLES {
            match std::env::var(name) {
                Ok(value) => {
                    environment.insert(name.to_owned(), value);
                }
                Err(std::env::VarError::NotPresent) => {}
                Err(std::env::VarError::NotUnicode(_)) => {
                    bail!("{name} must contain valid Unicode")
                }
            }
        }
        let mut builder = Config::builder()
            .set_default("auth_mode", "api-key")?
            .set_default("language", "en")?
            .set_default("timeout", "30")?
            .set_default("enabled_tools", "*")?
            .set_default("log_filter", "warn")?
            .set_default("http_listen", "127.0.0.1:3000")?
            .set_default("http_allowed_hosts", "")?
            .set_default("http_allowed_origins", "")?
            .set_default("log_format", "text")?
            .set_default("max_requests", "200")?
            .set_default("max_retries", "2")?
            .set_default("tool_timeout", "120")?
            .set_default("http_rate_limit", "120")?
            .set_default("metrics_enabled", "false")?
            .set_default("tls_cert", "")?
            .set_default("tls_key", "")?
            .set_default("tls_client_ca", "")?
            .add_source(source(file))
            .add_source(source(environment));
        for (arg, key) in [
            ("language", "language"),
            ("enabled-tools", "enabled_tools"),
            ("log-format", "log_format"),
        ] {
            if let Some(value) = matches.get_one::<String>(arg) {
                builder = builder.set_override(key, value.clone())?;
            }
        }
        if let Some(mode) = matches.get_one::<String>("auth-mode") {
            builder = builder.set_override("auth_mode", mode.clone())?;
        }
        if let Some(timeout) = matches.get_one::<u64>("timeout") {
            builder = builder.set_override("timeout", timeout.to_string())?;
        }
        for (arg, key) in [
            ("max-requests", "max_requests"),
            ("max-retries", "max_retries"),
            ("tool-timeout", "tool_timeout"),
        ] {
            if let Some(value) = matches.get_one::<u32>(arg) {
                builder = builder.set_override(key, value.to_string())?;
            }
        }
        if let Some(("serve", serve)) = matches.subcommand()
            && let Some(("http", http)) = serve.subcommand()
        {
            if let Some(listen) = http.get_one::<String>("listen") {
                builder = builder.set_override("http_listen", listen.clone())?;
            }
            for (arg, key) in [
                ("tls-cert", "tls_cert"),
                ("tls-key", "tls_key"),
                ("tls-client-ca", "tls_client_ca"),
            ] {
                if let Some(value) = http.get_one::<String>(arg) {
                    builder = builder.set_override(key, value.clone())?;
                }
            }
            if let Some(value) = http.get_one::<u32>("rate-limit") {
                builder = builder.set_override("http_rate_limit", value.to_string())?;
            }
            if let Some(value) = http.get_one::<String>("metrics") {
                builder = builder.set_override("metrics_enabled", value.clone())?;
            }
            for (arg, key) in [
                ("allowed-host", "http_allowed_hosts"),
                ("allowed-origin", "http_allowed_origins"),
            ] {
                if let Some(values) = http.get_many::<String>(arg) {
                    builder =
                        builder.set_override(key, values.cloned().collect::<Vec<_>>().join(","))?;
                }
            }
        }
        let config = builder.build()?;
        let log_format = config.get_string("log_format")?;
        if !matches!(log_format.as_str(), "text" | "json") {
            bail!("MCP_LOG_FORMAT must be text or json");
        }
        let metrics_enabled = config
            .get::<bool>("metrics_enabled")
            .ok()
            .context("MCP_METRICS_ENABLED must be true or false")?;
        let timeout = config
            .get::<u64>("timeout")
            .ok()
            .filter(|value| (1..=600).contains(value))
            .context("GOOGLE_MAPS_TIMEOUT must be an integer between 1 and 600 seconds")?;
        let oauth = match config.get_string("auth_mode")?.as_str() {
            "api-key" => None,
            "oauth" => Some(OAuth::new(
                config
                    .get_string("credentials_file")
                    .ok()
                    .map(PathBuf::from),
                config.get_string("quota_project").ok(),
                std::time::Duration::from_secs(timeout),
            )),
            _ => bail!("GOOGLE_MAPS_AUTH_MODE must be api-key or oauth"),
        };
        Ok(Self {
            api_key: config.get_string("api_key").ok(),
            oauth,
            language: config.get_string("language")?,
            timeout,
            enabled_tools: config.get_string("enabled_tools")?,
            log_filter: config.get_string("log_filter")?,
            log_format,
            policy: ClientPolicy {
                max_requests: integer(
                    &config,
                    "max_requests",
                    "GOOGLE_MAPS_MAX_REQUESTS",
                    1,
                    10000,
                )?,
                max_retries: integer(&config, "max_retries", "GOOGLE_MAPS_MAX_RETRIES", 0, 5)?,
                tool_timeout: std::time::Duration::from_secs(u64::from(integer(
                    &config,
                    "tool_timeout",
                    "GOOGLE_MAPS_TOOL_TIMEOUT",
                    1,
                    3600,
                )?)),
            },
            tls_cert: config.get_string("tls_cert")?,
            tls_key: config.get_string("tls_key")?,
            tls_client_ca: config.get_string("tls_client_ca")?,
            http_rate_limit: integer(
                &config,
                "http_rate_limit",
                "MCP_HTTP_RATE_LIMIT",
                0,
                1000000,
            )?,
            metrics_enabled,
            http_listen: config.get_string("http_listen")?,
            http_token: config
                .get_string("http_token")
                .ok()
                .filter(|value| !value.is_empty()),
            http_allowed_hosts: config.get_string("http_allowed_hosts")?,
            http_allowed_origins: config.get_string("http_allowed_origins")?,
        })
    }

    pub fn http_options(&self) -> anyhow::Result<HttpOptions> {
        if self
            .http_token
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            bail!("MCP_HTTP_TOKEN cannot be whitespace");
        }
        let tls = match (
            self.tls_cert.is_empty(),
            self.tls_key.is_empty(),
            self.tls_client_ca.is_empty(),
        ) {
            (true, true, true) => None,
            (false, false, _) => Some(TlsFiles {
                cert: self.tls_cert.clone().into(),
                key: self.tls_key.clone().into(),
                client_ca: (!self.tls_client_ca.is_empty())
                    .then(|| self.tls_client_ca.clone().into()),
            }),
            _ => bail!(
                "MCP_TLS_CERT and MCP_TLS_KEY must be supplied together; MCP_TLS_CLIENT_CA requires both"
            ),
        };
        Ok(HttpOptions {
            listen: self
                .http_listen
                .parse()
                .context("MCP_HTTP_LISTEN / --listen must be an IP:port socket address")?,
            token: self.http_token.clone(),
            allowed_hosts: comma_list(&self.http_allowed_hosts),
            allowed_origins: comma_list(&self.http_allowed_origins),
            rate_limit: self.http_rate_limit,
            metrics: self.metrics_enabled,
            tls,
        })
    }
}

fn integer(config: &Config, key: &str, variable: &str, min: u32, max: u32) -> anyhow::Result<u32> {
    config
        .get::<u32>(key)
        .ok()
        .filter(|value| (min..=max).contains(value))
        .with_context(|| format!("{variable} must be an integer between {min} and {max}"))
}

fn source(values: HashMap<String, String>) -> Environment {
    let mapped = VARIABLES
        .iter()
        .filter_map(|&(name, key)| {
            values
                .get(name)
                .map(|value| (key.to_owned(), value.clone()))
        })
        .collect();
    Environment::default().source(Some(mapped))
}

fn comma_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn dotenv_values(matches: &ArgMatches) -> anyhow::Result<HashMap<String, String>> {
    if matches.get_flag("no-env-file") {
        return Ok(HashMap::new());
    }
    let explicit = matches
        .get_one::<String>("env-file")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("MCP_ENV_FILE").map(PathBuf::from));
    let path = explicit
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new(".env"));
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound && explicit.is_none() => {
            return Ok(HashMap::new());
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("cannot read dotenv file {}", path.display()));
        }
    };
    // Parser errors can contain the original line, including credentials.
    dotenvy::from_read_iter(file)
        .collect::<Result<_, _>>()
        .map_err(|_| {
            anyhow::anyhow!(
                "cannot parse dotenv file {}; check its KEY=value syntax",
                path.display()
            )
        })
}
