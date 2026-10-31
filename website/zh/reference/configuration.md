# 配置参考 {#configuration-reference}

以下 21 项应用设置按适用范围用于 CLI、MCP stdio 和 Streamable HTTP。值遵循**默认值 → 可选 dotenv → 环境变量 → 选项**的顺序。文件选择与覆盖语义见[配置](../guide/configuration)。

全局选项位于工具或 `serve` 之前；HTTP 专用选项位于 `serve http` 之后。选项列的破折号表示无对应 CLI 选项，应通过环境变量或所选 dotenv 文件注入。

## Google 访问与工具执行 {#google-access-and-tool-execution}

| 环境变量 | 默认值 | CLI 选项 | 含义与允许值 |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_API_KEY` | 未设置 | — | 受限制的 Google API 密钥。`api-key` 模式和本实现中仅支持密钥的端点必须提供，即使选择 OAuth 也一样。绝不是工具参数。 |
| `GOOGLE_MAPS_AUTH_MODE` | `api-key` | `--auth-mode` | `api-key` 或 `oauth`。OAuth 支持 Places (New)、Routes、Weather、Air Quality、Address Validation 与 Places Aggregate。 |
| `GOOGLE_APPLICATION_CREDENTIALS` | ADC 发现 | — | 可读的服务账号或受支持 ADC JSON 文件路径。相对路径基于工作目录。本地 ADC 或元数据发现时不设置。 |
| `GOOGLE_CLOUD_QUOTA_PROJECT` | 未设置 | — | OAuth 必需的消费方结算/配额项目 ID 或编号。服务账号在该项目中需要 `serviceusage.services.use`。 |
| `GOOGLE_MAPS_LANGUAGE` | `en` | `--language` | Google 支持语言响应的语言代码，与本手册的语言选择独立。 |
| `GOOGLE_MAPS_TIMEOUT` | `30` | `--timeout` | 单次请求超时秒数，整数 1–600。包含排队、重试、退避及读取响应；OAuth 获取使用同一值单独限时。 |
| `GOOGLE_MAPS_ENABLED_TOOLS` | 全部（`*`） | `--enabled-tools` | 逗号分隔的 `maps_*` 名称或 `*`。未知名称导致启动失败；禁用工具不会出现在发现中，也不可调用。 |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | 每次工具调用的 Google HTTP 尝试预算，整数 1–10000。复合分支共享，重试消耗次数。 |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | 每个 Google 请求的额外临时故障尝试次数，整数 0–5。零表示仅首次尝试。重试受超时和预算限制。 |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | 整个工具期限秒数，整数 1–3600。包含凭据获取、排队、API 调用和重试。 |

每个进程可同时执行五个 Google HTTP 请求。预算限制请求次数，而不是金额；一次矩阵请求可能对多个元素计费。当分支耗尽预算或超过期限时，一些复合工具会返回逐项错误。见[生产限制](../guide/production#limits-retries-and-deadlines)。

## HTTP 监听与访问 {#http-listener-and-access}

| 环境变量 | 默认值 | HTTP 选项 | 含义与允许值 |
| --- | --- | --- | --- |
| `MCP_HTTP_TOKEN` | 未设置 | — | MCP 客户端共享的 bearer 令牌。非回环监听必需，与 Google 凭据独立，即使使用原生 mTLS 仍需提供。 |
| `MCP_HTTP_LISTEN` | `127.0.0.1:3000` | `--listen` | 监听 IP 和端口。Docker 镜像将该环境值设为 `0.0.0.0:3000`。 |
| `MCP_HTTP_ALLOWED_HOSTS` | 回环主机 | `--allowed-host` | 逗号分隔的允许 HTTP Host 值。多次选项替换整个文件/环境列表，需包含相应端口。 |
| `MCP_HTTP_ALLOWED_ORIGINS` | 无 | `--allowed-origin` | 逗号分隔的允许浏览器来源。存在但未获允许的 Origin 头会被拒绝。显式写端口；重复选项替换整个列表。 |
| `MCP_HTTP_RATE_LIMIT` | `120` | `--rate-limit` | 每进程每分钟已认证 `/mcp` 请求数，整数 0–1000000。零禁用；突发容量等于速率并持续补充。 |
| `MCP_METRICS_ENABLED` | `false` | `--metrics [true\|false]` | 启用 Prometheus `/metrics`。单独 `--metrics` 表示 true。配置了 bearer 凭据时使用相同凭据；原生 mTLS 同样生效。 |
| `MCP_TLS_CERT` | 未设置 | `--tls-cert` | PEM 服务器叶证书及中间证书链，必须与匹配私钥一起提供。 |
| `MCP_TLS_KEY` | 未设置 | `--tls-key` | PEM 服务器私钥路径，需运行身份可读，且不放入源代码/镜像。 |
| `MCP_TLS_CLIENT_CA` | 未设置 | `--tls-client-ca` | PEM 客户端 CA 包，使所有监听路径都要求客户端证书；不替代 bearer 令牌。 |

HTTP 令牌桶按进程全局生效，不按用户/IP 区分。未授权请求不消耗令牌，健康检查与指标不受限。拒绝返回 429 和 `Retry-After`。请求体上限为 1 MiB。参阅 [MCP 访问](../guide/mcp#bind-addresses-and-access)与 [TLS 运维](../guide/production#native-tls-and-mutual-tls)。

## 诊断事件 {#diagnostic-events}

| 环境变量 | 默认值 | CLI 选项 | 含义与允许值 |
| --- | --- | --- | --- |
| `RUST_LOG` | `warn` | — | 诊断日志过滤器。`info` 启用工具完成事件，不记录参数或 Google 响应正文。 |
| `MCP_LOG_FORMAT` | `text` | `--log-format` | `text` 或 `json`。诊断始终写入 stderr，使 stdout 保留给 CLI JSON 或 MCP 流量。 |

## 文件选择与输出控制 {#file-selection-and-output-controls}

文件选择在上述应用配置层之前进行。`MCP_ENV_FILE` 选择 dotenv 路径，全局 `--env-file PATH` 优先。显式选择的文件必须存在。`--no-env-file` 禁用全部 dotenv 加载，且不能与 `--env-file` 同用。没有指定时，只读取工作目录下可选的 `.env`。

显式空环境值仍覆盖文件值，可清空可选令牌、TLS 路径或允许列表，但不能让空数值设置变得有效。加载文件不会修改进程环境。

`doctor --live` 显式启用可能计费的探测；普通 `doctor` 为离线。工具 `--output PATH` 保存 JSON，对 `static-map` 则保存原始 PNG。这些是命令控制，不是额外环境设置。

应用设置和凭据文件变更需重启。在 Linux/macOS 上，SIGHUP 可共同重新加载原生 TLS 证书/私钥/客户端 CA 文件；重载失败会保留此前 TLS 配置。参阅[配置](../guide/configuration)与[生产运维](../guide/production)。
