# 部署 {#deployment}

Google API 密钥创建、API 启用和服务账号 OAuth 请参阅 [Google Cloud 配置指南](./google-cloud)。同样的分层配置适用于容器。OAuth 可使用挂载的 JSON 凭据文件或工作负载的 ADC 身份；凭据在运行时提供。

下载或克隆[源代码仓库](https://github.com/fcenedes/mcp-gmap-rs)，从仓库根目录构建：

```sh
docker build -t mcp-gmap-rs:local .
```

多阶段 Dockerfile 使用 Rust 1.94.1 和锁文件编译。运行时包含可执行文件、Debian CA 证书和许可证，以 UID/GID 10001 运行，并支持只读根文件系统。允许列表限定的构建上下文排除本地 `.env`、Git 数据与构建输出。构建不需要凭据。BuildKit 会缓存 Cargo 下载和构建输出，以便后续复用。

同一 Dockerfile 可在 Linux amd64 和 arm64 上原生构建。配置好 Buildx 构建器后，可同时构建两种架构并推送到自己的仓库：

```sh
docker buildx build --platform linux/amd64,linux/arm64 \
  -t registry.example.com/your-team/mcp-gmap-rs:YOUR_VERSION --push .
```

项目没有自动配置容器仓库发布。将镜像名称和版本替换为自己构建或发布的镜像。

## 运行 HTTP、CLI 或 stdio {#run-http-cli-or-stdio}

镜像默认在 `0.0.0.0:3000` 上执行 `serve http`。运行时提供 bearer 令牌，例如使用 `.env.example` 所示格式的 `.env`：

```sh
docker run --rm --read-only --cap-drop ALL --security-opt no-new-privileges \
  --env-file .env -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

由于容器监听非回环地址，`.env` 中必须有 `MCP_HTTP_TOKEN`。Google 调用根据启用的工具使用 `GOOGLE_MAPS_API_KEY`，或配置指南中的 OAuth 设置与凭据挂载。客户端访问 `http://127.0.0.1:3000/mcp`，并发送 `Authorization: Bearer <token>`。`GET /health` 无需令牌，也不会连接 Google。

Docker 的 `--env-file` 会注入环境变量。如需测试应用自身的文件加载器，可挂载可读文件并选择它：

```sh
docker run --rm --read-only \
  --mount type=bind,src="$(pwd)/.env",dst=/etc/mcp-gmap/runtime.env,readonly \
  -e GOOGLE_MAPS_LANGUAGE=fr -p 127.0.0.1:3000:3000 \
  mcp-gmap-rs:local --env-file /etc/mcp-gmap/runtime.env serve http
```

环境变量覆盖挂载文件，CLI 选项再覆盖两者。镜像的 `MCP_HTTP_LISTEN=0.0.0.0:3000` 也是环境变量；需要时使用 `-e MCP_HTTP_LISTEN=...` 或 `--listen` 修改。仅使用环境变量时，传入 `--no-env-file serve http`。

需要 CLI 或 stdio 时覆盖镜像命令：

```sh
docker run --rm mcp-gmap-rs:local tools
docker run --rm --env-file .env mcp-gmap-rs:local geocode --address Paris
docker run --rm -i --env-file .env mcp-gmap-rs:local serve stdio
```

stdio 使用 `-i`，不要分配 TTY。CLI 写输出文件时，挂载 UID 10001 可写的目录，并在其中指定 `--output`。

## Kubernetes {#kubernetes}

[基础 Kubernetes 清单](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml)通过 ConfigMap 提供非敏感设置，通过预先存在的 `mcp-gmap-rs` Secret 提供凭据。从包含 `GOOGLE_MAPS_API_KEY` 和 `MCP_HTTP_TOKEN` 的本地文件创建 Secret（该文件不提交到 Git）：

```sh
kubectl create secret generic mcp-gmap-rs --from-env-file=/secure/credentials.env
```

将 Deployment 中的 `mcp-gmap-rs:local` 替换为自己的仓库镜像，或将本地镜像载入开发集群，然后应用示例：

```sh
kubectl apply -f deploy/kubernetes.yaml
kubectl rollout status deployment/mcp-gmap-rs
kubectl port-forward service/mcp-gmap-rs 3000:3000
```

示例使用 `default` 命名空间。使用其他命名空间或入口时，调整服务 DNS 主机名。ClusterIP Service 仅在集群内可见；远程客户端应使用 TLS ingress/代理，转发 `MCP_HTTP_ALLOWED_HOSTS` 中允许的 Host、授权头及 MCP/SSE 头。需要时显式配置浏览器来源，包括端口。

Deployment 通过 `envFrom` 注入全部配置，因此使用 `--no-env-file`。另一种方式是挂载包含 dotenv 文件的 Secret，移除此选项，并设置 `args: ["--env-file", "/etc/mcp-gmap/runtime.env", "serve", "http"]`。投射文件必须可由 UID 10001 读取；Secret 或 ConfigMap 注入的环境变量仍会覆盖它。参阅 Kubernetes 的[环境变量注入](https://kubernetes.io/docs/tasks/inject-data-application/define-environment-variable-container/)和 [Secret 卷](https://kubernetes.io/docs/concepts/configuration/secret/)指南。

应用在启动时读取配置。修改 Secret/ConfigMap 值后重启 Pod；原生 TLS 文件也可通过 SIGHUP 重新加载。探针使用 `/health` 检查 HTTP 可用性；Pod 健康不代表 Google API 已启用或凭据已获授权。Google 调用需要出站 HTTPS 与可用 DNS。API 密钥限制必须允许集群的公网出口地址；OAuth 需要连接令牌/身份端点。

默认 MCP 会话存储在内存中，因此保持单副本。多副本需要会话亲和性，确保每个会话到达其所属 Pod；滚动重启会丢弃会话，客户端须重新连接。示例中的资源请求/限制只是起始值，应按工作负载调整。

SIGTERM 会触发 MCP/HTTP 的优雅关闭；示例中 Kubernetes 留出 30 秒。镜像同时移除全部 capabilities，并禁止提权。

## 生产 HTTPS、指标与限制 {#production-https-metrics-and-limits}

[生产指南](./production)涵盖原生 TLS/mTLS、证书重载、可选的 `deploy/gateway.yaml` HTTPS Gateway、指标、请求预算、HTTP 限流、JSON 日志及离线/在线 `doctor` 检查。可在镜像名称后传入相同 TLS 选项，或通过只读 PEM 挂载注入 `MCP_TLS_CERT`、`MCP_TLS_KEY` 和可选 `MCP_TLS_CLIENT_CA`。所有真实密钥都应保留在镜像之外。默认 ConfigMap 配置请求限制与 JSON 日志；指标只有显式启用才开启。默认明文 Service 面向私有的网关到 Pod 链路。

## 容器中的 OAuth 凭据 {#oauth-credentials-in-a-container}

服务账号 JSON 文件需只读挂载，并设置容器内的路径，而不是宿主机路径：

```sh
docker run --rm --read-only --env-file /secure/oauth.env \
  --mount type=bind,src=/secure/service-account.json,dst=/credentials/service-account.json,readonly \
  -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

OAuth 环境文件设置 `GOOGLE_MAPS_AUTH_MODE=oauth`、`GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json`、`GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id` 和 `MCP_HTTP_TOKEN`。UID 10001 必须可读取挂载文件。仅在使用只支持密钥端点的工具时保留 API 密钥。GKE 上使用[工作负载身份步骤](./authentication#c-kubernetes-and-gke-workload-identity)，避免挂载长期密钥。
