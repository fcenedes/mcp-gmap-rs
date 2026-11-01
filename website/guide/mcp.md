# MCP integration

Connect an MCP host through stdio for a local child process or Streamable HTTP for a long-running service. Both expose the same enabled tool catalog and schemas.

## Local stdio

A generic MCP client configuration is:

```json
{
  "mcpServers": {
    "google-maps": {
      "command": "/absolute/path/to/mcp-gmap-rs",
      "args": ["--env-file", "/secure/mcp-gmap/runtime.env", "serve", "stdio"]
    }
  }
}
```

Use the host’s configuration format and an absolute executable path. The selected dotenv file must exist. You may inject environment variables using the host instead. This is a standalone MCP server configuration; no assistant-specific runtime plugin is required.

```sh
mcp-gmap-rs serve stdio
```

Standard input and output carry newline-delimited MCP JSON-RPC only. Diagnostics go to stderr. Running the executable without arguments starts the same transport. A container-based stdio host must attach stdin without allocating a TTY.

## Streamable HTTP

```sh
mcp-gmap-rs serve http --listen 127.0.0.1:3000
```

Set your MCP client’s Streamable HTTP URL to **`http://127.0.0.1:3000/mcp`**. Where the host supports remote MCP entries, configure the endpoint and authorization header:

```json
{
  "url": "https://maps.example.com/mcp",
  "headers": {
    "Authorization": "Bearer your-mcp-http-token"
  }
}
```

The exact wrapper around this entry depends on the MCP host. The server supports protocol negotiation and the appropriate POST, GET/SSE and DELETE session operations through the Rust MCP SDK. Use an MCP client rather than treating one isolated JSON-RPC POST as a complete session.

## Bind addresses and access

Non-loopback listeners require `MCP_HTTP_TOKEN`. Clients send that token as a bearer credential; it is separate from Google OAuth or API keys. Add the endpoint’s Host value and any actual browser origins explicitly:

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3000 \
  --allowed-host maps.example.com \
  --allowed-origin https://app.example.com:443
```

The default rejects present Origin headers unless allowlisted. Request bodies are limited to 12 MiB. `/health` checks process liveness without calling Google. For public access, configure [native TLS/mTLS or a gateway](./production). Proxies must preserve authorization and MCP protocol/session headers and allow unbuffered streaming. Sessions and Google identity are process-local; see [deployment](./deployment) before adding replicas.
