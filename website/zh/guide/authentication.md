# 身份验证 {#authentication}

Google 出站凭据与 MCP HTTP 客户端使用的凭据需分别配置。默认采用 API 密钥模式；仅设置服务账号路径不会启用 OAuth。

## API 密钥与 OAuth 的适用范围 {#api-key-and-oauth-coverage}

| 端点类别 | API 密钥 | 本程序中的 OAuth |
| --- | --- | --- |
| Places (New)、Routes、Weather、Air Quality、Address Validation、Places Aggregate | 是 | 是 |
| Geocoding v3、Elevation、Time Zone、Maps Static、Pollen、Roads | 是 | 否 |

即使主操作支持 OAuth，命名地点解析和复合调用仍可能需要密钥。使用精确坐标可避免不必要的地理编码。OAuth 失败绝不会静默回退到 API 密钥。请参阅 [Google Cloud 配置](./google-cloud)，启用准确的服务并限制密钥。

## 服务账号 OAuth {#service-account-oauth}

OAuth 模式下，Places (New)、Routes、Weather、Air Quality、Address Validation 和 Places Aggregate 请求会携带 OAuth **访问令牌**及 `X-Goog-User-Project` 头。本项目申请这些 API 文档指定的 `https://www.googleapis.com/auth/cloud-platform` 范围。Google 的 Rust 认证库负责获取、缓存与刷新令牌。`GOOGLE_MAPS_TIMEOUT` 也限制凭据初始化和令牌获取的时间。服务账号流程无需手动粘贴访问令牌，也不需要应用 OAuth 同意屏幕。参阅 [Maps 服务器端 OAuth 指南](https://developers.google.com/maps/api-security-best-practices#use-oauth-for-server-side-apps)。

创建专用服务账号，并授予它在 Maps 结算/配额项目中使用服务的权限：

```sh
export PROJECT_ID='your-project-id'
export SA_EMAIL="mcp-gmap@${PROJECT_ID}.iam.gserviceaccount.com"
gcloud iam service-accounts create mcp-gmap --project="$PROJECT_ID" \
  --display-name='MCP Google Maps'
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:${SA_EMAIL}" \
  --role='roles/serviceusage.serviceUsageConsumer'
```

Service Usage Consumer 角色包含配额和结算所需的 `serviceusage.services.use` 权限，但不授予启用服务或创建密钥的能力；这些配置步骤需由另行授权的管理员执行。按照 [Google Cloud 配置指南](./google-cloud)在配额项目中启用 Maps API。参阅 [Service Usage 权限](https://docs.cloud.google.com/service-usage/docs/access-control)。

从以下凭据来源中选择一种。CLI、stdio 和 HTTP 使用相同设置，并遵循通常的文件 → 环境变量 → 选项优先级。

### A. 挂载服务账号 JSON 密钥 {#a-mounted-service-account-json-key}

组织允许 JSON 密钥时，打开 **IAM & Admin → Service Accounts**，选择账号，再选择 **Keys → Add key → Create new key → JSON**。将文件保存在源代码检出目录之外。也可使用以下命令：

```sh
mkdir -p "$HOME/.config/mcp-gmap-rs"
gcloud iam service-accounts keys create \
  "$HOME/.config/mcp-gmap-rs/service-account.json" \
  --iam-account="$SA_EMAIL" --project="$PROJECT_ID"
chmod 600 "$HOME/.config/mcp-gmap-rs/service-account.json"
export GOOGLE_APPLICATION_CREDENTIALS="$HOME/.config/mcp-gmap-rs/service-account.json"
export GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
export GOOGLE_MAPS_AUTH_MODE=oauth
```

对于挂载文件，等效 `.env` 如下：

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
# Optional, for Geocoding v3 / Elevation / Time Zone / Static Maps / Pollen / Roads:
# GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

仅在使用可读取的 JSON 文件时设置凭据路径；不要设为空字符串，也不要将 JSON 内容作为路径。相对路径按工作目录解析。文件值直接使用，不会修改进程环境。加载器也接受 ADC 的 `external_account`、`impersonated_service_account` 和 `authorized_user` JSON 配置。修改凭据文件后须重启进程。

服务账号密钥是长期凭据。条件允许时优先使用以下无密钥方案；不要将 JSON 密钥放入 Git 或镜像层，并定期轮换或删除未使用的密钥。Google 的[密钥创建与组织政策文档](https://docs.cloud.google.com/iam/docs/keys-create-delete)说明了相关要求。

### B. 无 JSON 密钥的 Application Default Credentials {#b-application-default-credentials-without-a-json-key}

进程环境和所选 dotenv 文件或 Kubernetes Secret/ConfigMap 中都应不设置 `GOOGLE_APPLICATION_CREDENTIALS`。单独执行 Shell `unset` 不会删除文件层的值；应移除该项，而不是设置空路径。然后配置：

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
```

官方认证库使用 [Application Default Credentials (ADC)](https://docs.cloud.google.com/docs/authentication/application-default-credentials)：存在本地 ADC 凭据时使用本地凭据，否则通过元数据服务器使用附加的工作负载/服务账号。凭据发现采用延迟加载：`tools`、`schema` 和帮助命令无需获取令牌。OAuth API 调用则需要配额项目和有效凭据。

本地开发使用服务账号模拟时，先启用 IAM Credentials API，并授予开发者账号对服务账号的 Token Creator 权限，再创建本地 ADC 凭据：

```sh
gcloud services enable iamcredentials.googleapis.com --project="$PROJECT_ID"
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
  --project="$PROJECT_ID" --member='user:you@example.com' \
  --role='roles/iam.serviceAccountTokenCreator'
unset GOOGLE_APPLICATION_CREDENTIALS
gcloud auth application-default login --impersonate-service-account="$SA_EMAIL"
export GOOGLE_MAPS_AUTH_MODE=oauth
export GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
```

此方式使用短期服务账号令牌调用 Maps，无需下载私钥。参阅[服务账号模拟](https://docs.cloud.google.com/iam/docs/service-account-impersonation)。

### C. Kubernetes 与 GKE 工作负载身份 {#c-kubernetes-and-gke-workload-identity}

通用 Kubernetes 环境中，将凭据 JSON 作为 Secret 文件挂载，并配置方案 A 的三个 OAuth 变量。文件必须可由容器使用的 UID 10001 读取。以只读方式挂载，将 `MCP_HTTP_TOKEN` 保存在单独的运行时 Secret 中。[容器指南](./deployment)介绍基础 Deployment。

GKE 上优先使用 [Workload Identity Federation for GKE](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/workload-identity)，将 Kubernetes ServiceAccount 关联到 IAM 服务账号。先在集群/节点池中启用该功能（Autopilot 已启用），然后执行：

```sh
gcloud services enable iamcredentials.googleapis.com --project="$PROJECT_ID"
kubectl create serviceaccount mcp-gmap-rs --namespace default
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
  --project="$PROJECT_ID" --role='roles/iam.workloadIdentityUser' \
  --member="serviceAccount:${PROJECT_ID}.svc.id.goog[default/mcp-gmap-rs]"
kubectl annotate serviceaccount mcp-gmap-rs --namespace default \
  "iam.gke.io/gcp-service-account=${SA_EMAIL}"
kubectl patch deployment mcp-gmap-rs --namespace default --type=merge \
  -p '{"spec":{"template":{"spec":{"serviceAccountName":"mcp-gmap-rs"}}}}'
kubectl set env deployment/mcp-gmap-rs --namespace default \
  GOOGLE_MAPS_AUTH_MODE=oauth GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
```

上述命令假定集群和 IAM 服务账号都位于 `PROJECT_ID`，且基础 Deployment 位于 `default`。根据安装调整项目/命名空间。此元数据流程不要设置 `GOOGLE_APPLICATION_CREDENTIALS`。Pod 通过 GKE 元数据服务器获取访问令牌。GKE 以外，可结合 `external_account` 凭据配置与提供方投射的身份令牌，使用 Google Workload Identity Federation。

仅使用 OAuth 时，从运行时 Secret 删除 Google API 密钥，并只启用调用始终落在 OAuth 支持 API 内的工具/输入。使用表中的仅密钥端点或依赖它们的复合流程时保留密钥。支持的 API 上 OAuth 失败会显式报错，绝不静默降级到密钥。

## 验证与故障排查 {#verify-and-troubleshoot}

```sh
mcp-gmap-rs --auth-mode oauth search-places --query 'coffee in Paris'
mcp-gmap-rs --auth-mode oauth weather --latitude 48.8584 --longitude 2.2945
```

这些是可能计费的真实调用。测试时检查有效模式和配额项目；OAuth 使用 `GOOGLE_CLOUD_QUOTA_PROJECT` 结算，而密钥认证调用使用密钥所属项目。两个项目可能不同。

使用 `mcp-gmap-rs doctor` 做离线配置检查，或使用 `doctor --live` 显式执行可能计费的 API 探测。[生产指南](./production)说明报告与工具过滤。

导出 OAuth 变量后，可执行六项显式启用的在线验收测试：

```sh
cargo test --locked --test live live_oauth_ -- --ignored --test-threads=1
```

这些测试会从每个子进程中显式移除 API 密钥，因此通过即表示实际使用了 OAuth。它们覆盖 Places、Routes、Weather、Air Quality、Address Validation 和 Places Aggregate，需启用相应 API。测试不会加载 `.env`；请导出 OAuth 设置或在已配置的工作负载中运行。普通测试/CI 使用本地令牌/API 测试夹具，不需要服务账号，也不会连接 Google。

| 现象 | 检查项 |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | 所选操作调用了仅支持密钥的端点，或选择了 API 密钥模式。 |
| OAuth 认证失败 | 检查凭据路径/类型、密钥有效性、工作负载身份/模拟绑定、所用 IAM Credentials API、令牌端点连接与系统时间。应用错误不会包含凭据内容或令牌。 |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | 为 OAuth 调用设置 Maps 消费项目 ID/编号。 |
| Google `403` / API 未启用 | 在消费项目中启用准确服务，并确认结算与 Service Usage 权限。 |
| API 密钥 IP 被拒绝 | 限制应匹配服务器或集群的公网出口 IP。 |
| 令牌超时 | 允许访问 Google 令牌/STS/IAM 端点或工作负载元数据服务器，并检查 `GOOGLE_MAPS_TIMEOUT`。 |
| 此 MCP 服务器返回 HTTP `401` | 发送其 `MCP_HTTP_TOKEN`；Google 服务账号令牌验证的是独立的出站连接。 |


## 新增服务与限制 {#expanded-services}

Solar 和 Map Tiles 也支持 OAuth。Datasets 与 Route Optimization 需要 OAuth 及对应操作的 IAM 权限。Street View Publish 必须使用明确指定的 authorized_user 凭据文件，并获得 streetviewpublish 范围的用户同意。

| 服务 | API | 身份验证 |
| --- | --- | --- |
| Geolocation API | `geolocation.googleapis.com` | API 密钥 |
| Solar API | `solar.googleapis.com` | API 密钥 / OAuth |
| Aerial View API | `aerialview.googleapis.com` | API 密钥 |
| Map Tiles API | `tile.googleapis.com` | API 密钥 / OAuth |
| Street View Static API | `street-view-image-backend.googleapis.com` | API 密钥 |
| Maps Platform Datasets API | `mapsplatformdatasets.googleapis.com` | OAuth / IAM |
| Route Optimization API | `routeoptimization.googleapis.com` | OAuth / IAM |
| Street View Publish API | `streetviewpublish.googleapis.com` | 需用户同意的 OAuth；不接受服务账号 |

Speed Limits 需要受限的 Asset Tracking 授权。Aerial View 仅支持特定美国地址，禁止下载或缓存视频。Map Tiles 仅供可视化展示，禁止机器分析和数据提取。Weather 分钟预报和地图工具属于实验功能，缓存最多一小时，并有额外使用限制。各工具参考页列出详细条款、署名要求和限制。

[Google: Speed Limits](https://developers.google.com/maps/documentation/roads/speed-limits), [Map Tiles](https://developers.google.com/maps/documentation/tile/policies), [Aerial View](https://developers.google.com/maps/documentation/aerial-view/policies), [Street View Publish](https://developers.google.com/streetview/publish/authorizing).
