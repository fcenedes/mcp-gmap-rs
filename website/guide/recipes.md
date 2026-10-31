# Recipes

Combine small, explicit calls around one location or a shortlist of returned place IDs. These examples make potentially billable Google requests. Review [authentication](./authentication) first; composites may use more than one API and return partial errors.

## Find a cafe and compare the trip

Search within a real radius, expand only shortlisted IDs, then compare travel. Nearby’s `keyword` is a Google place type, not free text.

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now --min-rating 4
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --max-photos 0
mcp-gmap-rs distance-matrix --origins '["48.8584,2.2945"]' \
  --destinations '["48.8606,2.3376","48.8530,2.3499"]' --mode walking
```

Use [`search-places`](../reference/tools#maps_search_places) for natural-language intent, [`compare-places`](../reference/tools#maps_compare_places) for detailed matches with optional driving distance, or [`explore-area`](../reference/tools#maps_explore_area) for category-based exploration. A search page is not an exhaustive inventory. Preserve Google source links, review authors and AI disclosures.

## Plan stops and find a detour

```sh
mcp-gmap-rs plan-route \
  --stops '["48.8584,2.2945","48.8606,2.3376","48.8530,2.3499"]' \
  --mode walking --optimize false
mcp-gmap-rs search-along-route --text-query cafes \
  --origin '48.8584,2.2945' --destination '48.8606,2.3376' \
  --mode walking --max-results 3
```

[`directions`](../reference/tools#maps_directions) handles one journey with time/avoidance options; [`plan-route`](../reference/tools#maps_plan_route) preserves endpoints when optimizing intermediates. Transit accepts only two total stops. Along-route search follows a polyline instead of substituting a midpoint circle. It defaults to walking, so always choose the intended travel mode.

## Resolve once, inspect local conditions

```sh
mcp-gmap-rs geocode --address 'Eiffel Tower, Paris, France'
mcp-gmap-rs weather --latitude 48.8584 --longitude 2.2945 \
  --type forecast_daily --forecast-days 3
mcp-gmap-rs air-quality --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs pollen --latitude 48.8584 --longitude 2.2945 --days 3
mcp-gmap-rs timezone --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs elevation --locations '[{"latitude":48.8584,"longitude":2.2945}]'
```

Reuse coordinates instead of repeating resolution. Keep observations, forecasts and provider recommendations distinct; coverage can vary. [`reverse-geocode`](../reference/tools#maps_reverse_geocode) resolves coordinates back to an address; [`batch-geocode`](../reference/tools#maps_batch_geocode) resolves up to 50 addresses with per-address errors. Time Zone’s optional timestamp uses Unix milliseconds; elevation is in metres.

## Validate a postal address

```sh
mcp-gmap-rs validate-address --address-lines '["5 Avenue Anatole France"]' \
  --locality Paris --postal-code 75007 --region-code FR
```

Inspect the verdict, corrections and unresolved components. Address validation is distinct from geocoding and does not guarantee delivery. USPS CASS is opt-in and requires `regionCode` US or PR in this implementation.

## Count places without counting search hits

```sh
mcp-gmap-rs area-insights \
  --area '{"circle":{"latLng":{"latitude":48.8584,"longitude":2.2945},"radius":500}}' \
  --included-types '["restaurant"]' --min-rating 4
```

Aggregate counts normally use Google’s string int64 representation. Optional place IDs are available only when the count is at most 100; details are not fetched automatically. For explicitly requested visibility analysis, [`local-rank-tracker`](../reference/tools#maps_local_rank_tracker) measures keyword ranks across a grid. Start with one keyword and a 3×3 grid; ranks are not review ratings, and failed points must stay visible.

## Match GPS data and save a map

Use [`snap-to-roads`](../reference/tools#maps_snap_to_roads) for an ordered trace and [`nearest-roads`](../reference/tools#maps_nearest_roads) for independent points. Preserve `originalIndex`; interpolation adds points that may not have one. Speed limits are not returned.

```sh
mcp-gmap-rs nearest-roads --points '[{"latitude":48.8584,"longitude":2.2945}]'
mcp-gmap-rs static-map --center '48.8584,2.2945' --zoom 14 --output area.png
```

The [tool reference](../reference/tools) lists exact bounds, required fields, defaults and result formats for every operation.
