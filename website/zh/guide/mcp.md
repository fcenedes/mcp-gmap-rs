# MCP 集成 {#mcp-integration}

本地子进程使用 stdio 连接 MCP 宿主，长期运行的服务使用 Streamable HTTP。两者公开相同的已启用工具目录和模式。

## 本地 stdio {#local-stdio}

通用 MCP 客户端配置如下：

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

使用宿主要求的配置格式，并指定可执行文件的绝对路径。所选 dotenv 文件必须存在。也可通过宿主注入环境变量。这是独立 MCP 服务器的配置，不需要特定助手的运行时插件。

```sh
mcp-gmap-rs serve stdio
```

标准输入和标准输出仅承载换行分隔的 MCP JSON-RPC。诊断信息写入 stderr。不带参数运行可执行文件会启动同一传输。容器形式的 stdio 宿主必须连接 stdin，且不能分配 TTY。

## Streamable HTTP {#streamable-http}

```sh
mcp-gmap-rs serve http --listen 127.0.0.1:3000
```

将 MCP 客户端的 Streamable HTTP URL 设置为 **`http://127.0.0.1:3000/mcp`**。如果宿主支持远程 MCP 条目，请配置端点与授权头：

```json
{
  "url": "https://maps.example.com/mcp",
  "headers": {
    "Authorization": "Bearer your-mcp-http-token"
  }
}
```

此条目外层的具体结构由 MCP 宿主决定。服务器通过 Rust MCP SDK 支持协议协商，以及适用的 POST、GET/SSE 和 DELETE 会话操作。应使用 MCP 客户端，不要将一次孤立的 JSON-RPC POST 当作完整会话。

## 监听地址与访问 {#bind-addresses-and-access}

非回环监听需要 `MCP_HTTP_TOKEN`。客户端以 bearer 凭据发送该令牌，与 Google OAuth 或 API 密钥相互独立。请显式添加端点 Host 值和实际浏览器来源：

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3000 \
  --allowed-host maps.example.com \
  --allowed-origin https://app.example.com:443
```

默认情况下，存在但不在允许列表中的 Origin 头会被拒绝。请求体限制为 1 MiB。`/health` 只检查进程存活，不会调用 Google。公开访问应配置[原生 TLS/mTLS 或网关](./production)。代理必须保留授权和 MCP 协议/会话头，并支持不缓冲的流式传输。会话与 Google 身份属于单个进程；添加副本前请阅读[部署](./deployment)。
