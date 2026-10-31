# 配置 {#configuration}

CLI 与两种 MCP 传输共享同一套启动配置。本地开发可选择文件，容器可注入环境变量，需要明确覆盖单个进程时使用选项。

## 优先级 {#precedence}

各层按以下顺序应用，后面的层优先：

**内置默认值 → 可选 dotenv 文件 → 进程环境变量 → CLI 选项**

默认文件为当前工作目录下的 `.env`，不会搜索父目录。允许默认文件不存在；但文件不可读或格式错误时，启动会失败且不会打印文件内容。

```sh
mcp-gmap-rs --env-file /etc/mcp-gmap/runtime.env serve http
mcp-gmap-rs --no-env-file tools
```

`MCP_ENV_FILE` 也可选择文件，但 `--env-file` 优先。显式选择的文件必须存在。`--no-env-file` 会禁用所有文件加载，包括 `MCP_ENV_FILE`，且不能与 `--env-file` 同时使用。解析 dotenv 不会修改进程环境。

## 覆盖挂载的配置 {#override-a-mounted-configuration}

```dotenv
GOOGLE_MAPS_LANGUAGE=en
GOOGLE_MAPS_TIMEOUT=30
GOOGLE_MAPS_ENABLED_TOOLS=maps_search_places,maps_place_details
```

挂载上述文件后，可用环境变量更改语言，并用选项更改超时：

```sh
GOOGLE_MAPS_LANGUAGE=fr mcp-gmap-rs \
  --env-file /etc/mcp-gmap/runtime.env --timeout 15 tools
```

重复的 `--allowed-host` 和 `--allowed-origin` 选项会整体替换相应的文件/环境变量列表。显式空环境值也会覆盖文件：可清空可选令牌或列表，但空超时值无效。

## 控制可用工具 {#control-the-tool-surface}

`GOOGLE_MAPS_ENABLED_TOOLS` 或 `--enabled-tools` 接受逗号分隔的逻辑名称，例如 `maps_weather,maps_air_quality`，也可用 `*` 启用全部工具。未知名称导致启动失败。被禁用的工具不会出现在发现结果中，也不能调用。工具发现和模式查看无需凭据，但实际 API 操作需要相应的 Google 凭据。

应用设置在启动时读取。修改环境变量、凭据 JSON 或 Kubernetes Secret/ConfigMap 配置后需重启。原生 TLS 证书、私钥和客户端 CA 文件除外：它们支持 [SIGHUP 重新加载](./production#native-tls-and-mutual-tls)。

所有 21 个设置、默认值、选项和范围见[完整配置参考](../reference/configuration)。网站语言选择不会改变控制 Google 响应文本的 `GOOGLE_MAPS_LANGUAGE`。
