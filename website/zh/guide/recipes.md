# 实用示例 {#recipes}

围绕一个位置或已返回的地点 ID 候选列表，组合小而明确的调用。这些示例会向 Google 发送可能计费的请求。先阅读[身份验证](./authentication)；复合工具可能使用多个 API，并返回部分错误。

## 查找咖啡馆并比较行程 {#find-a-cafe-and-compare-the-trip}

在真实半径范围内搜索，仅扩展入选 ID 的详情，再比较出行。Nearby 的 `keyword` 是 Google 地点类型，不是自由文本。

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now --min-rating 4
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --max-photos 0
mcp-gmap-rs distance-matrix --origins '["48.8584,2.2945"]' \
  --destinations '["48.8606,2.3376","48.8530,2.3499"]' --mode walking
```

自然语言意图用 [`search-places`](../reference/tools#maps_search_places)，需要详情及可选驾车距离时用 [`compare-places`](../reference/tools#maps_compare_places)，按类别探索用 [`explore-area`](../reference/tools#maps_explore_area)。单页搜索不是穷尽清单。保留 Google 来源链接、评论作者与 AI 披露。

## 规划停靠点并寻找沿途地点 {#plan-stops-and-find-a-detour}

```sh
mcp-gmap-rs plan-route \
  --stops '["48.8584,2.2945","48.8606,2.3376","48.8530,2.3499"]' \
  --mode walking --optimize false
mcp-gmap-rs search-along-route --text-query cafes \
  --origin '48.8584,2.2945' --destination '48.8606,2.3376' \
  --mode walking --max-results 3
```

[`directions`](../reference/tools#maps_directions)处理单次行程及时间/避让选项；[`plan-route`](../reference/tools#maps_plan_route)优化中间点时保留起终点。公共交通只接受总共两个停靠点。沿途搜索使用折线，不是以中点圆形搜索代替。其默认方式为步行，因此应始终明确选择期望的出行方式。

## 一次解析，查询当地环境 {#resolve-once-inspect-local-conditions}

```sh
mcp-gmap-rs geocode --address 'Eiffel Tower, Paris, France'
mcp-gmap-rs weather --latitude 48.8584 --longitude 2.2945 \
  --type forecast_daily --forecast-days 3
mcp-gmap-rs air-quality --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs pollen --latitude 48.8584 --longitude 2.2945 --days 3
mcp-gmap-rs timezone --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs elevation --locations '[{"latitude":48.8584,"longitude":2.2945}]'
```

复用坐标，避免重复解析。区分观测、预测和提供方建议，覆盖情况可能变化。[`reverse-geocode`](../reference/tools#maps_reverse_geocode)将坐标解析回地址；[`batch-geocode`](../reference/tools#maps_batch_geocode)一次解析最多 50 个地址，并保留逐地址错误。Time Zone 的可选时间戳使用 Unix 毫秒，海拔单位为米。

## 验证邮政地址 {#validate-a-postal-address}

```sh
mcp-gmap-rs validate-address --address-lines '["5 Avenue Anatole France"]' \
  --locality Paris --postal-code 75007 --region-code FR
```

检查判定结果、修正和未解决的组成部分。地址验证不同于地理编码，不保证投递成功。本实现中的 USPS CASS 需显式开启，并要求 `regionCode` 为 US 或 PR。

## 统计地点，而不是统计搜索结果 {#count-places-without-counting-search-hits}

```sh
mcp-gmap-rs area-insights \
  --area '{"circle":{"latLng":{"latitude":48.8584,"longitude":2.2945},"radius":500}}' \
  --included-types '["restaurant"]' --min-rating 4
```

聚合数量通常使用 Google 的字符串 int64 表示。仅当数量不超过 100 时可请求地点 ID；不会自动获取详情。需要显式分析可见度时，[`local-rank-tracker`](../reference/tools#maps_local_rank_tracker)测量网格上的关键词排名。先用一个关键词和 3×3 网格；排名不是评论评分，失败网格点必须保持可见。

## 匹配 GPS 数据并保存地图 {#match-gps-data-and-save-a-map}

有序轨迹使用 [`snap-to-roads`](../reference/tools#maps_snap_to_roads)，独立点使用 [`nearest-roads`](../reference/tools#maps_nearest_roads)。保留 `originalIndex`；插值添加的点可能没有该字段。不返回限速。

```sh
mcp-gmap-rs nearest-roads --points '[{"latitude":48.8584,"longitude":2.2945}]'
mcp-gmap-rs static-map --center '48.8584,2.2945' --zoom 14 --output area.png
```

[工具参考](../reference/tools)列出每个操作的精确边界、必填字段、默认值与结果格式。
