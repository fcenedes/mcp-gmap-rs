# トラブルシューティング {#troubleshooting}

最初に、ローカル設定、MCP クライアント・サーバー間、Google への外向きリクエストのどこが失敗したかを特定します。該当する層を修正してから、その操作を再実行してください。

## オフラインから確認する {#start-offline}

```sh
mcp-gmap-rs --version
mcp-gmap-rs tools
mcp-gmap-rs doctor
```

`doctor` は検査に失敗しても JSON を出力します。終了ステータスと `checks[].status` の `pass`、`fail`、`not_tested` を確認してください。オフライン検査は IAM、API の有効化、課金、証明書の信頼・期限、ADC アクセスを保証しません。`/health` はプロセス生存のみの確認です。

ローカル dotenv の構文エラーでツール検出できない場合、`--no-env-file tools` でファイルを無視できます。ただし有効ツールが変わる場合があり、本来のファイル設定の検証成功と混同しないでください。

## よくある失敗の解釈 {#interpret-common-failures}

| 症状 | 次の確認 |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | API キーモードが選択されたか、キー専用エンドポイントを使っています。 |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | OAuth の消費プロジェクト ID・番号を設定します。 |
| Google 403 / API が無効 | 認証情報の消費プロジェクトで正しい API を有効にし、課金とキーの制限を確認します。 |
| `serviceusage.services.use` の拒否 | 割り当てプロジェクトでサービスアカウントに `roles/serviceusage.serviceUsageConsumer` を付与し、IAM の反映を待って再試行します。 |
| API キーの IP 拒否 | プライベート Pod IP ではなく、サーバー・クラスターの公開送信元 IP を許可します。 |
| MCP HTTP 401 | サーバーの Bearer トークンを送信します。Google アクセストークンは別の接続の認証です。 |
| Host/Origin の拒否 | ポートも含め、設定した Host とブラウザーオリジンを正確に一致させます。 |
| MCP の HTTP 429 | プロセス全体のリミッターが枯渇しています。`Retry-After` に従いクライアントの並列数を確認します。 |
| OAuth タイムアウト | トークン・STS・IAM またはメタデータへの接続、時計、リクエストタイムアウトを確認します。 |
| TLS 接続失敗 | 証明書チェーン、ホスト名、信頼、任意の mTLS クライアント証明書を HTTP 認可と別に確認します。 |
| 終了 0 でも不完全な結果 | プロセスの終了ステータスだけでなく行列・複合処理の項目別エラーを確認します。 |

## 意図を持ってライブ接続を検証する {#verify-live-access-deliberately}

ライブプローブには料金が発生する場合があります。必要な API 依存関係に範囲を絞ってください。

```sh
mcp-gmap-rs --enabled-tools maps_weather doctor --live
```

プローブ成功は全地域・全操作を保証しません。Google Cloud の変更後はコンソール設定から成功を推測せず、元の失敗した呼び出しを再実行してください。IAM と認証取得元は[認証](./authentication)、優先順位は[設定](./configuration)、予算・期限・メトリクスは[本番運用](./production)を参照してください。問題報告に認証 JSON や環境変数全体を出力しないでください。
