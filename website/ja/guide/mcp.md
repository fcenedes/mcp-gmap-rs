# MCP 統合 {#mcp-integration}

ローカルの子プロセスには stdio、常駐サービスには Streamable HTTP で MCP ホストを接続します。両者は同じ有効ツール一覧とスキーマを公開します。

## ローカル stdio {#local-stdio}

一般的な MCP クライアント設定は次のとおりです。

```json
{
  "mcpServers": {
    "google-maps": {
      "command": "/absolute/path/to/mcp-gmap-rs",
      "args": ["--env-file", "/secure/mcp-gmap/runtime.env", "serve", "stdio"]
    }
  }
}
```

ホストの設定形式に従い、実行ファイルは絶対パスで指定します。選択した dotenv ファイルは存在する必要があります。ホストから環境変数を注入することも可能です。これは独立した MCP サーバーの設定であり、特定のアシスタント用ランタイムプラグインは不要です。

```sh
mcp-gmap-rs serve stdio
```

標準入力・標準出力は改行区切りの MCP JSON-RPC 専用です。診断は stderr に出力します。引数なしの起動も同じトランスポートです。コンテナを使う stdio ホストは TTY を割り当てず stdin を接続してください。

## Streamable HTTP {#streamable-http}

```sh
mcp-gmap-rs serve http --listen 127.0.0.1:3000
```

MCP クライアントの Streamable HTTP URL を **`http://127.0.0.1:3000/mcp`** に設定します。リモート MCP の登録に対応するホストでは、エンドポイントと認可ヘッダーを指定します。

```json
{
  "url": "https://maps.example.com/mcp",
  "headers": {
    "Authorization": "Bearer your-mcp-http-token"
  }
}
```

このエントリーを囲む具体的な形式は MCP ホストによって異なります。サーバーは Rust MCP SDK を通じ、プロトコルネゴシエーションと適切な POST、GET/SSE、DELETE のセッション操作に対応します。単独の JSON-RPC POST を完全なセッションとみなさず、MCP クライアントを使用してください。

## バインドアドレスとアクセス {#bind-addresses-and-access}

ループバック以外での待ち受けには `MCP_HTTP_TOKEN` が必要です。クライアントは Google OAuth・API キーとは別の Bearer 認証情報として送信します。エンドポイントの Host 値と実際に利用するブラウザーのオリジンを明示的に追加してください。

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3000 \
  --allowed-host maps.example.com \
  --allowed-origin https://app.example.com:443
```

既定では、許可リストにない Origin ヘッダー付きのリクエストを拒否します。本文の上限は 1 MiB です。`/health` は Google を呼ばずプロセスの生存だけを確認します。公開アクセスには[ネイティブ TLS・mTLS またはゲートウェイ](./production)を設定してください。プロキシは認可と MCP のプロトコル・セッションヘッダーを保持し、バッファリングしないストリーミングを許可する必要があります。セッションと Google の ID はプロセス単位なので、レプリカ追加前に[デプロイ](./deployment)を参照してください。
