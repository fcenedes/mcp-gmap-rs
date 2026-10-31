# 開発とリリース {#development-and-releases}

ランタイムは独立した Rust プロジェクトです。CLI、MCP stdio、Streamable HTTP は同じツール実装を共有します。ドキュメントサイトは別の執筆・ビルドツールで、利用者に必要なのはコンパイル済み実行ファイルだけです。

## ビルドと検証 {#build-and-check}

[リポジトリ](https://github.com/fcenedes/mcp-gmap-rs)をクローンし、ロックファイルを使います。

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
cargo test --doc --all-features --locked
cargo build --release --locked
```

宣言上の最低バージョンは Rust 1.88、リリースバイナリは Rust 1.94.1 で作成します。既定テストはローカル Google・トークンフィクスチャを使い、認証情報も Google への接続も不要です。CI は Linux・macOS、最低 Rust、stable・nightly rustdoc、ソースパッケージ、Docker・MCP・TLS のスモークテスト、Kubernetes マニフェストも検証します。

ローカル検証後は `cargo clean` で生成物を回収できます。次回ビルドでは再コンパイルします。

## リファレンスの完全性を保つ {#keep-the-reference-complete}

正規のツールガイドは実行ファイルの実際のスキーマと、レビュー済みの既定値・結果・動作の説明を組み合わせます。

```sh
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs --check --verify-examples
```

チェッカーは認証情報を除去して dotenv 読み込みを無効にし、キー不足の境界まで全実行例を検証します。Google には接続しません。引数や動作の変更時にはレビュー済み JSON 説明を更新してください。

ライブテストはオプトインで、料金が発生する場合があります。テストキーを明示的に設定して `cargo test --locked --test live live_maps_ -- --ignored --test-threads=1` を実行します。6 つの OAuth テストは `live_oauth_` フィルターを使い、子プロセスからキーを除去します。認証情報をコミットしたり、通常の CI でライブテストを有効化したりしないでください。

## バージョンの公開 {#publish-a-version}

Cargo のバージョン、ロックファイル、変更履歴、インストール例、リリースノートを一緒に更新します。プッシュして CI を待ち、一致する注釈付き `vX.Y.Z` タグを作ります。Release ワークフローは正確なタグを検証し、3 つのネイティブ対象をビルドし、アーカイブとチェックサムを確認してから準備済みドラフトを公開します。

別の **Publish crate** ワークフローには、公開済み GitHub Release が必要です。同じタグで手動実行します。範囲を限定した `CARGO_REGISTRY_TOKEN` は `crates-io` GitHub 環境に保存し、リポジトリには入れません。crates.io 公開後に docs.rs が自動ビルドします。GitHub Release だけでは Rust の公開ドキュメントは更新されないため、アップロード後に実際のホスト側ビルドを確認してください。

[公開手順](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/releases.md)、[検証記録](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/validation.md)、[公開 Rust エントリーポイント](https://docs.rs/mcp-gmap-rs/latest/mcp_gmap_rs/fn.run.html)も参照してください。
