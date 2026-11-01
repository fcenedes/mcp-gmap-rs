# 認証 {#authentication}

Google への外向き認証情報と MCP HTTP クライアント用の認証情報は別に設定します。既定は API キーモードであり、サービスアカウントのパスだけでは OAuth は有効になりません。

## API キーと OAuth の対応範囲 {#api-key-and-oauth-coverage}

| エンドポイントの種類 | API キー | この実行ファイルでの OAuth |
| --- | --- | --- |
| Places (New)、Routes、Weather、Air Quality、Address Validation、Places Aggregate | 対応 | 対応 |
| Geocoding v3、Elevation、Time Zone、Maps Static、Pollen、Roads | 対応 | 非対応 |

主処理が OAuth に対応していても、名前付きの場所解決や複合呼び出しでキーが必要になる場合があります。不要なジオコーディングを避けるには正確な座標を使ってください。OAuth の失敗が API キーに暗黙にフォールバックすることはありません。[Google Cloud の設定](./google-cloud)で正確なサービスの有効化とキーの制限を確認してください。

## サービスアカウント OAuth {#service-account-oauth}

OAuth モードでは、Places (New)、Routes、Weather、Air Quality、Address Validation、Places Aggregate のリクエストに OAuth **アクセストークン**と `X-Goog-User-Project` ヘッダーを付けます。これらの API が文書化する `https://www.googleapis.com/auth/cloud-platform` スコープを要求します。Google の Rust 認証ライブラリがトークンを取得・キャッシュ・更新します。`GOOGLE_MAPS_TIMEOUT` は認証初期化とトークン取得にも適用されます。サービスアカウントのフローでは、手動のトークン貼り付けやアプリの OAuth 同意画面は不要です。[Maps のサーバー側 OAuth ガイド](https://developers.google.com/maps/api-security-best-practices#use-oauth-for-server-side-apps)を参照してください。

専用サービスアカウントを作成し、Maps の課金・割り当てプロジェクトでサービスを消費する権限を与えます。

```sh
export PROJECT_ID='your-project-id'
export SA_EMAIL="mcp-gmap@${PROJECT_ID}.iam.gserviceaccount.com"
gcloud iam service-accounts create mcp-gmap --project="$PROJECT_ID" \
  --display-name='MCP Google Maps'
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:${SA_EMAIL}" \
  --role='roles/serviceusage.serviceUsageConsumer'
```

Service Usage Consumer ロールには課金・割り当てのための `serviceusage.services.use` が含まれます。ただしサービスの有効化やキー作成の権限はなく、その準備には別途権限を持つ管理者が必要です。[Google Cloud 設定ガイド](./google-cloud)に従って割り当てプロジェクトで Maps API を有効化してください。[Service Usage の権限](https://docs.cloud.google.com/service-usage/docs/access-control)も参照してください。

以下から認証情報の取得元を 1 つ選びます。CLI、stdio、HTTP で同じ設定を使い、通常のファイル → 環境変数 → オプションの優先順位に従います。

### A. サービスアカウント JSON キーのマウント {#a-mounted-service-account-json-key}

組織で JSON キーが許可されている場合は、**IAM & Admin → Service Accounts** でアカウントを選び、**Keys → Add key → Create new key → JSON** を選択します。ソースのチェックアウト外に保存してください。次のコマンドでも作成できます。

```sh
mkdir -p "$HOME/.config/mcp-gmap-rs"
gcloud iam service-accounts keys create \
  "$HOME/.config/mcp-gmap-rs/service-account.json" \
  --iam-account="$SA_EMAIL" --project="$PROJECT_ID"
chmod 600 "$HOME/.config/mcp-gmap-rs/service-account.json"
export GOOGLE_APPLICATION_CREDENTIALS="$HOME/.config/mcp-gmap-rs/service-account.json"
export GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
export GOOGLE_MAPS_AUTH_MODE=oauth
```

マウントするファイルの場合、同等の `.env` は次のとおりです。

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
# Optional, for Geocoding v3 / Elevation / Time Zone / Static Maps / Pollen / Roads:
# GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

読み取り可能な JSON ファイルを使う場合だけパスを設定してください。空文字列や JSON 本文を設定してはいけません。相対パスは作業ディレクトリから解決され、ファイルの値はプロセス環境を変更せず直接利用されます。ローダーは ADC の `external_account`、`impersonated_service_account`、`authorized_user` JSON 設定にも対応します。認証ファイル変更後はプロセスを再起動してください。

サービスアカウントキーは長期認証情報です。利用できる場合は以下のキーレス方式を優先し、JSON キーを Git やイメージ層に含めず、未使用キーをローテーションまたは削除してください。Google の[キー作成と組織ポリシー](https://docs.cloud.google.com/iam/docs/keys-create-delete)を参照してください。

### B. JSON キーを使わない Application Default Credentials {#b-application-default-credentials-without-a-json-key}

`GOOGLE_APPLICATION_CREDENTIALS` は、プロセス環境と選択した dotenv ファイルまたは Kubernetes Secret/ConfigMap の両方で未設定にします。シェルの `unset` だけではファイル層の値は消えません。空パスを設定するのでなく項目自体を削除し、次を設定してください。

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
```

公式認証ライブラリは [Application Default Credentials (ADC)](https://docs.cloud.google.com/docs/authentication/application-default-credentials) を使います。ローカル ADC があれば利用し、なければメタデータサーバー経由で付与されたワークロード・サービスアカウントを使用します。認証情報の検出は遅延実行され、`tools`、`schema`、ヘルプはトークン取得なしで動作します。OAuth API 呼び出しには割り当てプロジェクトと有効な認証情報が必要です。

ローカル開発でサービスアカウントの権限借用を使うには、IAM Credentials API を有効化し、開発者アカウントにそのサービスアカウントの Token Creator を付与してからローカル ADC を作成します。

```sh
gcloud services enable iamcredentials.googleapis.com --project="$PROJECT_ID"
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
  --project="$PROJECT_ID" --member='user:you@example.com' \
  --role='roles/iam.serviceAccountTokenCreator'
unset GOOGLE_APPLICATION_CREDENTIALS
gcloud auth application-default login --impersonate-service-account="$SA_EMAIL"
export GOOGLE_MAPS_AUTH_MODE=oauth
export GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
```

この方式は短期のサービスアカウントトークンで Maps を呼び出し、秘密鍵のダウンロードが不要です。[サービスアカウントの権限借用](https://docs.cloud.google.com/iam/docs/service-account-impersonation)を参照してください。

### C. Kubernetes と GKE のワークロード ID {#c-kubernetes-and-gke-workload-identity}

一般的な Kubernetes では、認証 JSON を Secret ファイルとしてマウントし、方式 A の 3 つの OAuth 変数を設定します。コンテナの UID 10001 が読める必要があります。読み取り専用でマウントし、`MCP_HTTP_TOKEN` は別の実行時 Secret に保存してください。[コンテナガイド](./deployment)で基本 Deployment を解説します。

GKE では、Kubernetes ServiceAccount を IAM サービスアカウントに関連付ける [Workload Identity Federation for GKE](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/workload-identity) を優先します。まずクラスター・ノードプールで有効化し（Autopilot では有効）、続いて次を実行します。

```sh
gcloud services enable iamcredentials.googleapis.com --project="$PROJECT_ID"
kubectl create serviceaccount mcp-gmap-rs --namespace default
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
  --project="$PROJECT_ID" --role='roles/iam.workloadIdentityUser' \
  --member="serviceAccount:${PROJECT_ID}.svc.id.goog[default/mcp-gmap-rs]"
kubectl annotate serviceaccount mcp-gmap-rs --namespace default \
  "iam.gke.io/gcp-service-account=${SA_EMAIL}"
kubectl patch deployment mcp-gmap-rs --namespace default --type=merge \
  -p '{"spec":{"template":{"spec":{"serviceAccountName":"mcp-gmap-rs"}}}}'
kubectl set env deployment/mcp-gmap-rs --namespace default \
  GOOGLE_MAPS_AUTH_MODE=oauth GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
```

これらのコマンドはクラスターと IAM サービスアカウントが `PROJECT_ID` 内にあり、基本 Deployment が `default` にある前提です。環境に合わせてプロジェクト・名前空間を変更してください。メタデータ方式では `GOOGLE_APPLICATION_CREDENTIALS` を設定しません。Pod は GKE メタデータサーバーからアクセストークンを取得します。GKE 以外では、`external_account` 設定とプロバイダーの投射 ID トークンを Google Workload Identity Federation と組み合わせられます。

OAuth のみで運用する場合は実行時 Secret から Google API キーを削除し、OAuth 対応 API だけを使うツール・入力を有効化します。表のキー専用エンドポイントや、それを必要とする複合処理を使う場合はキーを保持します。対応 API の OAuth 失敗は明示的なエラーとなり、暗黙にキーへ切り替わることはありません。

## 検証とトラブルシューティング {#verify-and-troubleshoot}

```sh
mcp-gmap-rs --auth-mode oauth search-places --query 'coffee in Paris'
mcp-gmap-rs --auth-mode oauth weather --latitude 48.8584 --longitude 2.2945
```

これらは料金が発生し得る実際の呼び出しです。テスト時に有効なモードと割り当てプロジェクトを確認してください。OAuth の課金先は `GOOGLE_CLOUD_QUOTA_PROJECT`、キー認証の呼び出しはキーのプロジェクトです。両者が異なる場合があります。

オフライン設定検査には `mcp-gmap-rs doctor`、明示的に料金の発生し得る API プローブを行うには `doctor --live` を使います。[本番運用ガイド](./production)でレポートとツールの絞り込みを説明しています。

OAuth 変数をエクスポートした後、明示的に有効化する 6 つのライブ受け入れテストを実行できます。

```sh
cargo test --locked --test live live_oauth_ -- --ignored --test-threads=1
```

各テストは子プロセスから API キーを明示的に除去するため、成功は OAuth の動作を示します。Places、Routes、Weather、Air Quality、Address Validation、Places Aggregate を対象とし、対応 API の有効化が必要です。`.env` は読み込まないため、OAuth 設定をエクスポートするか設定済みワークロードで実行してください。通常のテスト・CI はローカルのトークン・API フィクスチャを使い、サービスアカウント不要で Google に接続しません。

| 症状 | 確認事項 |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | 選択した処理がキー専用エンドポイントを使っているか、API キーモードが選択されています。 |
| OAuth 認証失敗 | 認証ファイルのパス・種類、キーの有効性、ワークロード ID・権限借用の関連付け、利用する IAM Credentials API、トークンエンドポイントへの接続、時計の正確さを確認します。アプリのエラーに認証内容やトークンは含まれません。 |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | OAuth 呼び出し用の Maps 消費プロジェクト ID・番号を設定します。 |
| Google `403` / API が無効 | 消費プロジェクトで正しいサービスを有効化し、課金と Service Usage 権限を確認します。 |
| API キーの IP 拒否 | サーバー・クラスターの公開送信元 IP と制限を一致させます。 |
| トークンのタイムアウト | Google のトークン・STS・IAM エンドポイントまたはメタデータサーバーへの接続を許可し、`GOOGLE_MAPS_TIMEOUT` を確認します。 |
| この MCP サーバーの HTTP `401` | サーバーの `MCP_HTTP_TOKEN` を送ります。Google のサービスアカウントトークンは別の外向き接続を認証するものです。 |


## 追加サービスと制限 {#expanded-services}

Solar と Map Tiles も OAuth に対応します。Datasets と Route Optimization は OAuth と操作別の IAM 権限が必要です。Street View Publish は明示的な authorized_user 認証情報ファイルと streetviewpublish スコープへのユーザー同意が必要です。

| サービス | API | 認証 |
| --- | --- | --- |
| Geolocation API | `geolocation.googleapis.com` | API キー |
| Solar API | `solar.googleapis.com` | API キー / OAuth |
| Aerial View API | `aerialview.googleapis.com` | API キー |
| Map Tiles API | `tile.googleapis.com` | API キー / OAuth |
| Street View Static API | `street-view-image-backend.googleapis.com` | API キー |
| Maps Platform Datasets API | `mapsplatformdatasets.googleapis.com` | OAuth / IAM |
| Route Optimization API | `routeoptimization.googleapis.com` | OAuth / IAM |
| Street View Publish API | `streetviewpublish.googleapis.com` | 同意済みユーザー OAuth。サービス アカウントは不可 |

Speed Limits は制限付きの Asset Tracking 利用資格が必要です。Aerial View は対応する米国の住所に限られ、動画のダウンロードやキャッシュは禁止です。Map Tiles は表示専用で、機械による分析やデータ抽出は禁止されています。Weather の分単位予報と地図は実験機能で、キャッシュは最大 1 時間、追加の利用制限があります。各ツールのリファレンスに利用条件、帰属表示、制限を記載しています。

[Google: Speed Limits](https://developers.google.com/maps/documentation/roads/speed-limits), [Map Tiles](https://developers.google.com/maps/documentation/tile/policies), [Aerial View](https://developers.google.com/maps/documentation/aerial-view/policies), [Street View Publish](https://developers.google.com/streetview/publish/authorizing).
