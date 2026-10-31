# 設定リファレンス {#configuration-reference}

以下の 21 設定は、必要に応じて CLI、MCP stdio、Streamable HTTP に適用されます。優先順位は **既定値 → 任意の dotenv → 環境変数 → オプション** です。ファイルの選択と上書きは[設定](../guide/configuration)を参照してください。

グローバルオプションはツールまたは `serve` の前、HTTP 専用オプションは `serve http` の後に指定します。オプション欄のダッシュは対応する CLI オプションがないことを示し、環境変数または選択した dotenv ファイルで渡します。

## Google アクセスとツール実行 {#google-access-and-tool-execution}

| 環境変数 | 既定値 | CLI オプション | 意味と許容値 |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_API_KEY` | 未設定 | — | 制限付き Google API キー。`api-key` モードと、この実装のキー専用エンドポイントに必須です。OAuth モードでも後者には必要で、ツール引数にはしません。 |
| `GOOGLE_MAPS_AUTH_MODE` | `api-key` | `--auth-mode` | `api-key` または `oauth`。OAuth は Places (New)、Routes、Weather、Air Quality、Address Validation、Places Aggregate に対応します。 |
| `GOOGLE_APPLICATION_CREDENTIALS` | ADC 検出 | — | 読み取り可能なサービスアカウントまたは対応 ADC JSON のパス。相対パスは作業ディレクトリ基準。ローカル ADC・メタデータ検出では未設定にします。 |
| `GOOGLE_CLOUD_QUOTA_PROJECT` | 未設定 | — | OAuth に必要な消費側の課金・割り当てプロジェクト ID または番号。サービスアカウントにそのプロジェクトの `serviceusage.services.use` が必要です。 |
| `GOOGLE_MAPS_LANGUAGE` | `en` | `--language` | 言語指定に対応した Google 応答の言語コード。本マニュアルの言語選択とは独立です。 |
| `GOOGLE_MAPS_TIMEOUT` | `30` | `--timeout` | リクエスト単位のタイムアウト秒数、整数 1–600。待機、再試行、バックオフ、応答読み取りを含み、OAuth 取得は同じ値で別途制限します。 |
| `GOOGLE_MAPS_ENABLED_TOOLS` | 全部（`*`） | `--enabled-tools` | カンマ区切りの `maps_*` 名または `*`。不明名は起動失敗です。無効ツールは検出されず呼び出せません。 |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | 呼び出し単位の Google HTTP 試行予算、整数 1–10000。複合分岐で共有し、再試行も消費します。 |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | Google リクエストの一時障害に対する追加試行、整数 0–5。0 は初回のみ。タイムアウト・予算内で実行します。 |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | ツール全体の期限秒数、整数 1–3600。認証取得、待機、API 呼び出し、再試行を含みます。 |

プロセスあたり 5 件の Google HTTP リクエストを同時実行できます。予算は金額でなく試行数を制限し、行列 1 リクエストでも複数要素に課金される場合があります。分岐が予算・期限に達した場合、複合ツールによっては項目別エラーを返します。[本番の制限](../guide/production#limits-retries-and-deadlines)を参照してください。

## HTTP リスナーとアクセス {#http-listener-and-access}

| 環境変数 | 既定値 | HTTP オプション | 意味と許容値 |
| --- | --- | --- | --- |
| `MCP_HTTP_TOKEN` | 未設定 | — | MCP クライアントの共有 Bearer トークン。ループバック外で必須です。Google 認証情報とは別で、ネイティブ mTLS でも必要です。 |
| `MCP_HTTP_LISTEN` | `127.0.0.1:3000` | `--listen` | 待ち受ける IP とポート。Docker イメージでは環境値を `0.0.0.0:3000` に設定します。 |
| `MCP_HTTP_ALLOWED_HOSTS` | ループバックホスト | `--allowed-host` | 許可する HTTP Host をカンマ区切りで指定。複数指定のオプションはファイル・環境変数の一覧全体を置換します。適切なポートを含めます。 |
| `MCP_HTTP_ALLOWED_ORIGINS` | なし | `--allowed-origin` | 許可するブラウザーオリジンをカンマ区切りで指定。許可されない Origin ヘッダーは拒否します。ポートを明示し、繰り返すオプションは一覧を置換します。 |
| `MCP_HTTP_RATE_LIMIT` | `120` | `--rate-limit` | プロセス・分あたりの認証済み `/mcp` リクエスト数、整数 0–1000000。0 で無効。レートと同じバースト容量で連続補充します。 |
| `MCP_METRICS_ENABLED` | `false` | `--metrics [true\|false]` | Prometheus `/metrics` を有効化。`--metrics` 単独は true。設定済みの Bearer 認証を共用し、ネイティブ mTLS も適用されます。 |
| `MCP_TLS_CERT` | 未設定 | `--tls-cert` | PEM のサーバーリーフ証明書と中間チェーン。一致する秘密鍵との同時指定が必要です。 |
| `MCP_TLS_KEY` | 未設定 | `--tls-key` | PEM サーバー秘密鍵のパス。実行時の ID に読み取り権限を与え、ソース・イメージから除外します。 |
| `MCP_TLS_CLIENT_CA` | 未設定 | `--tls-client-ca` | 全リスナーパスでクライアント証明書を必須にする PEM クライアント CA バンドル。Bearer トークンを置き換えません。 |

HTTP のトークンバケットはユーザー・IP 単位でなくプロセス全体です。未認可リクエストは消費せず、ヘルスとメトリクスは対象外です。拒否は 429 と `Retry-After` を返します。本文上限は 1 MiB です。[MCP アクセス](../guide/mcp#bind-addresses-and-access)と [TLS 運用](../guide/production#native-tls-and-mutual-tls)を参照してください。

## 診断イベント {#diagnostic-events}

| 環境変数 | 既定値 | CLI オプション | 意味と許容値 |
| --- | --- | --- | --- |
| `RUST_LOG` | `warn` | — | 診断ログフィルター。`info` でツール完了イベントを有効化。引数と Google の応答本文は記録しません。 |
| `MCP_LOG_FORMAT` | `text` | `--log-format` | `text` または `json`。診断は常に stderr に出力し、stdout は CLI JSON・MCP 通信専用にします。 |

## ファイル選択と出力制御 {#file-selection-and-output-controls}

ファイル選択は上記のアプリケーション設定層より先です。`MCP_ENV_FILE` は dotenv パスを指定し、グローバル `--env-file PATH` が優先します。明示したファイルは存在する必要があります。`--no-env-file` は全 dotenv 読み込みを無効化し、`--env-file` と併用できません。未指定なら作業ディレクトリの任意の `.env` だけを読みます。

明示的な空の環境値もファイルを上書きし、任意のトークン・TLS パス・許可リストを消去できます。ただし空の数値設定は無効です。ファイルの読み込みはプロセス環境を変更しません。

`doctor --live` は料金が発生し得るプローブへのオプトインで、通常の `doctor` はオフラインです。ツールの `--output PATH` は JSON、`static-map` では生の PNG を保存します。これらは追加の環境設定ではなくコマンド制御です。

アプリケーション設定と認証ファイルの変更は再起動が必要です。Linux・macOS では SIGHUP でネイティブ TLS の証明書・秘密鍵・クライアント CA をまとめて再読み込みでき、失敗すると既存設定を維持します。[設定](../guide/configuration)と[本番運用](../guide/production)を参照してください。
