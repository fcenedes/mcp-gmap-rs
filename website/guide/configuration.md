# Configuration

The CLI and both MCP transports share the same startup configuration. Choose a file for local work, environment injection for containers, and flags for explicit per-process overrides.

## Precedence

Values are applied in this order, with later layers taking priority:

**Built-in defaults → optional dotenv file → process environment → CLI flags**

The default file is `.env` in the current working directory. Parent directories are not searched. A missing default file is allowed; an unreadable or malformed file fails startup without printing its contents.

```sh
mcp-gmap-rs --env-file /etc/mcp-gmap/runtime.env serve http
mcp-gmap-rs --no-env-file tools
```

`MCP_ENV_FILE` selects a file too, while `--env-file` takes precedence. An explicitly selected file must exist. `--no-env-file` disables file loading, including `MCP_ENV_FILE`; it conflicts with `--env-file`. Dotenv parsing does not mutate the process environment.

## Override a mounted configuration

```dotenv
GOOGLE_MAPS_LANGUAGE=en
GOOGLE_MAPS_TIMEOUT=30
GOOGLE_MAPS_ENABLED_TOOLS=maps_search_places,maps_place_details
```

With that file mounted, an environment variable changes the language and a flag changes the timeout:

```sh
GOOGLE_MAPS_LANGUAGE=fr mcp-gmap-rs \
  --env-file /etc/mcp-gmap/runtime.env --timeout 15 tools
```

Repeated `--allowed-host` and `--allowed-origin` flags replace their corresponding file/environment lists as a whole. An explicitly empty environment value also overrides the file: it can clear an optional token or list, while an empty timeout is invalid.

## Control the tool surface

`GOOGLE_MAPS_ENABLED_TOOLS` or `--enabled-tools` accepts comma-separated logical names such as `maps_weather,maps_air_quality`, or `*` for all tools. Unknown names fail startup. Disabled tools disappear from discovery and cannot be called. Discovery and schema inspection need no credentials, but a requested API operation must have the appropriate Google credential.

Application settings are read at startup. Restart after changing environment values, credential JSON or Kubernetes Secret/ConfigMap configuration. Native TLS certificate, key and client-CA files are the exception: they support [SIGHUP reload](./production#native-tls-and-mutual-tls).

See the [complete configuration reference](../reference/configuration) for all 21 settings, defaults, flags and ranges. Website language selection does not change `GOOGLE_MAPS_LANGUAGE`, which controls Google response text.
