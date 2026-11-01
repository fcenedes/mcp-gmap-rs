# Google Cloud setup and authentication

This executable authenticates **outbound calls to Google** with an API key or
OAuth 2.0 credentials. `MCP_HTTP_TOKEN` independently authenticates clients
connecting to your MCP HTTP server; choosing Google OAuth does not replace it.

## 1. Create a project, link billing and enable APIs

In the [Google Cloud console](https://console.cloud.google.com/), select or create
a project, then link an active billing account under **Billing**. Enable the
APIs you need under **APIs & Services → Library**. Enabling an API and allowing it
in an API key's restrictions are separate steps. Calls can incur charges.

The following table describes the endpoints used by this version of the project:

| Google API | Service to enable | Authentication used here |
| --- | --- | --- |
| [Places API (New)](https://developers.google.com/maps/documentation/places/web-service/get-api-key) | `places.googleapis.com` | API key or OAuth |
| [Routes API](https://developers.google.com/maps/documentation/routes/get-api-key) | `routes.googleapis.com` | API key or OAuth |
| [Weather API](https://developers.google.com/maps/documentation/weather/get-api-key) | `weather.googleapis.com` | API key or OAuth |
| [Air Quality API](https://developers.google.com/maps/documentation/air-quality/get-api-key) | `airquality.googleapis.com` | API key or OAuth |
| [Address Validation API](https://developers.google.com/maps/documentation/address-validation/get-api-key) | `addressvalidation.googleapis.com` | API key or OAuth |
| [Places Aggregate API](https://developers.google.com/maps/documentation/places-aggregate/get-api-key) | `areainsights.googleapis.com` | API key or OAuth |
| [Pollen API](https://developers.google.com/maps/documentation/pollen/get-api-key) | `pollen.googleapis.com` | API key |
| [Roads API](https://developers.google.com/maps/documentation/roads/get-api-key) | `roads.googleapis.com` | API key; speed limits additionally require Asset Tracking access |
| [Geocoding API v3](https://developers.google.com/maps/documentation/geocoding/guides-v3/get-api-key) | `geocoding-backend.googleapis.com` | API key |
| [Elevation API](https://developers.google.com/maps/documentation/elevation/get-api-key) | `elevation-backend.googleapis.com` | API key |
| [Time Zone API](https://developers.google.com/maps/documentation/timezone/get-api-key) | `timezone-backend.googleapis.com` | API key |
| [Maps Static API](https://developers.google.com/maps/documentation/maps-static/get-api-key) | `static-maps-backend.googleapis.com` | API key |
| [Geolocation API](https://developers.google.com/maps/documentation/geolocation/get-api-key) | `geolocation.googleapis.com` | API key |
| [Solar API](https://developers.google.com/maps/documentation/solar/get-api-key) | `solar.googleapis.com` | API key or OAuth |
| [Aerial View API](https://developers.google.com/maps/documentation/aerial-view/get-api-key) | `aerialview.googleapis.com` | API key |
| [Map Tiles API](https://developers.google.com/maps/documentation/tile/get-api-key) | `tile.googleapis.com` | API key or OAuth |
| [Street View Static API](https://developers.google.com/maps/documentation/streetview/get-api-key) | `street-view-image-backend.googleapis.com` | API key; optional caller-provided URL signature |
| [Maps Platform Datasets API](https://developers.google.com/maps/documentation/datasets/request) | `mapsplatformdatasets.googleapis.com` | OAuth only, resource IAM required |
| [Route Optimization API](https://developers.google.com/maps/documentation/route-optimization/cloud-setup) | `routeoptimization.googleapis.com` | OAuth only, resource IAM required |
| [Street View Publish API](https://developers.google.com/streetview/publish/authorizing) | `streetviewpublish.googleapis.com` | End-user OAuth only; service accounts are rejected |

To enable the twenty server-side products with an administrator's `gcloud` login:

```sh
export PROJECT_ID='your-project-id'
gcloud services enable --project="$PROJECT_ID" \
  places.googleapis.com routes.googleapis.com \
  weather.googleapis.com airquality.googleapis.com \
  geocoding-backend.googleapis.com elevation-backend.googleapis.com \
  timezone-backend.googleapis.com static-maps-backend.googleapis.com \
  addressvalidation.googleapis.com areainsights.googleapis.com \
  pollen.googleapis.com roads.googleapis.com \
  geolocation.googleapis.com solar.googleapis.com aerialview.googleapis.com \
  tile.googleapis.com street-view-image-backend.googleapis.com \
  mapsplatformdatasets.googleapis.com routeoptimization.googleapis.com \
  streetviewpublish.googleapis.com
```

The CLI's `directions` and `distance-matrix` tools use **Routes API**, so the
legacy Directions and Distance Matrix APIs are not required. Geocoding still
uses v3 in this project; [OAuth support in Google's Geocoding v4](https://developers.google.com/maps/documentation/geocoding/geocoding-v4-overview) does not change
the authentication of these v3 calls. Composite tools can call multiple APIs.
Named-location resolution or fallback can require Geocoding even when the main
operation uses Places or Routes. Use coordinates when you need to avoid that
dependency, or retain a restricted key for the key-only endpoints.

## 2. API key setup

1. Open [Google Maps Platform → Keys & Credentials](https://console.cloud.google.com/google/maps-apis/credentials)
   in the selected project and choose **Create credentials → API key**.
2. Select only the APIs from the table that this instance will use. For an
   existing key, edit its API restrictions and save them.
3. Set the application restriction to **IP addresses** for this server-side
   application. Allow the public egress IP addresses Google will see. Kubernetes
   pod/private IPs are normally not the public source address; a fixed Cloud NAT
   or other egress address makes this restriction predictable.
4. Store the key in an ignored `.env`, environment injection or a Kubernetes
   Secret. See [Google's key restriction guidance](https://developers.google.com/maps/api-security-best-practices#restrict-your-api-keys).

Example `.env`:

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

`api-key` is the default. Supplying a service-account path alone does not change
authentication mode. Discovery does not contact Google; use a real call to
verify project billing, API enablement and restrictions:

```sh
mcp-gmap-rs geocode --address 'Paris, France'
```

## 3. Service accounts and OAuth 2.0

In OAuth mode, Places (New), Routes, Weather, Air Quality, Address Validation,
Places Aggregate, Solar, Map Tiles, Datasets and Route Optimization requests receive an
OAuth **access token** and an `X-Goog-User-Project` header. This project requests
the `https://www.googleapis.com/auth/cloud-platform` scope documented by those
APIs. Street View Publish uses a separate end-user credential flow and the
`https://www.googleapis.com/auth/streetviewpublish` scope described below. Google’s Rust auth library obtains, caches and refreshes the tokens.
`GOOGLE_MAPS_TIMEOUT` also bounds credential initialization and token acquisition.
No manually pasted access token or application OAuth consent screen is needed
for the service-account flow. See [Maps server-side OAuth guidance](https://developers.google.com/maps/api-security-best-practices#use-oauth-for-server-side-apps).

Create a dedicated service account and grant it permission to consume services
on the project used for Maps billing/quota:

```sh
export PROJECT_ID='your-project-id'
export SA_EMAIL="mcp-gmap@${PROJECT_ID}.iam.gserviceaccount.com"
gcloud iam service-accounts create mcp-gmap --project="$PROJECT_ID" \
  --display-name='MCP Google Maps'
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:${SA_EMAIL}" \
  --role='roles/serviceusage.serviceUsageConsumer'
```

The Service Usage Consumer role includes `serviceusage.services.use` for quota
and billing. It does not grant the ability to enable services or create keys;
those setup steps need a separately authorized administrator. Enable the Maps
APIs in the quota project as in step 1. See [Service Usage permissions](https://docs.cloud.google.com/service-usage/docs/access-control).

Choose one credential source below. The same settings work with CLI, stdio and
HTTP. Configuration follows the usual file → environment → flag precedence.

### A. Mounted service-account JSON key

If your organization permits JSON keys, open **IAM & Admin → Service Accounts**,
select the account, then **Keys → Add key → Create new key → JSON**. Save it
outside the source checkout. Alternatively:

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

For a mounted file, the equivalent `.env` is:

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
# Optional, for Geocoding v3 / Elevation / Time Zone / Static Maps / Pollen / Roads /
# Geolocation / Aerial View / Street View Static:
# GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

Set the credential path only when using a readable JSON file; do not set it to
an empty string or to the JSON contents.
Relative paths are resolved from the working directory. File values are used
directly, without mutating the process environment. The loader also accepts ADC
`external_account`, `impersonated_service_account` and `authorized_user` JSON
configurations. Restart the process after changing a credential file.

Service-account keys are long-lived credentials. Prefer the keyless options
below when available; keep JSON keys out of Git and image layers, and rotate or
delete unused keys. Google documents [key creation and organization policies](https://docs.cloud.google.com/iam/docs/keys-create-delete).

### B. Application Default Credentials without a JSON key

Leave `GOOGLE_APPLICATION_CREDENTIALS` unset and set:

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
```

The official auth library uses [Application Default Credentials (ADC)](https://docs.cloud.google.com/docs/authentication/application-default-credentials):
local ADC credentials when present, otherwise the attached workload/service
account through the metadata server. Credential discovery is lazy: `tools`,
`schema` and help work without fetching a token. An OAuth API call needs a quota
project and valid credentials.

For local development with service-account impersonation, enable the IAM
Credentials API and grant your developer account Token Creator on the service
account, then create local ADC credentials:

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

This uses a short-lived service-account token for Maps calls without downloading
its private key. See [service-account impersonation](https://docs.cloud.google.com/iam/docs/service-account-impersonation).

### C. Kubernetes and GKE workload identity

For generic Kubernetes, mount the credential JSON as a Secret file and set the
three OAuth variables in option A. The file must be readable by UID 10001 used
by the container. Mount it read-only and keep `MCP_HTTP_TOKEN` in its own runtime
Secret. [The container guide](containers.md) explains the base Deployment.

On GKE, prefer [Workload Identity Federation for GKE](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/workload-identity)
with a Kubernetes ServiceAccount linked to the IAM service account. Enable it
for your cluster/node pool first (Autopilot already enables it), then:

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

These commands assume the cluster and IAM service account are in `PROJECT_ID`
and the base Deployment is in `default`. Adjust project/namespace names for
your installation. Do not set `GOOGLE_APPLICATION_CREDENTIALS` for this metadata
flow. The pod obtains access tokens through the GKE metadata server. Outside
GKE, an `external_account` credential configuration plus the provider's projected
identity token can be used with Google's Workload Identity Federation setup.

For OAuth-only operation, remove the Google API key from the runtime Secret and
enable only tools/inputs whose calls stay on OAuth-supported APIs. Keep the key
when using the key-only endpoints from the table or composite flows that need
them. OAuth failures on supported APIs fail explicitly; they never silently
downgrade to the key.

### D. Resource IAM for Datasets and Route Optimization

Service Usage Consumer permits quota consumption; it does **not** grant access
to datasets or optimization jobs. Grant the least permissions needed in the
**resource project** named by `parent`/`name`, which can differ from the quota
project. Both services require OAuth even when an API key is configured.

| Service/operation | Additional resource permissions |
| --- | --- |
| Datasets list | `mapsplatformdatasets.datasets.list` on the project |
| Datasets get/status/errors | `mapsplatformdatasets.datasets.get` |
| Datasets create/update/delete | `mapsplatformdatasets.datasets.create`, `.update`, `.delete` respectively |
| Datasets import/export | `mapsplatformdatasets.datasets.import` / `.export`; GCS import also needs `storage.objects.get` on the source object |
| Route Optimization optimize/validate | `routeoptimization.locations.use`; `roles/routeoptimization.viewer` includes access |
| Route Optimization batch/long-running/URI | `routeoptimization.operations.create`; use `roles/routeoptimization.editor` |
| Route Optimization operation status | `routeoptimization.operations.get` |

See the [Datasets REST permission references](https://developers.google.com/maps/documentation/datasets/reference/rest)
and [Route Optimization IAM setup](https://developers.google.com/maps/documentation/route-optimization/cloud-setup).
For example, after creating the dedicated service account:

```sh
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:${SA_EMAIL}" \
  --role='roles/routeoptimization.viewer'
```

Use Editor instead only if this instance needs to create batch jobs. Datasets
imports and route batch/URI requests operate on caller-specified Cloud Storage
objects; grant the documented storage permissions on those objects separately.
The executable submits a job only for an explicit write operation and never
polls automatically. HTTP/tool timeouts do not cancel cloud work. Google currently
marks Route Optimization cancellation/list/delete operations unsupported.

### E. Street View Publish requires a consenting user

A service-account JSON key cannot own a person's Street View contributions.
Create an OAuth client and obtain user consent for
`https://www.googleapis.com/auth/streetviewpublish` using Google's
[authorization flow](https://developers.google.com/streetview/publish/authorizing).
Save the resulting refresh credentials as an ignored `authorized_user` JSON file
and configure it explicitly with `GOOGLE_APPLICATION_CREDENTIALS`, OAuth mode and
a quota project. Implicit ADC and service-account credentials are intentionally
rejected for Publish; the executable refreshes tokens but does not host a browser
consent callback. A refresh token must have the required consent already granted.

Use a separate process/credential configuration for publishing when other tools
use a workload service account. Start with read-only list/get operations. Upload,
create, update and delete explicitly change the user's public contributions.
Photos/sequences must meet Google's Photo Sphere, XMP, rights and acceptance
requirements. Upload bodies are capped at 8 MiB; resumable uploads are not
implemented. Larger media must use Google's external upload flow before passing
its UploadRef to this executable.

### F. API access is separate from licensing and content rights

Roads speed limits require the [Asset Tracking entitlement](https://developers.google.com/maps/documentation/roads/speed-limits);
enabling Roads does not grant it. Weather minute forecasts and weather-map tiles
are [experimental/pre-GA](https://developers.google.com/maps/documentation/weather/minute-forecast):
Google requires attribution, limits caching to one hour and prohibits recreating
a product whose primary purpose is providing weather information. Map imagery
is for visualization under the applicable product terms; an agent tool does not
grant imagery interpretation, extraction, bulk-download or redistribution rights.

See the [coverage and limitations matrix](service-research/coverage.md) before
enabling tools broadly. The tool reference describes operation-specific billing,
coverage, attribution and mutation behavior.

## 4. Verify and troubleshoot

```sh
mcp-gmap-rs --auth-mode oauth search-places --query 'coffee in Paris'
mcp-gmap-rs --auth-mode oauth weather --latitude 48.8584 --longitude 2.2945
```

These are real, potentially billable calls. Check the effective mode and quota
project when testing; OAuth billing uses `GOOGLE_CLOUD_QUOTA_PROJECT`, while
key-authenticated calls use the key's project. The projects may differ.

Use `mcp-gmap-rs doctor` for offline configuration checks, or `doctor --live` for
explicitly billable API probes. The [production guide](production.md) describes
the report and tool filtering.

With OAuth variables exported, run the six opt-in live acceptance tests with:

```sh
cargo test --locked --test live live_oauth_ -- --ignored --test-threads=1
```

These tests explicitly remove the API key from each child process, so a passing
test exercises OAuth. They cover Places, Routes, Weather, Air Quality, Address
Validation and Places Aggregate and
require their APIs enabled. They do not load `.env`; export the OAuth settings
or run in a configured workload. Ordinary tests/CI use local token/API fixtures
and do not require a service account or contact Google.

| Symptom | Check |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | The selected operation reaches a key-only endpoint, or API-key mode is selected. |
| OAuth authentication failed | Credential path/type, key validity, workload identity/impersonation binding, IAM Credentials API when used, token-endpoint connectivity and clock accuracy. Application errors omit credential contents and tokens. |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | Set the Maps consumer project ID/number for OAuth calls. |
| Google `403` / API disabled | Enable the exact service in the consumer project; ensure billing and Service Usage permission. |
| API-key IP denial | Match restrictions to the server or cluster's public egress IP. |
| Datasets / Route Optimization OAuth required | Select OAuth mode; a restricted API key alone cannot authenticate these services. |
| Datasets or optimization IAM denied | Grant the operation-specific resource permission in the parent/name project, plus quota-project Service Usage permission. |
| Roads Speed Limits permission denied | Check Asset Tracking entitlement as well as enablement/key restrictions; not every 403 is a license failure. |
| Street View Publish end-user OAuth required | Use an explicitly mounted authorized_user JSON credential with prior streetviewpublish consent; service-account IAM does not replace user consent. |
| Token timeout | Allow access to Google token/STS/IAM endpoints or the workload metadata server; review `GOOGLE_MAPS_TIMEOUT`. |
| HTTP `401` from this MCP server | Send its `MCP_HTTP_TOKEN`; Google service-account tokens authenticate a separate outbound connection. |
