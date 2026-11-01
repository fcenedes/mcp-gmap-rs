# 快速开始 {#getting-started}

版本 0.4.0 在一个可执行文件中提供 42 个工具，可通过终端或 MCP 客户端使用。本手册介绍安装、Google 访问配置以及完整的工具接口。

在仓库工作目录中构建：

```sh
cargo install --path . --locked
```

## 安装 {#install}

可从 [GitHub Releases](https://github.com/fcenedes/mcp-gmap-rs/releases) 下载 Linux x86_64、macOS Apple Silicon 和 macOS Intel 的预编译文件。选择适合机器的压缩包，对照 `SHA256SUMS` 验证校验和，解压后将 `mcp-gmap-rs` 放到 `PATH` 中。Linux 文件要求 glibc 2.35 或更新版本。macOS 文件未签名，也未经公证。

使用 Rust 1.88 或更新版本时，可从 crates.io 安装：

```sh
cargo install mcp-gmap-rs --version 0.4.0 --locked
mcp-gmap-rs --version
```

可执行文件构建完成后，无需 Rust、Python 或 JavaScript 运行时。宿主机需要 CA 证书、DNS 以及到 Google 的出站 HTTPS 连接。

## 配置访问 {#configure-access}

[创建受限制的密钥并启用服务](./google-cloud)，或配置[服务账号 OAuth](./authentication)。将凭据保存在工作目录下、已被 Git 忽略的 `.env` 文件中：

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

环境变量会覆盖文件中的值。不要将凭据放入版本控制或镜像层。注意工作目录：程序不会向父目录查找 `.env`。

## 发现、检查与调用 {#discover-check-call}

工具发现和离线诊断不会向 Google 发送请求：

```sh
mcp-gmap-rs tools
mcp-gmap-rs schema search-places
mcp-gmap-rs doctor
```

然后执行首次可能产生费用的调用：

```sh
mcp-gmap-rs search-places --query 'coffee near Gare de Lyon, Paris'
```

结果为 JSON。保存返回的地点 ID 以便后续查询详情，避免重复解析相同名称。自动化场景请阅读 [CLI 用法](./cli)，连接智能体请阅读 [MCP 集成](./mcp)。[实用示例](./recipes)展示工具组合，[完整参考](../reference/tools)说明每个参数、默认值与结果。
