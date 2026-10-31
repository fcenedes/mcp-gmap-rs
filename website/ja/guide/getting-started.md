# はじめに {#getting-started}

実行ファイルを 1 つインストールして Google の認証を設定すれば、ターミナルや MCP クライアントから同じ 23 ツールを使えます。本マニュアルは v0.3.1 を対象としています。

## インストール {#install}

Linux x86_64、macOS Apple Silicon、macOS Intel のビルド済みバイナリを [GitHub Releases](https://github.com/fcenedes/mcp-gmap-rs/releases) で公開しています。環境に合うアーカイブをダウンロードし、`SHA256SUMS` と照合してから展開し、`mcp-gmap-rs` を `PATH` に置いてください。Linux バイナリには glibc 2.35 以降が必要です。macOS バイナリは署名・公証されていません。

Rust 1.88 以降では crates.io からインストールできます。

```sh
cargo install mcp-gmap-rs --version 0.3.1 --locked
mcp-gmap-rs --version
```

ビルド後の実行には Rust、Python、JavaScript のランタイムは不要です。ホストには CA 証明書、DNS、Google への外向き HTTPS 接続が必要です。

## 認証情報の設定 {#configure-access}

[制限付きキーを作成してサービスを有効化](./google-cloud)するか、[サービスアカウントの OAuth](./authentication) を設定します。作業ディレクトリ内の Git 管理対象外の `.env` に認証情報を保存します。

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

環境変数はファイルの値を上書きします。認証情報をバージョン管理やイメージ層に含めないでください。`.env` は親ディレクトリを検索しないため、作業ディレクトリに注意してください。

## 検出、確認、呼び出し {#discover-check-call}

ツール検出とオフライン診断は Google にリクエストを送信しません。

```sh
mcp-gmap-rs tools
mcp-gmap-rs schema search-places
mcp-gmap-rs doctor
```

続いて、料金が発生する可能性のある最初の呼び出しを実行します。

```sh
mcp-gmap-rs search-places --query 'coffee near Gare de Lyon, Paris'
```

結果は JSON です。返された場所 ID を保存して詳細取得に再利用し、同じ名前を何度も解決しないようにします。自動化には [CLI の使い方](./cli)、エージェント接続には [MCP 統合](./mcp)を参照してください。[活用例](./recipes)ではツールの組み合わせ、[完全なリファレンス](../reference/tools)ではすべての引数・既定値・結果を説明しています。
