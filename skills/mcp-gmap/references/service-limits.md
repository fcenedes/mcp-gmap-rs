# Service access and content limits

These limits come from Google's products and agreements, independently of this
executable's MIT license. Inspect the installed schema and returned error; a
403 may mean API/key/IAM restrictions, not necessarily a missing license.

| Service | Decision that changes agent behavior |
| --- | --- |
| Roads Speed Limits | Requires restricted Asset Tracking entitlement; enabling Roads is insufficient. Do not infer truck, real-time or guaranteed legal limits from automobile data. Each returned speed-limit element can be billable. |
| Map Tiles | Visualization only. Google prohibits image analysis, machine interpretation, object detection, geodata extraction and offline use. Do not feed returned map tiles to a vision model. Preserve map/provider/3D/panorama credits and Report a problem links. Honor returned cache headers and Google terms; this server does not render a compliant viewer for you. |
| Aerial View | Supported US addresses only. Return fresh playback URLs and attribution; never download, store or cache video. `metadata` avoids playback lookup; `maps_render_aerial_video` explicitly queues a render and is not a read. |
| Street View Static | Display attribution. Panorama IDs can change. Image saving through CLI does not grant redistribution, print or indefinite caching rights. |
| Street View Publish | End-user OAuth with `streetviewpublish` scope and an explicit authorized_user credential file. A service-account key cannot supply user consent/ownership. Upload must satisfy Google Photo Sphere/XMP/360-degree rules. Updates cannot replace photo pixels. Larger/resumable/video upload is external; executable uploads cap decoded media at 8 MiB. |
| Datasets | OAuth/IAM; GCS import additionally needs object access. Data-driven styling is the supported dataset usage. Import is asynchronous; inspect native status/errors. No v1 version-list endpoint is advertised. |
| Route Optimization | OAuth/IAM. Native model fields are validated by Google. Explicit batch/LRO/URI calls can create billable jobs and use GCS objects. Experimental variants have separate access/terms; cancellation/list/delete of operations are not supported by Google here. `get_operation` returns pending/failed status without claiming success. |
| Solar | Geographic, imagery-quality and EEA content differences apply. Expanded coverage is experimental. Data-layer URLs are temporary; use fixed-host ID download rather than arbitrary URL fetching. |
| Weather minute forecasts/maps | Experimental/pre-GA, limited map coverage, required attribution, at most one hour caching, and restrictions on recreating a primary-purpose weather app/model. Do not present previews as generally available. |

All calls share attempt/time/response bounds. Pagination is explicit. Native
request bodies accept Google's documented fields; tool-level unknown arguments
are rejected. The server never changes Google IAM or API enablement for you.

Maintained endpoint and policy links are in the packaged full tool manual and
[coverage matrix](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/service-research/coverage.md).
