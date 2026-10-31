# 命令行接口 {#command-line-interface}

CLI 适合单次查询、Shell 自动化和结构化子进程调用。每个工具的输入模式和实现与 MCP 完全共享。

## 查看已安装版本的接口 {#discover-the-installed-interface}

```sh
mcp-gmap-rs --version
mcp-gmap-rs --help
mcp-gmap-rs tools
mcp-gmap-rs schema search-nearby
mcp-gmap-rs search-nearby --help
```

`tools` 返回已启用工具及其模式。应核对已安装版本，不要假定最新版手册的所有功能都可用。不带子命令运行会启动 stdio 服务器，因此单次自动化调用必须指定命令。

## 选项或共享 JSON {#flags-or-shared-json}

工具命令使用 kebab-case 选项；对象和数组值使用 JSON。布尔选项接受 `true` 或 `false`，不带值的布尔选项表示 `true`。

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now
```

通用 `call` 接口接受逻辑 MCP 工具名称或 CLI 命令名称，并使用与 MCP 完全相同的参数对象：

```sh
mcp-gmap-rs call maps_geocode --args '{"address":"Paris, France"}'
printf '%s' '{"address":"Paris, France"}' | mcp-gmap-rs call geocode --args -
```

以程序方式启动子进程时，应将序列化 JSON 作为独立参数传递。避免把不可信文本插入 Shell 命令字符串。顶层参数对应选项；嵌套字段保留在 JSON 对象中。

## 输出与失败处理 {#output-and-failure-handling}

普通工具结果以 JSON 输出到 stdout。错误输出到 stderr，并产生非零退出状态。`doctor` 检查失败时也会把 JSON 报告写入 stdout。批量、矩阵和复合结果即使进程成功退出也可能有逐项错误：将操作视为完整成功前，请检查这些字段。

```sh
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --output place.json
mcp-gmap-rs static-map --center Paris --zoom 12 --output paris.png
```

`--output` 对普通工具保存 JSON，对静态地图保存原始 PNG。不指定时，CLI 静态地图输出包含 base64 数据。时间通常采用显式时区的 RFC 3339；Time Zone 工具的可选 `timestamp` 使用 Unix **毫秒**。参阅[全部工具和示例](../reference/tools)。
