# Google Cloud 配置 {#google-cloud-setup}

本程序使用 API 密钥或 OAuth 2.0 凭据验证**发送到 Google 的出站调用**。`MCP_HTTP_TOKEN` 独立验证连接 MCP HTTP 服务器的客户端；选择 Google OAuth 不会替代它。

## 启用服务与结算 {#enable-services-and-billing}

在 [Google Cloud 控制台](https://console.cloud.google.com/)选择或创建项目，然后在 **Billing** 下关联有效的结算账号。在 **APIs & Services → Library** 中启用所需 API。启用 API 与在 API 密钥限制中允许该 API 是两个独立步骤。调用可能产生费用。

下表列出本版本使用的端点：

| Google API | 需要启用的服务 | 本项目使用的认证方式 |
| --- | --- | --- |
| [Places API (New)](https://developers.google.com/maps/documentation/places/web-service/get-api-key) | `places.googleapis.com` | API 密钥或 OAuth |
| [Routes API](https://developers.google.com/maps/documentation/routes/get-api-key) | `routes.googleapis.com` | API 密钥或 OAuth |
| [Weather API](https://developers.google.com/maps/documentation/weather/get-api-key) | `weather.googleapis.com` | API 密钥或 OAuth |
| [Air Quality API](https://developers.google.com/maps/documentation/air-quality/get-api-key) | `airquality.googleapis.com` | API 密钥或 OAuth |
| [Address Validation API](https://developers.google.com/maps/documentation/address-validation/get-api-key) | `addressvalidation.googleapis.com` | API 密钥或 OAuth |
| [Places Aggregate API](https://developers.google.com/maps/documentation/places-aggregate/get-api-key) | `areainsights.googleapis.com` | API 密钥或 OAuth |
| [Pollen API](https://developers.google.com/maps/documentation/pollen/get-api-key) | `pollen.googleapis.com` | API 密钥 |
| [Roads API](https://developers.google.com/maps/documentation/roads/get-api-key) | `roads.googleapis.com` | API 密钥 |
| [Geocoding API v3](https://developers.google.com/maps/documentation/geocoding/guides-v3/get-api-key) | `geocoding-backend.googleapis.com` | API 密钥 |
| [Elevation API](https://developers.google.com/maps/documentation/elevation/get-api-key) | `elevation-backend.googleapis.com` | API 密钥 |
| [Time Zone API](https://developers.google.com/maps/documentation/timezone/get-api-key) | `timezone-backend.googleapis.com` | API 密钥 |
| [Maps Static API](https://developers.google.com/maps/documentation/maps-static/get-api-key) | `static-maps-backend.googleapis.com` | API 密钥 |

用管理员的 `gcloud` 登录一次启用全部 12 项服务：

```sh
export PROJECT_ID='your-project-id'
gcloud services enable --project="$PROJECT_ID" \
  places.googleapis.com routes.googleapis.com \
  weather.googleapis.com airquality.googleapis.com \
  geocoding-backend.googleapis.com elevation-backend.googleapis.com \
  timezone-backend.googleapis.com static-maps-backend.googleapis.com \
  addressvalidation.googleapis.com areainsights.googleapis.com \
  pollen.googleapis.com roads.googleapis.com
```

CLI 的 `directions` 和 `distance-matrix` 工具使用 **Routes API**，因此无需旧版 Directions 和 Distance Matrix API。本项目的 Geocoding 仍使用 v3；[Google Geocoding v4 的 OAuth 支持](https://developers.google.com/maps/documentation/geocoding/geocoding-v4-overview)不会改变这些 v3 调用的认证方式。复合工具可能调用多个 API。即使主操作使用 Places 或 Routes，按名称解析地点或回退查询仍可能需要 Geocoding。需要避免此依赖时请使用坐标，或为仅支持密钥的端点保留受限制的密钥。

## 创建受限制的 API 密钥 {#create-a-restricted-api-key}

1. 在所选项目中打开 [Google Maps Platform → Keys & Credentials](https://console.cloud.google.com/google/maps-apis/credentials)，选择 **Create credentials → API key**。
2. 仅选择此实例会使用的表中 API。对于已有密钥，编辑其 API 限制并保存。
3. 此应用在服务器端运行，请将应用限制设为 **IP addresses**，允许 Google 实际看到的公网出口 IP。Kubernetes Pod 或私有 IP 通常不是公共来源地址；固定 Cloud NAT 或其他出口地址可让限制更可预测。
4. 将密钥存入被忽略的 `.env`、注入环境变量或 Kubernetes Secret。参阅 [Google 密钥限制指南](https://developers.google.com/maps/api-security-best-practices#restrict-your-api-keys)。

`.env` 示例：

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

默认模式为 `api-key`。仅提供服务账号路径不会更改认证模式。工具发现不会连接 Google；请用实际调用验证项目结算、API 启用情况与限制：

```sh
mcp-gmap-rs geocode --address 'Paris, France'
```

## 后续步骤 {#next-steps}

服务账号 JSON、ADC、本地模拟和 GKE 身份请参阅[身份验证](./authentication)，然后选择 [CLI](./cli) 或 [MCP](./mcp)。[工具参考](../reference/tools)列出了各工具的凭据要求。


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
