# Media services: implementation and provider limits

Checked against Google's official documentation on 2026-11-01. No live requests,
uploads, publication or deletion were made while developing these adapters.
Context7 was queried first; it did not contain the Map Tiles or Street View
Publish REST documentation, so the primary sources below were used directly.

## Map Tiles

`maps_map_tiles` exposes nine operations on `tile.googleapis.com`:

| Operation | Method and path | Required inputs |
| --- | --- | --- |
| `create_session` | POST `/v1/createSession` | `sessionOptions` with mapType, language, region |
| `tile_2d` | GET `/v1/2dtiles/{zoom}/{x}/{y}` | session, zoom, x, y |
| `viewport` | GET `/tile/v1/viewport` | session, zoom, north, south, east, west |
| `root_3d` | GET `/v1/3dtiles/root.json` | none |
| `resource_3d` | GET Google-issued `/v1/3dtiles/...` resource | resourceUrl |
| `streetview_pano_ids` | POST `/v1/streetview/panoIds` | session, locations |
| `streetview_metadata` | GET `/v1/streetview/metadata` | session; panoId OR latitude+longitude |
| `streetview_thumbnail` | GET `/v1/streetview/thumbnail` | session, panoId |
| `streetview_tile` | GET `/v1/streetview/tiles/{zoom}/{x}/{y}` | session, panoId, zoom, x, y |

Session and metadata JSON remain native; successful JSON undergoes shared
credential redaction. 3D resource input accepts the fixed HTTPS Google host,
designated path namespace, no userinfo/fragment/custom port/encoded path, and
discards incoming API-key query parameters. Redirects are disabled by the shared
client. Images and GLB resources are bounded binary responses; HTTP cache and
entity-tag headers are retained. This executable does not render or stitch tiles.

Map Tiles supports API keys or OAuth with `cloud-platform` scope. Google's current
guide explicitly permits service-account server-to-server authentication and
requires `X-Goog-User-Project` for the billed project:
[authentication](https://developers.google.com/maps/documentation/tile/get-api-key).

**License constraint:** Map Tiles is for map visualization. Google's policy
expressly excludes image analysis, machine interpretation, object identification,
geodata extraction/resale and offline use. This is material for agents: these
images must not become vision-model analysis input. Prefetching, indexing,
storage and caching are limited by the Google agreement; clients must respect
Cache-Control/ETag. Display Google Maps and provider attributions. Street View
viewers must display panorama copyright and the supplied Report a problem link;
3D renderers must display tile credits. EEA terms and available content differ.

Primary sources:

- [Session contract and options](https://developers.google.com/maps/documentation/tile/session_tokens)
- [Roadmap tile coordinates](https://developers.google.com/maps/documentation/tile/roadmap)
- [Viewport endpoint, bounds and attribution](https://developers.google.com/maps/documentation/tile/2d-tiles-overview)
- [Street View endpoints, query bounds, panorama lifetime and required credits](https://developers.google.com/maps/documentation/tile/streetview)
- [Photorealistic 3D root, renderer and attribution requirements](https://developers.google.com/maps/documentation/tile/3d-tiles)
- [Map Tiles usage, caching and license policy](https://developers.google.com/maps/documentation/tile/policies)

## Street View Static

`maps_streetview` offers GET `/maps/api/streetview/metadata` (JSON) and GET
`/maps/api/streetview` (JPEG) on `maps.googleapis.com`. Exactly one location or
panorama ID is required. The implementation bounds images to 640×640, defaults
to 640×400, forwards optional camera/search/signature parameters, and defaults
`return_error_code=true` to avoid gray placeholders. Metadata responses retain
copyright, dates, native statuses and panorama IDs. Panorama IDs are unstable.

This implementation uses API-key authentication for Static Street View. It
forwards a caller-computed signature but does not hold a URL-signing secret or
generate signatures; accounts requiring signed requests need a valid signature
for the exact request. The signature is appended last, after the API key. Sign
this exact encoded unsigned path and query, including the generated defaults;
spaces use `+` and UTF-8 bytes use percent encoding. Image parameter order is
location, radius, source, pano, size, heading, fov, pitch, return_error_code, key;
absent optional fields are skipped. Defaults are size=640x400 and
return_error_code=true. Metadata excludes all image-only fields. A fixture checks
both exact outgoing encodings with a Unicode/space input and signature last.
This adapter does not validate a supplied signature cryptographically.
[Google's signing contract](https://developers.google.com/maps/documentation/streetview/digital-signature).
Premium image sizes are not implemented. Metadata is
currently free and does not consume imagery quota; image requests are billed.
Display and caching restrictions apply, including no print use of Street View
imagery. The MCP/CLI interface is not a grant to redistribute provider imagery.

Primary sources:

- [Imagery request parameters and error behavior](https://developers.google.com/maps/documentation/streetview/request-streetview)
- [Metadata request and native response](https://developers.google.com/maps/documentation/streetview/metadata)
- [Street View Static policies](https://developers.google.com/maps/documentation/streetview/policies)
- [Google geo imagery guidelines](https://about.google/brand-resource-center/products-and-services/geo-guidelines/)

## Street View Publish

`maps_streetview_publish` implements all 14 methods listed by the official v1
REST discovery document, plus a bounded media upload operation:

| Operation | Method and path |
| --- | --- |
| `list` | GET `/v1/photos` |
| `get` | GET `/v1/photo/{photoId}` |
| `start_upload` | POST `/v1/photo:startUpload` |
| `upload` | POST the returned fixed-origin `/media/user/{account}/photo/{ref}` UploadRef |
| `create` | POST `/v1/photo` |
| `update` | PUT `/v1/photo/{photoId}` with updateMask |
| `delete` | DELETE `/v1/photo/{photoId}` |
| `batch_get` | GET `/v1/photos:batchGet` with repeated photoIds |
| `batch_update` | POST `/v1/photos:batchUpdate` |
| `batch_delete` | POST `/v1/photos:batchDelete` |
| `list_sequences` | GET `/v1/photoSequences` |
| `get_sequence` | GET `/v1/photoSequence/{sequenceId}` |
| `start_sequence_upload` | POST `/v1/photoSequence:startUpload` |
| `create_sequence` | POST `/v1/photoSequence` with inputType |
| `delete_sequence` | DELETE `/v1/photoSequence/{sequenceId}` |

All paths use `streetviewpublish.googleapis.com`. List/filter/view/page arguments
are forwarded; pagination is explicit. Complex Photo, PhotoSequence and batch
update bodies use Google's native JSON schema, while the tool validates required
operation inputs, matching resource IDs, update masks and UploadRef origins.
Sequence processing returns a native long-running Operation; get_sequence reads
its current state. No automatic mutation or publication is performed.

Publishing is **end-user OAuth**, with the scope
`https://www.googleapis.com/auth/streetviewpublish`. Google attributes imagery to
the authenticated user's account. A service-account JSON key and IAM grants do
not provide the necessary user's consent or imagery ownership. The executable
requires explicit authorized-user credentials for this service; obtaining consent
and the refresh credential is an external OAuth-client setup step.

The photo flow is start_upload → actual media POST → create with uploadReference.
The tool accepts base64 media decoded to **at most 8 MiB**, an implementation bound
rather than Google's product maximum. Larger media, particularly videos, needs
an external Google-supported upload flow; its returned UploadRef can still be
used with create_sequence. Chunked/resumable upload is not implemented. A photo
must satisfy Google's 360-degree Photo Sphere/XMP metadata and acceptance criteria.
Updating photo pixels is unsupported by Google. No mutation, upload or publication
was tested live; supplied service-account credentials cannot cover this flow.

All writes disable automatic retries, so ambiguous failures do not duplicate
publishing actions. Fixture tests exercise every operation, actual decoded byte
POST, user bearer authentication, binary/cache metadata, URL-origin restrictions,
invalid argument rejection, attribution preservation and non-retrying mutation
errors. The parent task runs the shared Cargo build/tests.

Primary sources:

- [REST methods](https://developers.google.com/streetview/publish/reference/rest)
- [Machine-readable v1 discovery document](https://streetviewpublish.googleapis.com/$discovery/rest?version=v1)
- [User authorization and exact scope](https://developers.google.com/streetview/publish/authorizing)
- [Three-step actual media upload, metadata, PUT update and acceptance requirements](https://developers.google.com/streetview/publish/first-app)
- [Photo native JSON schema](https://developers.google.com/streetview/publish/reference/rest/v1/photo)
- [PhotoSequence schema, input types and processing](https://developers.google.com/streetview/publish/reference/rest/v1/photoSequence/create)
