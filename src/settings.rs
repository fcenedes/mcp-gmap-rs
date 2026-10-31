use std::{collections::HashMap, fs::File, io::ErrorKind, path::PathBuf};

use anyhow::{Context, bail};
use clap::ArgMatches;
use config::{Config, Environment};

use crate::auth::OAuth;
use crate::server::HttpOptions;

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
];

pub(crate) struct Settings {
    pub api_key: Option<String>,
    pub oauth: Option<OAuth>,
    pub language: String,
    pub timeout: u64,
    pub enabled_tools: String,
    pub log_filter: String,
    http_listen: String,
    http_token: Option<String>,
    http_allowed_hosts: String,
    http_allowed_origins: String,
}

impl Settings {
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
            .add_source(source(file))
            .add_source(source(environment));
        for (arg, key) in [("language", "language"), ("enabled-tools", "enabled_tools")] {
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
        if let Some(("serve", serve)) = matches.subcommand()
            && let Some(("http", http)) = serve.subcommand()
        {
            if let Some(listen) = http.get_one::<String>("listen") {
                builder = builder.set_override("http_listen", listen.clone())?;
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
        Ok(HttpOptions {
            listen: self
                .http_listen
                .parse()
                .context("MCP_HTTP_LISTEN / --listen must be an IP:port socket address")?,
            token: self.http_token.clone(),
            allowed_hosts: comma_list(&self.http_allowed_hosts),
            allowed_origins: comma_list(&self.http_allowed_origins),
        })
    }
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
