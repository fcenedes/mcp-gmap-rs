# Traffic segments along a route

Set `trafficSegments: true` on `maps_directions` or `maps_plan_route` to receive
Google's traffic categories aligned to the returned route and leg polylines.
The option is off by default. It extends these two existing tools; it does not
add a separate tool or request posted speed limits.

This wrapper supports traffic segments only with `mode: "driving"`. Walking,
bicycling and transit requests with this option are rejected before routing.
Google's upstream API also supports traffic-aware polylines for two-wheelers,
but this executable does not expose that travel mode.

## CLI and MCP

Request a route using exact coordinates:

```sh
mcp-gmap-rs call maps_directions --args '{"origin":"48.8584,2.2945","destination":"48.8606,2.3376","mode":"driving","trafficSegments":true}'
```

Request a route through several stops:

```sh
mcp-gmap-rs call maps_plan_route --args '{"stops":["48.8584,2.2945","48.8606,2.3376","48.8530,2.3499"],"mode":"driving","optimize":false,"trafficSegments":true}'
```

The equivalent MCP `tools/call` request is:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "maps_directions",
    "arguments": {
      "origin": "48.8584,2.2945",
      "destination": "48.8606,2.3376",
      "mode": "driving",
      "trafficSegments": true
    }
  }
}
```

Credentials and billing requirements are the same as for the underlying Routes
API. Address and place-name inputs can require the usual location-resolution
calls; coordinate inputs avoid that extra resolution. An optional
`departure_time` uses RFC 3339 with a timezone. Without it, Google chooses the
departure time according to its request defaults.

## Choose geometry quality independently

Both tools accept `polylineQuality: "HIGH_QUALITY"` or `"OVERVIEW"`.
Use the CLI flag `--polyline-quality HIGH_QUALITY` when a consumer needs a more
detailed road path, including when it does not request traffic segments:

```sh
mcp-gmap-rs directions --origin '37.422,-122.084' \
  --destination '37.7749,-122.4194' --polyline-quality HIGH_QUALITY
```

An explicit quality is honored. Without it, ordinary route calls retain Google's
`OVERVIEW` default and `trafficSegments:true` defaults to `HIGH_QUALITY`.
High quality contains more provider points and can increase response size and
latency. It does not guarantee lane-level or survey accuracy. The executable
preserves Google's returned geometry; it does not interpolate points to imply
missing road detail. Traffic indexes always refer to the exact returned geometry,
including when the caller explicitly chooses `OVERVIEW` with traffic enabled.
Quality selection by itself does not request `TRAFFIC_ON_POLYLINE`; that separate
extra computation remains opt-in and has its documented billing implications.
See Google's [polyline quality values](https://developers.google.com/maps/documentation/routes/reference/rest/v2/TopLevel/computeRoutes#PolylineQuality).

## What is requested from Google

When enabled, the executable sends `TRAFFIC_AWARE` routing, the extra computation
`TRAFFIC_ON_POLYLINE`, `ENCODED_POLYLINE` encoding and `HIGH_QUALITY` geometry
unless an explicit `polylineQuality` overrides that default.
It requests matching route and leg polylines and their
`travelAdvisory.speedReadingIntervals`. `TRAFFIC_AWARE` also remains compatible
with intermediate-waypoint optimization. Google does not supply traffic
intervals at the step level. See the official
[traffic-polyline guide](https://developers.google.com/maps/documentation/routes/traffic_on_polylines)
and [waypoint optimization requirements](https://developers.google.com/maps/documentation/routes/opt-way).

`maps_directions` retains the native routes, legs, polylines and advisories,
adding normalized `segments` and `segmentMetadata`. `maps_plan_route` keeps its
existing flattened itinerary format and preserves the matching polylines and
native advisories alongside the normalized data. Google's `staticDuration` and
any `fallbackInfo` remain available when returned; a fallback is not proof of
current traffic coverage.

| Tool | Complete-route fields | Per-leg fields |
| --- | --- | --- |
| `maps_directions` | `routes[i].segments`, `routes[i].segmentMetadata`, `routes[i].polyline` | `routes[i].legs[j].segments`, `routes[i].legs[j].segmentMetadata`, `routes[i].legs[j].polyline` |
| `maps_plan_route` | Top-level `segments`, `segmentMetadata`, `polyline` | `legs[j].segments`, `legs[j].segmentMetadata`, `legs[j].polyline` |

`maps_plan_route` has no `routes` array. In both tools the raw
`travelAdvisory.speedReadingIntervals` is retained beside the corresponding
segments.

## Segment fields

| Field | Meaning |
| --- | --- |
| `startPolylinePointIndex` | Zero-based start boundary, inclusive. A native omitted start is normalized to `0`, following Google's protobuf default. |
| `endPolylinePointIndex` | End boundary under Google's start-inclusive/end-exclusive interval convention. It refers to the matching polyline's point indexes. |
| `trafficCategory` | Native `NORMAL`, `SLOW` or `TRAFFIC_JAM`; `null` when unavailable. Future provider category strings are preserved rather than guessed. |
| `roadCategory` | `null`: this response provides no general structured road classification. |
| `speedLimit` | `null`: Routes traffic intervals do not provide posted legal limits. |
| `trafficObservedAt` | `null`: Google does not provide a traffic-observation timestamp for these intervals. |

These are traffic categories, not numerical vehicle speeds. `NORMAL` describes
Google's traffic assessment; it is not a claim that a vehicle is travelling at
the posted limit. The adapter does not assign application-specific speed
multipliers or derive a legal limit from traffic. The
[SpeedReadingInterval reference](https://developers.google.com/maps/documentation/routes/reference/rest/v2/SpeedReadingInterval)
defines the native categories.

Missing traffic, missing categories and uncovered gaps are never filled with
`NORMAL`. A missing observation time stays unknown. A route's distance and
duration also do not establish the speed of an individual segment.

## Route and leg index scopes

Each route's segments index that route's own `polyline.encodedPolyline`. Each
leg's segments independently index that leg's own `polyline.encodedPolyline`.
Leg indexes restart from zero; they cannot be used directly against the complete
route geometry. Do not join route and leg interval arrays into a single index
space or assume one traffic segment equals one navigation step.

Google describes intervals as `[start, end)` with adjacent intervals sharing a
boundary. For geometry reconstruction, include the end boundary vertex: an
interval with `start=1` and `end=3` uses points `1, 2, 3` to draw its two edges.
Thus, for `M` vertices, full edge coverage finishes at boundary `M−1`, not `M`.
This slicing guidance is an inference from Google's documented full-coverage
example and shared boundaries, rather than a separate provider field. See
[RouteTravelAdvisory](https://developers.google.com/maps/documentation/routes/reference/rest/v2/RouteTravelAdvisory).

## Provenance and availability

`segmentMetadata` describes each route or leg scope independently:

| Field | Meaning |
| --- | --- |
| `retrievedAt` | Local UTC timestamp when the executable processed the fetched response. It is **not** Google's observation time or a guarantee of traffic freshness. |
| `requestedDepartureTime` | Departure time sent in the request, or `null` when none was supplied. This is a request parameter, not a traffic observation. |
| `polylineScope` | `ROUTE` or `LEG`; identifies the geometry to which the indexes apply. |
| `routeIndex` | Zero-based index of the containing route. |
| `legIndex` | Zero-based index of the containing leg; omitted for route scope. |
| `indexConvention` | `START_INCLUSIVE_END_EXCLUSIVE`; indexes are zero-based. |
| `trafficSource` | `GOOGLE_ROUTES`; the traffic category comes from Google's advisory. |
| `trafficStatus` | `AVAILABLE`, `NOT_RETURNED` or `PARTIAL`; inspect this rather than assuming complete data. |
| `roadCategoryStatus`, `speedLimitStatus` | `NOT_PROVIDED_BY_ROUTES`; reserved null values are not failed Roads lookups. |

`NOT_RETURNED` means there are no normalized intervals. `PARTIAL` marks gaps,
incomplete coverage, missing categories or unrecognized category strings;
`AVAILABLE` requires complete coverage with known categories. Invalid geometry,
overlapping intervals and invalid index bounds cause a response error. A
route-level result does not imply leg-level traffic is available, or the reverse.
Retain unknown values and inspect the native advisory for provider-specific detail.

The following illustrative `maps_directions` response excerpt shows a
three-vertex polyline and one traffic interval. Other fields and leg scopes are
omitted for brevity; these are example values, not a live Google response. For
`maps_plan_route`, the fields shown inside `routes[0]` appear at the top level.

```json
{
  "routes": [
    {
      "polyline": {
        "encodedPolyline": "_p~iF~ps|U_ulLnnqC_mqNvxq`@"
      },
      "travelAdvisory": {
        "speedReadingIntervals": [
          {"endPolylinePointIndex": 2, "speed": "SLOW"}
        ]
      },
      "segments": [
        {
          "startPolylinePointIndex": 0,
          "endPolylinePointIndex": 2,
          "trafficCategory": "SLOW",
          "roadCategory": null,
          "speedLimit": null,
          "trafficObservedAt": null
        }
      ],
      "segmentMetadata": {
        "retrievedAt": "2026-11-01T11:00:00.000Z",
        "requestedDepartureTime": null,
        "polylineScope": "ROUTE",
        "routeIndex": 0,
        "indexConvention": "START_INCLUSIVE_END_EXCLUSIVE",
        "trafficSource": "GOOGLE_ROUTES",
        "trafficStatus": "AVAILABLE",
        "roadCategoryStatus": "NOT_PROVIDED_BY_ROUTES",
        "speedLimitStatus": "NOT_PROVIDED_BY_ROUTES"
      }
    }
  ]
}
```

## Posted limits, road classes and timestamps

The Google Routes response used here has no general road-category field, actual
posted speed limit, numerical interval speed or provider observation timestamp
for these traffic intervals. Navigation instructions are display text; the
adapter does not mine them to invent road classes. Specialized upstream road
advisories, when available, do not establish a complete general classification.
See the [Compute Routes response reference](https://developers.google.com/maps/documentation/routes/reference/rest/v2/TopLevel/computeRoutes).

For actual posted limits, call `maps_speed_limits` separately. Google's Roads
Speed Limits service requires **Asset Tracking access**; simply enabling Roads
API is insufficient. Its data can be missing or outdated and it bills separately.
Traffic intervals contain no Roads road-segment place IDs, so the adapter makes
no implicit Roads call and establishes no automatic one-to-one join between the
two datasets. See [Roads speed limits and access restrictions](https://developers.google.com/maps/documentation/roads/speed-limits).

## Cost and content restrictions

Traffic-aware routing by itself triggers the current Compute Routes **Pro**
SKU. Requesting traffic information on polylines triggers **Enterprise** billing,
which is why `trafficSegments` requires an explicit opt-in. Rates depend on the
current price list and project region; consult Google's
[Routes SKU feature triggers](https://developers.google.com/maps/billing-and-pricing/sku-details/routes)
and [usage and billing guide](https://developers.google.com/maps/documentation/routes/usage-and-billing).

Normalizing a response does not relax Google's content license. Preserve Google
Maps attribution, follow the applicable display and caching rules, and account
for EEA terms or regional functionality differences. See
[Routes policies and attributions](https://developers.google.com/maps/documentation/routes/policies).
