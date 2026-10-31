# Geo service implementation references

Verified against Google primary documentation on 2026-11-01. No live Google
requests or credential reads were made for this implementation.

| Tool | Method and host | Access and behavior |
| --- | --- | --- |
| `maps_speed_limits` | `GET roads.googleapis.com/v1/speedLimits` | GPS path or repeated road-segment `placeId`, at most 100. `units=KPH/MPH`. Restricted Asset Tracking feature; per-returned-limit billing. API key in this implementation. |
| `maps_autocomplete` | `POST places.googleapis.com/v1/places:autocomplete` | Current native fields include `includeFutureOpeningBusinesses`. Bias/restriction and circle/rectangle are mutually exclusive. API key or OAuth. Caller owns session tokens. |
| `maps_geolocate` | `POST www.googleapis.com/geolocation/v1/geolocate` | Native radio/WiFi body; API key query parameter is required even for POST. IP fallback can locate server egress and is opt-out using `considerIp:false`. |
| `maps_solar` | `GET solar.googleapis.com/v1/buildingInsights:findClosest`, `/v1/dataLayers:get`, `/v1/geoTiff:get` | API key or cloud-platform OAuth. Raster downloads accept an ID only and always use the fixed Solar endpoint. Data-layer URLs are credential-free and temporary. |
| `maps_aerial_view` | `GET aerialview.googleapis.com/v1/videos:lookupVideo`, `/v1/videos:lookupVideoMetadata` | US postal addresses/known video IDs. Metadata omits playback URLs and is free-to-call; lookup is billable. No media downloading. API key in this implementation. |
| `maps_render_aerial_video` | `POST aerialview.googleapis.com/v1/videos:renderVideo` | Explicit upstream mutation, asynchronous; no automatic retry and no waiting/downloading. |

## Roads

- [Speed limits](https://developers.google.com/maps/documentation/roads/speed-limits)
- [Usage and billing](https://developers.google.com/maps/documentation/roads/usage-and-billing)

Speed-limit access is not granted just by enabling Roads. Google documents the
Asset Tracking license requirement and retained access for certain transitioned
Premium Plan projects. Speed data can be estimated/outdated; variable limits
return the maximum automobile limit. A 403 may also be caused by API enablement,
key restrictions or permissions. The upstream reason remains visible, sanitized.

The general Roads billing page mentions OAuth, while the specific speed-limit
guide documents a required API key. This implementation retains its existing
Roads key-auth behavior rather than claiming a live-verified OAuth capability.

## Places and Geolocation

- [Autocomplete REST reference](https://developers.google.com/maps/documentation/places/web-service/reference/rest/v1/places/autocomplete)
- [Autocomplete guide and session rules](https://developers.google.com/maps/documentation/places/web-service/place-autocomplete)
- [Geolocation native request/response](https://developers.google.com/maps/documentation/geolocation/requests-geolocation)

Autocomplete supports five primary types, 15 region codes and 50 km circles.
`inputOffset` counts Unicode characters, not UTF-8 bytes. Special type collections
must be used alone. Tokens belong to one session and credentials in that session
must belong to one project. The adapter preserves predictions and Google metadata.

The Geolocation body deliberately accepts Google's complete extensible native
payload. Radio-specific field validation is delegated to Google. WiFi positioning
needs distinct stationary access points; reserved/local MAC ranges are excluded.

## Solar

- [Building insights method](https://developers.google.com/maps/documentation/solar/reference/rest/v1/buildingInsights/findClosest)
- [Data layers method](https://developers.google.com/maps/documentation/solar/reference/rest/v1/dataLayers/get)
- [GeoTIFF method](https://developers.google.com/maps/documentation/solar/reference/rest/v1/geoTiff/get)
- [Data-layer resource and current imagery-quality enum](https://developers.google.com/maps/documentation/solar/reference/rest/v1/dataLayers)
- [Coverage](https://developers.google.com/maps/documentation/solar/coverage)
- [Expanded-coverage experiment](https://developers.google.com/maps/documentation/solar/reference/rest/v1/Experiment)
- [EEA adjustments](https://developers.google.com/maps/comms/eea/solar)

Current imagery qualities are `HIGH`, `MEDIUM`, `BASE`. Rasters above 100m radius
must respect the requested pixel scale; above 175m, views including monthly flux
or hourly shade are forbidden. Experimental expanded coverage is pre-GA.
Building search may find no building within approximately 50m. Coverage and EEA
content differ. Binary downloads are bounded by the executable's 16 MiB response
limit; callers may need a smaller radius or coarser pixels.

## Aerial View

- [Lookup](https://developers.google.com/maps/documentation/aerial-view/reference/rest/v1/videos/lookupVideo)
- [Metadata](https://developers.google.com/maps/documentation/aerial-view/reference/rest/v1/videos/lookupVideoMetadata)
- [Render](https://developers.google.com/maps/documentation/aerial-view/reference/rest/v1/videos/renderVideo)
- [Playback URI restrictions](https://developers.google.com/maps/documentation/aerial-view/lookup-video)
- [Policies and attribution](https://developers.google.com/maps/documentation/aerial-view/policies)
- [Key setup](https://developers.google.com/maps/documentation/aerial-view/get-api-key)

Playback URLs are short-lived, must be obtained for each display, and must not be
downloaded, stored or cached. Video IDs have a documented caching exception.
The tools preserve original Google metadata/URLs and never fetch video content.
Rendering is separate from lookups so MCP can advertise the upstream mutation.
