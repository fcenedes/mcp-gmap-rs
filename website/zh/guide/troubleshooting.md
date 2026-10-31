# 故障排查 {#troubleshooting}

先确定失败的是哪一层：本地配置、MCP 客户端到服务器的连接，还是到 Google 的出站请求。修正对应层后，重新执行受影响操作。

## 从离线检查开始 {#start-offline}

```sh
mcp-gmap-rs --version
mcp-gmap-rs tools
mcp-gmap-rs doctor
```

`doctor` 即使检查失败也会输出 JSON。检查退出状态与 `checks[].status`：`pass`、`fail` 或 `not_tested`。离线检查不能证明 IAM、API 启用、结算、证书信任/有效期或 ADC 访问正常。`/health` 只证明进程存活。

若格式错误的本地 dotenv 阻止工具发现，可用 `--no-env-file tools` 跳过文件。但这可能改变所选工具集，因此不能据此证明目标文件配置正确。

## 解释常见故障 {#interpret-common-failures}

| 现象 | 下一步检查 |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | 选择了 API 密钥模式，或操作涉及仅支持密钥的端点。 |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | 为 OAuth 设置消费项目 ID/编号。 |
| Google 403 / API 未启用 | 在凭据的消费项目启用准确 API，并检查结算与密钥限制。 |
| `serviceusage.services.use` 被拒绝 | 在配额项目向服务账号授予 `roles/serviceusage.serviceUsageConsumer`，等待 IAM 传播后重试。 |
| API 密钥 IP 被拒绝 | 允许服务器或集群的公网出口 IP，而不是私有 Pod IP。 |
| MCP HTTP 401 | 发送服务器的 bearer 令牌；Google 访问令牌验证另一条连接。 |
| Host/Origin 被拒绝 | 精确匹配配置的 Host 和浏览器来源，包括相应端口。 |
| MCP 返回 HTTP 429 | 进程级限流器已耗尽；遵守 `Retry-After` 并检查客户端并发。 |
| OAuth 超时 | 检查令牌/STS/IAM 或元数据连接、系统时间与每请求超时。 |
| TLS 连接失败 | 分别检查证书链、主机名、信任与可选 mTLS 客户端证书，不要与 HTTP 授权混淆。 |
| 退出 0 但数据不完整 | 检查矩阵/复合结果的逐项错误，而不只是进程退出状态。 |

## 主动验证在线访问 {#verify-live-access-deliberately}

在线探测可能产生费用。将范围限定在所需的 API 依赖：

```sh
mcp-gmap-rs --enabled-tools maps_weather doctor --live
```

探测通过不保证每个地区或操作都可用。Google Cloud 设置更改后，应重新执行原来失败的调用，而不是从控制台设置推断成功。IAM 与凭据来源见[身份验证](./authentication)，优先级见[配置](./configuration)，预算、期限与指标见[生产运维](./production)。切勿在问题报告中打印凭据 JSON 或完整环境变量。
