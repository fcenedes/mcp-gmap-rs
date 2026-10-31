# 本番運用 {#production-operations}

このガイドは v0.3.x を対象とします。CLI、MCP stdio、Streamable HTTP の全 Google ツールは同じ実装を共有します。

## ネイティブ TLS と相互 TLS {#native-tls-and-mutual-tls}

HTTP リスナーは rustls と TLS 1.2/1.3 を使って直接 HTTPS を提供できます。

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3443 \
  --tls-cert /etc/mcp-gmap/tls/tls.crt \
  --tls-key /etc/mcp-gmap/tls/tls.key \
  --allowed-host maps.example.com:3443
```

`MCP_HTTP_TOKEN` は環境変数または Secret で別途渡します。クライアントは `https://maps.example.com:3443/mcp` に接続し、サーバー証明書を検証して `Authorization: Bearer <token>` を送ります。証明書ファイルは PEM のリーフ証明書とその後の中間証明書チェーンで構成し、秘密鍵は一致する必要があります。実行ファイルは証明書発行や ACME を実装しません。発行・更新には既存 PKI、cert-manager、TLS ゲートウェイを使ってください。

`--tls-client-ca /etc/mcp-gmap/tls/client-ca.pem` を追加すると、その CA が署名したクライアント証明書を必須にできます。ネイティブ mTLS は `/health`、`/metrics` を含む全パスに適用され、HTTP Bearer トークンの代わりにはなりません。TLS 層では設定済み信頼ルートの下の有効なクライアント ID を受け入れ、証明書ごとのツール権限や失効リスト確認はありません。

Linux・macOS ではファイルをアトミックに置き換え、`SIGHUP` を送り、証明書・秘密鍵・任意のクライアント CA をまとめて再読み込みします。既存接続の TLS 状態は保持され、新規ハンドシェイクから新設定を使います。読み込みに失敗すると旧設定を維持し、キーデータを表示せずエラーを記録します。Kubernetes Secret の投射更新後も同様に再読み込みできます。その他の設定は再起動が必要です。

コンテナの UID/GID 10001 が読める権限で、キーを読み取り専用マウントします。投射 Secret には適切なグループと `fsGroup` を使ってください。ネイティブ TLS では Kubernetes HTTP プローブに `scheme: HTTPS` が必要です。kubelet の HTTP プローブは mTLS クライアント証明書を送れないため、その構成では TCP プローブを使います。ただし検証対象はリスナーであり TLS や Google ではありません。

## Kubernetes TLS ゲートウェイ {#kubernetes-tls-gateway}

[Gateway マニフェスト](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/gateway.yaml)は[基本 Kubernetes マニフェスト](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml)の Deployment 用に HTTPS Gateway と HTTPRoute を提供します。先に Gateway API CRD と準拠コントローラーを導入してください。`your-gateway-class` と `maps.example.com` を置き換え、同じ名前空間に `kubernetes.io/tls` 型の `mcp-gmap-tls` Secret を作ります。例は安定版 Gateway・HTTPRoute v1 フィールドと、Gateway API v1.2 以降の HTTPRoute タイムアウトを使います。コントローラーの対応を確認してください。

```sh
kubectl create secret tls mcp-gmap-tls --cert=/secure/fullchain.pem --key=/secure/key.pem
kubectl set env deployment/mcp-gmap-rs \
  MCP_HTTP_ALLOWED_HOSTS='maps.example.com,mcp-gmap-rs:3000,localhost:3000,127.0.0.1:3000'
kubectl apply -f deploy/gateway.yaml
kubectl get gateway,httproute
```

対応する allowed-host 設定を ConfigMap またはデプロイ元ファイルにも保存します。必要なブラウザーオリジンは `MCP_HTTP_ALLOWED_ORIGINS` に追加します。ルートは `/mcp` のみを公開し、ヘルス・メトリクスは非公開 Service に残します。認可と MCP のセッション・プロトコルヘッダーを変更せず通してください。選んだコントローラーで応答のバッファリング・キャッシュを無効にし、ストリーミング・アイドルタイムアウトを設定します。ルートではリクエスト・バックエンドリクエストの期限を無効化しているため、クライアントに合う接続・アイドル制限をゲートウェイに設定してください。

この構成ではゲートウェイで TLS を終端し、ClusterIP バックエンドへ HTTP で接続します。ゲートウェイ・Pod 間も暗号化する場合は、ネイティブ TLS と、コントローラー対応の BackendTLSPolicy またはメッシュを使います。クラスターのネットワークポリシーで Pod・Service への直接アクセスを制限してください。TLS は MCP OAuth ログインを提供しません。実行ファイルはプロセス単位で共有 Bearer トークンと 1 つの Google ID を使います。

上流の [Gateway TLS ガイド](https://gateway-api.sigs.k8s.io/guides/user-guides/tls/)と [HTTPRoute タイムアウトガイド](https://gateway-api.sigs.k8s.io/guides/http-timeouts/)を参照してください。

## 制限、再試行、期限 {#limits-retries-and-deadlines}

| 変数 | 既定値 | オプション | 意味 |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | ツール呼び出し単位の Google HTTP 試行予算、1–10000。再試行を含む |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | Google リクエストごとの追加試行、0–5 |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | ツール全体の期限、秒、1–3600 |
| `MCP_HTTP_RATE_LIMIT` | `120` | HTTP `--rate-limit` | プロセス単位・1 分あたりの認証済み `/mcp` HTTP リクエスト。0 で無効 |
| `MCP_METRICS_ENABLED` | `false` | HTTP `--metrics [true\|false]` | 保護された Prometheus `/metrics` を有効化 |
| `MCP_LOG_FORMAT` | `text` | `--log-format text\|json` | stderr の診断イベント形式 |
| `MCP_TLS_CERT` | 未設定 | HTTP `--tls-cert` | PEM サーバー証明書チェーンファイル |
| `MCP_TLS_KEY` | 未設定 | HTTP `--tls-key` | 対応する PEM サーバー秘密鍵ファイル |
| `MCP_TLS_CLIENT_CA` | 未設定 | HTTP `--tls-client-ca` | PEM CA バンドル。クライアント証明書を必須にする |

これらは既定値 → 任意の dotenv → 環境変数 → オプションの順で適用します。TLS 証明書と秘密鍵は同時指定が必要です。空の環境変数の TLS パスはファイルの値を消去します。`GOOGLE_MAPS_TIMEOUT` は各 Google HTTP リクエストのセマフォ待機・再試行・バックオフ・応答読み取りを制限し、OAuth 取得も同じ値で別途制限します。ツール全体の期限には両方が含まれます。

各呼び出しは新しいリクエスト予算を持ち、並行する全分岐で共有します。プロセスあたり 5 件の Google API リクエストを同時実行できます。予算が尽きると追加試行はローカルで失敗します。複合処理は成功出力に項目別の予算・その他のエラーを含むことがあるため確認してください。制限するのは金額ではなくリクエスト数です。リクエスト・フィールドごとに料金は異なり、行列は複数要素に課金される場合があります。

読み取り専用の Google リクエストは HTTP 429、500、502、503、504 とネットワーク障害を再試行します。秒数・HTTP 日付の `Retry-After` に従い、なければジッター付き指数バックオフを使います。常にリクエスト期限と呼び出し予算の範囲内です。400・401・403 を含む他の HTTP エラーは再試行しません。認証失敗で API キーモードへ暗黙に切り替わることもありません。再試行を 0 にすると初回のみになります。従来の Google JSON 内部エラーステータスは再試行せず報告します。

HTTP 制限は 1 分のレートと同じバースト容量を持ち、連続補充するトークンバケットです。全セッションの `/mcp` 初期化・POST・GET・DELETE に適用します。未認証リクエストは消費せず、拒否時は HTTP 429 と `Retry-After` を返します。ヘルスチェックとメトリクス取得は対象外です。ユーザー・IP 別ではなくプロセス全体の制限で、転送 IP ヘッダーは信頼しません。共有環境ではゲートウェイの認証とクライアント単位の制限を使ってください。

## メトリクスと診断イベント {#metrics-and-diagnostic-events}

```sh
mcp-gmap-rs --log-format json serve http --metrics --rate-limit 120
```

Bearer トークンを設定していれば `GET /metrics` は `/mcp` と同じトークンを要求し、未認証のループバックリスナーでは認証不要です。既定では無効で、ネイティブ mTLS 有効時はその対象になります。信頼するスクレーパーだけに公開してください。Prometheus テキストエンドポイントは次を出力します。

- `mcp_gmap_tool_calls_total{tool,outcome}`：完了したツール呼び出し。
- `mcp_gmap_tool_duration_seconds{tool,outcome}`：所要時間のヒストグラム。
- `mcp_gmap_google_requests_total{api,status}`：再試行を含む Google HTTP 試行。
- `mcp_gmap_http_rate_limited_total`：拒否された HTTP リクエスト。

ラベルは組み込みツール名、API ホスト名、HTTP ステータスコード、固定の結果値だけです。住所・座標・認証情報・トークン・セッション ID は含めません。項目別エラーのある複合処理もトップレベル成功として数える場合があるため、結果と上流ステータスのメトリクスを確認してください。再起動するとメトリクスはリセットされます。

`RUST_LOG=info` でツール名・成否・所要時間を含む完了イベントを確認できます。引数と Google の応答本文は含みません。リスナーの起動アドレスは別途 stderr に出力し、stdout は CLI JSON または MCP 通信専用です。

## Doctor 診断 {#doctor}

```sh
mcp-gmap-rs doctor
# Explicit opt-in: makes potentially billable requests.
mcp-gmap-rs doctor --live
# Check only the API dependencies of selected tools:
mcp-gmap-rs --enabled-tools maps_pollen,maps_area_insights doctor --live
```

`--live` なしの doctor はネットワーク通信をせず、ローカル認証設定、HTTP 設定、TLS ファイル・キーの一致を確認します。キーの有効性、IAM、API の有効化、証明書の信頼・ホスト名・期限、ADC 接続を保証しません。ADC 検出はライブ要求まで `not_tested` です。認証値・JSON は表示せず、JSON レポートを stdout に出し、失敗時の終了ステータスは 1 です。`checks[].status` の `pass`、`fail`、`not_tested` を解析してください。

ライブモードは、有効ツールが使う Google API ごとに代表的なツール呼び出しを 1 回行い、通常の再試行・期限・予算に従います。高コストの複合処理は避け、Routes は正確な座標、Roads は最寄り道路検索を使います。プローブはユーザー住所でなくパリの公開ランドマークを用います。成功はそのプローブが受理されたことだけを意味し、全操作・地域の利用可能性を保証しません。

## 終了と設定変更 {#shutdown-and-configuration-changes}

Ctrl-C と SIGTERM は HTTP サーバーと有効な MCP セッションを終了します。Kubernetes の例では 30 秒を与えます。環境変数で注入した Secret の変更を含むその他の設定は再起動が必要で、SIGHUP 再読み込みに対応するのはネイティブ TLS の証明書・秘密鍵・クライアント CA ファイルだけです。セッションとメトリクスはメモリ内なので再起動で失われます。単一レプリカまたはセッションアフィニティを使用し、所有プロセス終了時にはクライアントが再接続します。

コンテナの権限、プローブ、実行時認証情報のマウントは[デプロイ](./deployment)、全設定とオプションは[設定リファレンス](../reference/configuration)を参照してください。
