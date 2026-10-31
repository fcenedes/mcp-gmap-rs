# 生产环境运维 {#production-operations}

本指南适用于 v0.3.x。CLI、MCP stdio 与 Streamable HTTP 中的所有 Google 工具共享同一实现。

## 原生 TLS 与双向 TLS {#native-tls-and-mutual-tls}

HTTP 监听器可使用 rustls 和 TLS 1.2/1.3 直接提供 HTTPS：

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3443 \
  --tls-cert /etc/mcp-gmap/tls/tls.crt \
  --tls-key /etc/mcp-gmap/tls/tls.key \
  --allowed-host maps.example.com:3443
```

通过环境变量或 Secret 单独提供 `MCP_HTTP_TOKEN`。客户端连接 `https://maps.example.com:3443/mcp`，验证服务器证书并发送 `Authorization: Bearer <token>`。证书文件包含 PEM 叶证书，后接中间证书链；私钥必须匹配。可执行文件不签发证书，也不实现 ACME。签发与续期请使用自己的 PKI、cert-manager 或 TLS 网关。

添加 `--tls-client-ca /etc/mcp-gmap/tls/client-ca.pem`，要求客户端证书由该 CA 签署。原生 mTLS 适用于所有路径，包括 `/health` 和 `/metrics`，且不能替代 HTTP bearer 令牌。TLS 层接受配置的信任根下的任何有效客户端身份；没有按证书分配工具权限或检查撤销列表的功能。

Linux/macOS 上原子替换文件后，发送 `SIGHUP`，一起重新加载证书、私钥与可选客户端 CA。已有连接保持其 TLS 状态，新握手使用替换后的配置。如果加载失败，保留旧配置，并在日志中报错而不打印密钥材料。挂载的 Kubernetes Secret 文件在投射更新后也可如此重载。其他设置仍需重启进程。

容器以 UID/GID 10001 运行，应只读挂载密钥，并赋予该身份读取权限；投射 Secret 可使用合适的组与 `fsGroup`。原生 TLS 下，Kubernetes HTTP 探针需设置 `scheme: HTTPS`。Kubelet HTTP 探针无法提供 mTLS 客户端证书，因此这类部署需改用 TCP 探针；其仅检查监听器，不验证 TLS 或 Google。

## Kubernetes TLS 网关 {#kubernetes-tls-gateway}

[Gateway 清单](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/gateway.yaml)为[基础 Kubernetes 清单](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml)中的 Deployment 提供 HTTPS Gateway 与 HTTPRoute。先安装 Gateway API CRD 与合规控制器。替换 `your-gateway-class` 和 `maps.example.com`，并在同一命名空间创建名为 `mcp-gmap-tls`、类型为 `kubernetes.io/tls` 的 Secret。示例使用稳定的 Gateway/HTTPRoute v1 字段，以及从 Gateway API v1.2 起支持的 HTTPRoute 超时设置。请确认控制器的超时支持情况。

```sh
kubectl create secret tls mcp-gmap-tls --cert=/secure/fullchain.pem --key=/secure/key.pem
kubectl set env deployment/mcp-gmap-rs \
  MCP_HTTP_ALLOWED_HOSTS='maps.example.com,mcp-gmap-rs:3000,localhost:3000,127.0.0.1:3000'
kubectl apply -f deploy/gateway.yaml
kubectl get gateway,httproute
```

同时将相应的 allowed-host 设置写入 ConfigMap 或部署源文件。需要时通过 `MCP_HTTP_ALLOWED_ORIGINS` 添加实际浏览器来源。路由只公开 `/mcp`，健康与指标保留在私有 Service 中。授权与 MCP 会话/协议头必须原样传递。为选用控制器禁用响应缓冲/缓存，配置流式传输与空闲超时。该路由禁用请求/后端请求期限；应按客户端需求在网关配置连接/空闲限制。

此配置在网关终止 TLS，并通过 HTTP 连接 ClusterIP 后端。如果网关到 Pod 的连接也需加密，请使用原生 TLS，配合控制器支持的 BackendTLSPolicy 或服务网格。通过集群网络策略限制直接访问 Pod/Service。TLS 不会提供 MCP OAuth 登录：程序每个进程使用共享 bearer 令牌和一个 Google 身份。

参阅上游 [Gateway TLS 指南](https://gateway-api.sigs.k8s.io/guides/user-guides/tls/)与 [HTTPRoute 超时指南](https://gateway-api.sigs.k8s.io/guides/http-timeouts/)。

## 限制、重试与期限 {#limits-retries-and-deadlines}

| 变量 | 默认值 | 选项 | 含义 |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | 每次工具调用的 Google HTTP 尝试次数，1–10000，包含重试 |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | 每个 Google 请求的额外尝试，0–5 |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | 整个工具的期限，单位秒，1–3600 |
| `MCP_HTTP_RATE_LIMIT` | `120` | HTTP `--rate-limit` | 每进程每分钟已认证 `/mcp` HTTP 请求数；0 为禁用 |
| `MCP_METRICS_ENABLED` | `false` | HTTP `--metrics [true\|false]` | 启用受保护的 Prometheus `/metrics` |
| `MCP_LOG_FORMAT` | `text` | `--log-format text\|json` | stderr 诊断事件格式 |
| `MCP_TLS_CERT` | 未设置 | HTTP `--tls-cert` | PEM 服务器证书链文件 |
| `MCP_TLS_KEY` | 未设置 | HTTP `--tls-key` | 匹配的 PEM 服务器私钥文件 |
| `MCP_TLS_CLIENT_CA` | 未设置 | HTTP `--tls-client-ca` | PEM CA 包；要求客户端证书 |

这些设置遵循默认值 → 可选 dotenv → 环境变量 → 选项。TLS 证书与私钥必须一起提供。空的环境 TLS 路径会清空文件中的值。`GOOGLE_MAPS_TIMEOUT` 限制每个 Google HTTP 请求，包括信号量排队、重试、退避和读取响应；OAuth 获取使用同一超时值，但单独计时。整个工具期限包含两者。

每次调用有独立的新请求预算，所有并发分支共享。每个进程可同时执行五个 Google API 请求。预算耗尽后，后续尝试会在本地失败。复合工具可能返回顶层成功、逐项包含预算/其他错误的结果，请检查这些字段。此限制针对请求次数而不是费用：不同请求/字段价格不同，一次矩阵请求可能按多个元素计费。

只读 Google 请求会重试 HTTP 429、500、502、503、504 以及网络失败。重试遵守以秒或 HTTP 日期表示的 `Retry-After`，否则使用带抖动的指数退避，且始终受请求期限与调用预算限制。其他 HTTP 错误，包括 400/401/403，不重试。认证失败不会静默切换到 API 密钥模式。将重试设为零可只尝试一次。Google 旧版 JSON 内部错误状态会直接报告而不重试。

HTTP 限流器采用令牌桶，突发容量等于每分钟限额，并持续补充。覆盖全部会话 `/mcp` 的初始化、POST、GET 和 DELETE。未认证请求不消耗令牌。拒绝时返回 HTTP 429 和 `Retry-After`。健康检查与指标抓取不受限流。限流按进程全局生效，而非按用户/IP，且不信任转发的 IP 头。共享部署应在网关增加认证和逐客户端限制。

## 指标与诊断事件 {#metrics-and-diagnostic-events}

```sh
mcp-gmap-rs --log-format json serve http --metrics --rate-limit 120
```

配置了 bearer 令牌时，`GET /metrics` 与 `/mcp` 使用同一令牌。无认证的回环监听器上它也无需认证。指标默认关闭；启用原生 mTLS 时也受其保护。只应向受信任的抓取端开放。Prometheus 文本端点导出：

- `mcp_gmap_tool_calls_total{tool,outcome}`：已完成的工具调用；
- `mcp_gmap_tool_duration_seconds{tool,outcome}`：耗时直方图；
- `mcp_gmap_google_requests_total{api,status}`：Google HTTP 尝试，包含重试；
- `mcp_gmap_http_rate_limited_total`：被拒绝的 HTTP 请求。

标签只包含内置工具名称、API 主机名、HTTP 状态码和固定结果值，绝不包含地址、坐标、凭据、令牌或会话 ID。含逐项错误的复合结果仍可能记为顶层调用成功；需检查结果及上游状态指标。重启后指标清零。

设置 `RUST_LOG=info` 可查看带名称、成功状态和耗时的工具完成事件，不包括参数和 Google 响应正文。监听器启动地址单独打印到 stderr。stdout 始终保留给 CLI JSON 或 MCP 流量。

## Doctor 诊断 {#doctor}

```sh
mcp-gmap-rs doctor
# Explicit opt-in: makes potentially billable requests.
mcp-gmap-rs doctor --live
# Check only the API dependencies of selected tools:
mcp-gmap-rs --enabled-tools maps_pollen,maps_area_insights doctor --live
```

不带 `--live` 时，doctor 检查本地凭据配置、HTTP 设置及 TLS 文件/密钥匹配，不发送网络请求。它不能证明密钥有效、IAM/API 已配置、证书信任/主机名/有效期或 ADC 可用。ADC 发现会在实际请求前报告为 `not_tested`。doctor 从不打印凭据值或 JSON。报告以 JSON 写入 stdout；检查失败时退出状态为 1。解析 `checks[].status`：`pass`、`fail` 或 `not_tested`。

在线模式按已启用工具使用的每个不同 Google API 执行一次代表性工具调用，遵循常规重试/期限/预算设置，避免昂贵的复合操作。Routes 使用精确坐标，Roads 使用最近道路查询。探测使用巴黎的公开地标，而非用户地址。成功只表示 Google 接受该次探测，不保证全部操作或地区可用。

## 关闭与配置变更 {#shutdown-and-configuration-changes}

Ctrl-C 和 SIGTERM 会关闭 HTTP 服务器与活动 MCP 会话。Kubernetes 示例给予进程 30 秒。其他应用配置，包括环境注入的 Secret 变更，均需重启；仅原生 TLS 证书/私钥/客户端 CA 文件支持 SIGHUP 重载。会话和指标存储在内存，重启即丢失。运行单副本或配置会话亲和性；所属进程退出后客户端必须重新连接。

容器权限、探针和运行时凭据挂载见[部署](./deployment)，所有设置与选项见[配置参考](../reference/configuration)。
