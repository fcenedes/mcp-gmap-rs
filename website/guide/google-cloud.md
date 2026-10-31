# Google Cloud setup

This executable authenticates **outbound calls to Google** with an API key or
OAuth 2.0 credentials. `MCP_HTTP_TOKEN` independently authenticates clients
connecting to your MCP HTTP server; choosing Google OAuth does not replace it.

## Enable services and billing

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
| [Roads API](https://developers.google.com/maps/documentation/roads/get-api-key) | `roads.googleapis.com` | API key |
| [Geocoding API v3](https://developers.google.com/maps/documentation/geocoding/guides-v3/get-api-key) | `geocoding-backend.googleapis.com` | API key |
| [Elevation API](https://developers.google.com/maps/documentation/elevation/get-api-key) | `elevation-backend.googleapis.com` | API key |
| [Time Zone API](https://developers.google.com/maps/documentation/timezone/get-api-key) | `timezone-backend.googleapis.com` | API key |
| [Maps Static API](https://developers.google.com/maps/documentation/maps-static/get-api-key) | `static-maps-backend.googleapis.com` | API key |

To enable all twelve with an administrator's `gcloud` login:

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

The CLI's `directions` and `distance-matrix` tools use **Routes API**, so the
legacy Directions and Distance Matrix APIs are not required. Geocoding still
uses v3 in this project; [OAuth support in Google's Geocoding v4](https://developers.google.com/maps/documentation/geocoding/geocoding-v4-overview) does not change
the authentication of these v3 calls. Composite tools can call multiple APIs.
Named-location resolution or fallback can require Geocoding even when the main
operation uses Places or Routes. Use coordinates when you need to avoid that
dependency, or retain a restricted key for the key-only endpoints.

## Create a restricted API key

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


## Next steps

For service-account JSON, ADC, local impersonation and GKE identity, follow
[Authentication](./authentication). Then choose [CLI](./cli) or [MCP](./mcp).
The [tool reference](../reference/tools) identifies each tool’s credential requirements.
