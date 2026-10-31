# デプロイ {#deployment}

Google API キーの作成、API の有効化、サービスアカウント OAuth は [Google Cloud 設定ガイド](./google-cloud)を参照してください。コンテナでも同じ多層設定を使います。OAuth はマウントした JSON 認証ファイルまたはワークロードの ADC ID を利用でき、認証情報は実行時に渡します。

[ソースリポジトリ](https://github.com/fcenedes/mcp-gmap-rs)をダウンロードまたはクローンし、ルートでビルドします。

```sh
docker build -t mcp-gmap-rs:local .
```

マルチステージ Dockerfile は Rust 1.94.1 とロックファイルでビルドします。実行環境にはバイナリ、Debian の CA 証明書、ライセンスが入り、UID/GID 10001 で動作し、読み取り専用ルートファイルシステムに対応します。許可リスト方式のビルドコンテキストはローカル `.env`、Git データ、ビルド出力を除外します。ビルドに認証情報は不要です。BuildKit は次回用に Cargo のダウンロードとビルド出力をキャッシュします。

同じ Dockerfile で Linux amd64・arm64 のネイティブビルドが可能です。設定済み Buildx ビルダーで両方をビルドし、自分のレジストリへ送るには次を使います。

```sh
docker buildx build --platform linux/amd64,linux/arm64 \
  -t registry.example.com/your-team/mcp-gmap-rs:YOUR_VERSION --push .
```

コンテナレジストリへの自動公開は設定されていません。イメージ名とバージョンを、自分でビルド・公開したものに置き換えてください。

## HTTP、CLI、stdio の実行 {#run-http-cli-or-stdio}

イメージは既定で `0.0.0.0:3000` に対して `serve http` を起動します。実行時に Bearer トークンを渡してください。たとえば `.env.example` と同じ形式の `.env` を使えます。

```sh
docker run --rm --read-only --cap-drop ALL --security-opt no-new-privileges \
  --env-file .env -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

コンテナはループバック外で待ち受けるため、`.env` に `MCP_HTTP_TOKEN` が必要です。Google 呼び出しは有効ツールに応じて `GOOGLE_MAPS_API_KEY`、または設定ガイドの OAuth 設定と認証ファイルのマウントを使います。クライアントは `http://127.0.0.1:3000/mcp` に `Authorization: Bearer <token>` を送ります。`GET /health` はトークン不要で Google に接続しません。

Docker の `--env-file` は環境変数を注入します。アプリ自身のファイルローダーを使うには、読み取り可能なファイルをマウントして指定します。

```sh
docker run --rm --read-only \
  --mount type=bind,src="$(pwd)/.env",dst=/etc/mcp-gmap/runtime.env,readonly \
  -e GOOGLE_MAPS_LANGUAGE=fr -p 127.0.0.1:3000:3000 \
  mcp-gmap-rs:local --env-file /etc/mcp-gmap/runtime.env serve http
```

環境変数はマウントファイルを、CLI オプションはその両方を上書きします。イメージの `MCP_HTTP_LISTEN=0.0.0.0:3000` も環境変数です。必要なら `-e MCP_HTTP_LISTEN=...` または `--listen` で変更してください。環境変数だけを使うには `--no-env-file serve http` を渡します。

CLI・stdio ではイメージのコマンドを上書きします。

```sh
docker run --rm mcp-gmap-rs:local tools
docker run --rm --env-file .env mcp-gmap-rs:local geocode --address Paris
docker run --rm -i --env-file .env mcp-gmap-rs:local serve stdio
```

stdio では TTY を割り当てず `-i` を使います。CLI で出力ファイルを作る場合、UID 10001 が書けるディレクトリをマウントし、その中に `--output` を指定してください。

## Kubernetes {#kubernetes}

[基本 Kubernetes マニフェスト](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml)は非機密設定を ConfigMap、認証情報を既存の `mcp-gmap-rs` Secret から取得します。`GOOGLE_MAPS_API_KEY` と `MCP_HTTP_TOKEN` を含むローカルファイルから Secret を作成します（そのファイルはコミットしません）。

```sh
kubectl create secret generic mcp-gmap-rs --from-env-file=/secure/credentials.env
```

Deployment の `mcp-gmap-rs:local` をレジストリのイメージに置き換えるか、ローカルイメージを開発クラスターに読み込んでから、例を適用します。

```sh
kubectl apply -f deploy/kubernetes.yaml
kubectl rollout status deployment/mcp-gmap-rs
kubectl port-forward service/mcp-gmap-rs 3000:3000
```

例は `default` 名前空間用です。別の名前空間や ingress ではサービス DNS ホスト名を調整してください。ClusterIP Service はクラスター内部用なので、リモート接続には TLS ingress・プロキシを使い、`MCP_HTTP_ALLOWED_HOSTS` 内の Host、認可、MCP/SSE ヘッダーを転送します。必要ならポートを含むブラウザーオリジンを明示します。

Deployment は全設定を `envFrom` で注入するため `--no-env-file` を指定しています。代わりに dotenv ファイルを含む Secret をマウントする場合はそのオプションを削除し、`args: ["--env-file", "/etc/mcp-gmap/runtime.env", "serve", "http"]` を設定します。投射ファイルは UID 10001 が読める必要があり、Secret・ConfigMap の環境変数が引き続き優先します。Kubernetes の[環境変数注入](https://kubernetes.io/docs/tasks/inject-data-application/define-environment-variable-container/)と [Secret ボリューム](https://kubernetes.io/docs/concepts/configuration/secret/)を参照してください。

設定は起動時に読み込みます。Secret・ConfigMap の変更後は Pod を再起動します。ネイティブ TLS ファイルは SIGHUP 再読み込みにも対応します。プローブの `/health` は HTTP の可用性を確認するだけで、Google API の有効化や認証の成功は保証しません。Google 呼び出しには外向き HTTPS と DNS が必要です。API キーの制限はクラスターの公開送信元アドレスを許可し、OAuth ではトークン・ID エンドポイントに接続できる必要があります。

既定の MCP セッションストアはメモリ内なので 1 レプリカにします。複数レプリカには各セッションを所有 Pod に届けるセッションアフィニティが必要です。ローリング再起動はセッションを破棄するため、クライアントは再接続します。例のリソース要求・上限は出発点であり、負荷に合わせて調整してください。

SIGTERM で MCP・HTTP を正常終了します。例では Kubernetes が 30 秒を与えます。イメージは capability を除去し、権限昇格を禁止して動作します。

## 本番の HTTPS、メトリクス、制限 {#production-https-metrics-and-limits}

[本番運用ガイド](./production)ではネイティブ TLS・mTLS、証明書再読み込み、任意の `deploy/gateway.yaml` HTTPS Gateway、メトリクス、リクエスト予算、HTTP レート制限、JSON ログ、オフライン・ライブの `doctor` 検査を解説します。イメージ名の後に同じ TLS オプションを渡すか、読み取り専用 PEM マウントとともに `MCP_TLS_CERT`、`MCP_TLS_KEY`、任意の `MCP_TLS_CLIENT_CA` を注入します。実際のキーはイメージの外に置きます。既定の ConfigMap はリクエスト制限と JSON ログを設定し、メトリクスは明示的に有効化するまで無効です。平文 Service は内部のゲートウェイ・Pod 間接続用です。

## コンテナでの OAuth 認証情報 {#oauth-credentials-in-a-container}

サービスアカウント JSON を読み取り専用でマウントし、ホスト側ではなくコンテナ内のパスを指定します。

```sh
docker run --rm --read-only --env-file /secure/oauth.env \
  --mount type=bind,src=/secure/service-account.json,dst=/credentials/service-account.json,readonly \
  -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

OAuth 環境ファイルには `GOOGLE_MAPS_AUTH_MODE=oauth`、`GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json`、`GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id`、`MCP_HTTP_TOKEN` を設定します。UID 10001 がマウントファイルを読める必要があります。API キーはキー専用エンドポイントを使うツールの場合だけ保持します。GKE では長期キーをマウントする代わりに[ワークロード ID の手順](./authentication#c-kubernetes-and-gke-workload-identity)を使ってください。
