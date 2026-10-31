# 活用例 {#recipes}

1 つの場所や返された場所 ID の候補一覧を中心に、小さく明示的な呼び出しを組み合わせます。これらの例は料金が発生し得る Google リクエストを送ります。まず[認証](./authentication)を確認してください。複合ツールは複数 API を使い、部分エラーを返す場合があります。

## カフェを探して移動を比較する {#find-a-cafe-and-compare-the-trip}

実際の半径内を検索し、候補 ID だけの詳細を取得して移動を比較します。Nearby の `keyword` は自由文ではなく Google の場所タイプです。

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now --min-rating 4
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --max-photos 0
mcp-gmap-rs distance-matrix --origins '["48.8584,2.2945"]' \
  --destinations '["48.8606,2.3376","48.8530,2.3499"]' --mode walking
```

自然言語の意図には [`search-places`](../reference/tools#maps_search_places)、詳細候補と任意の運転距離には [`compare-places`](../reference/tools#maps_compare_places)、カテゴリ探索には [`explore-area`](../reference/tools#maps_explore_area) を使います。検索 1 ページは網羅的な一覧ではありません。Google のソースリンク、レビュー著者、AI 開示情報を保持してください。

## 立ち寄り先を計画して寄り道を探す {#plan-stops-and-find-a-detour}

```sh
mcp-gmap-rs plan-route \
  --stops '["48.8584,2.2945","48.8606,2.3376","48.8530,2.3499"]' \
  --mode walking --optimize false
mcp-gmap-rs search-along-route --text-query cafes \
  --origin '48.8584,2.2945' --destination '48.8606,2.3376' \
  --mode walking --max-results 3
```

[`directions`](../reference/tools#maps_directions) は時刻・回避設定を含む 1 行程、[`plan-route`](../reference/tools#maps_plan_route) は始終点を保持した中間地点の最適化に使います。公共交通は合計 2 停留点だけです。沿道検索は中点の円で代用せずポリラインに沿って検索します。既定は徒歩なので、意図する移動方法を明示してください。

## 一度解決して周辺環境を調べる {#resolve-once-inspect-local-conditions}

```sh
mcp-gmap-rs geocode --address 'Eiffel Tower, Paris, France'
mcp-gmap-rs weather --latitude 48.8584 --longitude 2.2945 \
  --type forecast_daily --forecast-days 3
mcp-gmap-rs air-quality --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs pollen --latitude 48.8584 --longitude 2.2945 --days 3
mcp-gmap-rs timezone --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs elevation --locations '[{"latitude":48.8584,"longitude":2.2945}]'
```

座標を再利用して解決を繰り返さないようにします。観測・予報・提供者の助言を区別し、対応地域が異なることを考慮してください。[`reverse-geocode`](../reference/tools#maps_reverse_geocode) は座標を住所に、[`batch-geocode`](../reference/tools#maps_batch_geocode) は最大 50 住所を項目別エラー付きで解決します。Time Zone の任意のタイムスタンプは Unix ミリ秒、標高はメートルです。

## 郵便住所を検証する {#validate-a-postal-address}

```sh
mcp-gmap-rs validate-address --address-lines '["5 Avenue Anatole France"]' \
  --locality Paris --postal-code 75007 --region-code FR
```

判定、修正、未解決の構成要素を確認してください。住所検証とジオコーディングは別で、配送を保証しません。この実装の USPS CASS はオプトインで、`regionCode` が US または PR である必要があります。

## 検索ヒット数ではなく場所を集計する {#count-places-without-counting-search-hits}

```sh
mcp-gmap-rs area-insights \
  --area '{"circle":{"latLng":{"latitude":48.8584,"longitude":2.2945},"radius":500}}' \
  --included-types '["restaurant"]' --min-rating 4
```

集計数は通常 Google の文字列 int64 表現です。任意の場所 ID は件数が 100 以下の場合だけ取得でき、詳細は自動取得しません。明示的な可視性分析には [`local-rank-tracker`](../reference/tools#maps_local_rank_tracker) でグリッド上のキーワード順位を測定します。最初は 1 キーワードと 3×3 グリッドを使ってください。順位はレビュー評価ではなく、失敗した点も表示する必要があります。

## GPS データを道路に対応付けて地図を保存する {#match-gps-data-and-save-a-map}

順序付き軌跡には [`snap-to-roads`](../reference/tools#maps_snap_to_roads)、独立した点には [`nearest-roads`](../reference/tools#maps_nearest_roads) を使います。`originalIndex` を保持してください。補間で追加される点はこの値を持たない場合があります。速度制限は返しません。

```sh
mcp-gmap-rs nearest-roads --points '[{"latitude":48.8584,"longitude":2.2945}]'
mcp-gmap-rs static-map --center '48.8584,2.2945' --zoom 14 --output area.png
```

[ツールリファレンス](../reference/tools)に各操作の正確な範囲、必須項目、既定値、結果形式を記載しています。
