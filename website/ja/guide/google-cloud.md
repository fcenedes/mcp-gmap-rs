# Google Cloud の設定 {#google-cloud-setup}

この実行ファイルは API キーまたは OAuth 2.0 認証情報で **Google への外向き呼び出し**を認証します。MCP HTTP サーバーに接続するクライアントの認証には独立した `MCP_HTTP_TOKEN` を使い、Google OAuth を選んでも置き換わりません。

## サービスと課金の有効化 {#enable-services-and-billing}

[Google Cloud コンソール](https://console.cloud.google.com/)でプロジェクトを選択または作成し、**Billing** で有効な請求先アカウントに関連付けます。**APIs & Services → Library** で必要な API を有効化してください。API の有効化と API キーの制限での許可は別の手順です。呼び出しには料金が発生する場合があります。

このバージョンで利用するエンドポイントは次のとおりです。

| Google API | 有効化するサービス | この実装での認証 |
| --- | --- | --- |
| [Places API (New)](https://developers.google.com/maps/documentation/places/web-service/get-api-key) | `places.googleapis.com` | API キーまたは OAuth |
| [Routes API](https://developers.google.com/maps/documentation/routes/get-api-key) | `routes.googleapis.com` | API キーまたは OAuth |
| [Weather API](https://developers.google.com/maps/documentation/weather/get-api-key) | `weather.googleapis.com` | API キーまたは OAuth |
| [Air Quality API](https://developers.google.com/maps/documentation/air-quality/get-api-key) | `airquality.googleapis.com` | API キーまたは OAuth |
| [Address Validation API](https://developers.google.com/maps/documentation/address-validation/get-api-key) | `addressvalidation.googleapis.com` | API キーまたは OAuth |
| [Places Aggregate API](https://developers.google.com/maps/documentation/places-aggregate/get-api-key) | `areainsights.googleapis.com` | API キーまたは OAuth |
| [Pollen API](https://developers.google.com/maps/documentation/pollen/get-api-key) | `pollen.googleapis.com` | API キー |
| [Roads API](https://developers.google.com/maps/documentation/roads/get-api-key) | `roads.googleapis.com` | API キー |
| [Geocoding API v3](https://developers.google.com/maps/documentation/geocoding/guides-v3/get-api-key) | `geocoding-backend.googleapis.com` | API キー |
| [Elevation API](https://developers.google.com/maps/documentation/elevation/get-api-key) | `elevation-backend.googleapis.com` | API キー |
| [Time Zone API](https://developers.google.com/maps/documentation/timezone/get-api-key) | `timezone-backend.googleapis.com` | API キー |
| [Maps Static API](https://developers.google.com/maps/documentation/maps-static/get-api-key) | `static-maps-backend.googleapis.com` | API キー |

管理者の `gcloud` ログインで全 12 サービスを有効化するには、次を実行します。

```sh
export PROJECT_ID='your-project-id'
gcloud services enable --project="$PROJECT_ID" \
  places.googleapis.com routes.googleapis.com \
  weather.googleapis.com airquality.googleapis.com \
  geocoding-backend.googleapis.com elevation-backend.googleapis.com \
  timezone-backend.googleapis.com static-maps-backend.googleapis.com \
  addressvalidation.googleapis.com areainsights.googleapis.com \
  pollen.googleapis.com roads.googleapis.com
```

CLI の `directions` と `distance-matrix` は **Routes API** を使うため、従来の Directions API と Distance Matrix API は不要です。このプロジェクトの Geocoding は v3 のままであり、[Google Geocoding v4 の OAuth 対応](https://developers.google.com/maps/documentation/geocoding/geocoding-v4-overview)は v3 呼び出しの認証方式を変更しません。複合ツールは複数の API を使うことがあります。主処理が Places や Routes でも、名前による場所解決やフォールバックで Geocoding が必要になる場合があります。この依存を避ける場合は座標を使うか、キー専用エンドポイント用に制限付きキーを保持してください。

## 制限付き API キーの作成 {#create-a-restricted-api-key}

1. 選択したプロジェクトの [Google Maps Platform → Keys & Credentials](https://console.cloud.google.com/google/maps-apis/credentials) を開き、**Create credentials → API key** を選びます。
2. このインスタンスが使う表内の API だけを選択します。既存のキーでは API 制限を編集して保存します。
3. サーバー側アプリケーションなので、アプリケーション制限を **IP addresses** にします。Google が見る公開送信元 IP を許可してください。通常、Kubernetes の Pod IP・プライベート IP は公開送信元ではありません。固定 Cloud NAT などの送信元アドレスを用意すると制限を予測しやすくなります。
4. キーは Git 管理対象外の `.env`、環境変数注入、Kubernetes Secret に保存します。[Google のキー制限ガイド](https://developers.google.com/maps/api-security-best-practices#restrict-your-api-keys)を参照してください。

`.env` の例：

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

既定値は `api-key` です。サービスアカウントのパスを指定するだけでは認証モードは変わりません。ツール検出は Google に接続しないため、実際の呼び出しでプロジェクトの課金、API の有効化、制限を確認してください。

```sh
mcp-gmap-rs geocode --address 'Paris, France'
```

## 次のステップ {#next-steps}

サービスアカウント JSON、ADC、ローカルでの権限借用、GKE の ID は[認証](./authentication)を参照し、その後 [CLI](./cli) または [MCP](./mcp) を選んでください。[ツールリファレンス](../reference/tools)に各ツールの認証要件を記載しています。


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
