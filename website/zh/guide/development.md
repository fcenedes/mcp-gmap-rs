# 开发与发布 {#development-and-releases}

运行时是独立 Rust 项目。CLI、MCP stdio 和 Streamable HTTP 共享同一套工具实现。文档网站属于独立的编写/构建工具；最终用户只需要编译后的可执行文件。

## 构建与检查 {#build-and-check}

克隆[仓库](https://github.com/fcenedes/mcp-gmap-rs)，使用仓库锁文件：

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
cargo test --doc --all-features --locked
cargo build --release --locked
```

声明的最低版本为 Rust 1.88；发布二进制使用 Rust 1.94.1。默认测试使用本地 Google/令牌测试夹具，不需凭据，也不连接 Google。CI 还检查 Linux/macOS、最低 Rust、stable/nightly rustdoc、源代码打包、Docker/MCP/TLS 冒烟场景和 Kubernetes 清单。

本地验证后，`cargo clean` 可回收生成的构建输出；后续构建会重新编译。

## 保持参考完整 {#keep-the-reference-complete}

规范工具指南结合实际可执行文件模式，以及审阅过的默认值、结果和行为说明：

```sh
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs --check --verify-examples
```

检查器会移除凭据、禁用 dotenv 加载，在缺少密钥的边界验证所有示例，不连接 Google。参数或行为变更后应更新审阅的 JSON 说明。

在线测试需显式启用且可能计费。明确配置测试密钥后运行 `cargo test --locked --test live live_maps_ -- --ignored --test-threads=1`。六项 OAuth 测试使用 `live_oauth_` 过滤器，并从子进程移除密钥。切勿提交凭据，也不要在普通 CI 启用在线测试。

## 发布版本 {#publish-a-version}

同时更新 Cargo 版本、锁文件、变更日志、安装示例和发布说明。推送并等待 CI，然后创建匹配的带注释 `vX.Y.Z` 标签。Release 工作流检查准确标签，构建三种原生目标，验证压缩包/校验和，再发布暂存草稿。

独立的 **Publish crate** 工作流要求对应 GitHub Release 已发布，并对同一标签手动触发。将限定范围的 `CARGO_REGISTRY_TOKEN` 存入 `crates-io` GitHub 环境，绝不要存到仓库。crates.io 发布后 docs.rs 自动构建；仅发布 GitHub Release 不会更新托管 Rust 文档。上传后应验证实际托管构建。

参阅[发布流程](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/releases.md)、[验证记录](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/validation.md)和[公开 Rust 入口](https://docs.rs/mcp-gmap-rs/latest/mcp_gmap_rs/fn.run.html)。
