# Select tools for the task

These are logical tool names. Discover the actual schema and enabled selection
from the MCP host or `mcp-gmap-rs tools`. Newer tools may be absent in older
versions. Avoid loading full schemas for unrelated tools.

| Need | Tool choice and decision |
| --- | --- |
| Resolve a location | `maps_geocode` for an address; `maps_reverse_geocode` for coordinates. `maps_batch_geocode` handles up to 50 addresses and reports per-address failures. |
| Validate a postal address | `maps_validate_address` returns verdicts and corrections. Geocoding alone does not validate postal components; validation is not a delivery guarantee. USPS CASS requires explicit `regionCode` US or PR in this implementation. |
| Find places | `maps_search_places` for natural-language queries, `maps_search_nearby` for a bounded radius around a known center. Nearby's `keyword` is a Google place type. Use `center.isCoordinates=true` for an exact coordinate pair. |
| Inspect a known place | `maps_place_details` using its returned `placeId`. Request photos only when needed; retain author/source/disclosure metadata. |
| Explore or compare | `maps_explore_area` searches categories and expands top results. `maps_compare_places` compares detailed matches and optionally travel distance. Both can make many requests; keep result/category counts small. |
| Count places in an area | `maps_area_insights` with a circle, region or polygon and `includedTypes`. Counts are not inferred from a truncated search page. Optional IDs require a count at most 100; do not automatically fan out into details. |
| Travel between endpoints | `maps_directions`. Use the user's travel mode and explicit departure/arrival time when relevant. Tolls/highway avoidance applies to driving. |
| Compare many journeys | `maps_distance_matrix`. Keep the origins × destinations product small; inspect individual element errors/route availability. |
| Order stops on one route | `maps_plan_route` with 2–27 stops. Optimization preserves first/last endpoints and supports up to 25 intermediate stops, excluding transit. This is not fleet optimization. |
| Find a stop along a trip | `maps_search_along_route` searches using the route polyline. Avoid approximating a long route with one midpoint/radius. |
| Local conditions | `maps_weather` for current/daily/hourly weather, `maps_air_quality` for indices and optional pollutants, `maps_pollen` for 1–5 days of pollen. Keep coverage/date distinctions visible; health recommendations are Google's returned guidance. |
| Geography and local time | `maps_elevation` for height in metres; `maps_timezone` for UTC/DST offsets and local time. Its optional `timestamp` is Unix milliseconds. |
| Match GPS points to roads | `maps_snap_to_roads` for 2–100 ordered trace points; `maps_nearest_roads` for 1–100 independent points. Preserve `originalIndex`; interpolated points may not have it. These tools do not return speed limits. |
| Show a map | `maps_static_map` with center, markers or path. MCP returns image content; CLI can save a PNG. Preserve attribution. |
| Measure geographic search visibility | `maps_local_rank_tracker` for an explicitly requested keyword/place/grid analysis. A search rank is not a place's user rating. Start with a 3×3 grid and one keyword; inspect failed points before interpreting ARP/ATRP/SoLV. |

## Representative MCP requests

These objects describe a tool name and its arguments. Pass `arguments` to the
host's matching MCP tool, or serialize it as CLI `call --args` input. They are not
raw JSON-RPC messages and do not prescribe an MCP server name.

Find a small set of nearby cafes using exact coordinates:

```json
{"tool":"maps_search_nearby","arguments":{"center":{"value":"48.8584,2.2945","isCoordinates":true},"keyword":"cafe","radius":500,"openNow":true,"minRating":4}}
```

Plan a walking route through a user's ordered stops. Keep optimization off unless
reordering intermediate stops is wanted:

```json
{"tool":"maps_plan_route","arguments":{"stops":["48.8584,2.2945","48.8606,2.3376","48.8530,2.3499"],"mode":"walking","optimize":false}}
```

Count matching restaurants with Places Aggregate instead of counting search hits:

```json
{"tool":"maps_area_insights","arguments":{"area":{"circle":{"latLng":{"latitude":48.8584,"longitude":2.2945},"radius":500}},"includedTypes":["restaurant"],"minRating":4}}
```

Validate postal components without enabling USPS CASS implicitly:

```json
{"tool":"maps_validate_address","arguments":{"addressLines":["5 Avenue Anatole France"],"locality":"Paris","postalCode":"75007","regionCode":"FR"}}
```

Get a short pollen forecast:

```json
{"tool":"maps_pollen","arguments":{"latitude":48.8584,"longitude":2.2945,"days":3,"plantsDescription":false}}
```

Match a short ordered GPS trace to a road:

```json
{"tool":"maps_snap_to_roads","arguments":{"path":[{"latitude":-35.27801,"longitude":149.12958},{"latitude":-35.28032,"longitude":149.12907}],"interpolate":true}}
```

## Combine results without inventing data

For place recommendations, search with the user's constraints, keep the returned
IDs, expand only the shortlisted places, and compare travel only if it affects the
choice. Do not describe a search response as an exhaustive inventory.

For route recommendations, resolve ambiguous stops once, preserve known
coordinates, choose the travel mode/time, and use an along-route search for detours.
Include time-sensitive conditions only when relevant. A missing forecast or failed
matrix element must stay visible instead of becoming an estimated success.

For location conditions, resolve the requested location once and reuse it for
weather, air quality and pollen. Request only the conditions asked for; separate
observations from forecasts and recommendations returned by the provider.
