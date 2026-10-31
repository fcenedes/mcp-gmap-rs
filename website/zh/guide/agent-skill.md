# 智能体技能 {#agent-skill}

可移植的 `mcp-gmap` 技能指导智能体选择工具、安全调用 MCP 或 CLI、检查部分失败，并保留 Google 归属信息。它补充的是单独配置的可执行文件或 MCP 连接。

## 从源代码或发布包安装 {#install-from-source-or-a-release}

源代码包和每个发布压缩包都包含[技能目录](https://github.com/fcenedes/mcp-gmap-rs/tree/main/skills/mcp-gmap)。在检出目录或解压目录中，可为 Codex 安装：

```sh
mkdir -p "${CODEX_HOME:-$HOME/.codex}/skills"
cp -R skills/mcp-gmap "${CODEX_HOME:-$HOME/.codex}/skills/"
```

使用 `$mcp-gmap` 调用，例如：

> 使用 $mcp-gmap 查找 Gare de Lyon 附近的咖啡馆，比较步行路线，并保留来源链接。

其他智能体可将同一目录放到支持的技能路径，或加载 `SKILL.md` 及其引用文档。指令主体与智能体无关；`agents/openai.yaml` 提供可选的 Codex UI 元数据。请在智能体的 PATH 安装可用可执行文件，或配置其 [MCP 连接](./mcp)。

## 技能提供的能力 {#what-the-skill-adds}

技能从工具发现开始，不假定旧版已安装版本支持最新工具目录。它优先使用精确坐标与已返回的地点 ID，保持搜索范围较小，并为地址验证、聚合统计或沿途搜索选择专用工具，而不是从无关结果中近似推断。

它将凭据视为进程/部署配置，不作为工具参数。它区分入站 MCP 授权与出站 Google 访问，优先采用离线诊断。在线诊断探测可能计费。即使顶层调用成功，也必须检查复合输出中的嵌套错误。

[工具选择参考](https://github.com/fcenedes/mcp-gmap-rs/blob/main/skills/mcp-gmap/references/tool-selection.md)解释决策规则，[CLI 参考](https://github.com/fcenedes/mcp-gmap-rs/blob/main/skills/mcp-gmap/references/cli.md)介绍结构化调用与访问诊断。完整工作流见本手册[实用示例](./recipes)，所有参数见[工具参考](../reference/tools)。
