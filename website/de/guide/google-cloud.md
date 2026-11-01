# Google Cloud einrichten

Dieses Programm authentifiziert **ausgehende Aufrufe an Google** mit einem API-Schlüssel oder OAuth-2.0-Zugangsdaten. `MCP_HTTP_TOKEN` authentifiziert davon unabhängig Clients Ihres MCP-HTTP-Servers; Google OAuth ersetzt dieses Token nicht.

## Dienste und Abrechnung aktivieren

Wählen oder erstellen Sie in der [Google Cloud Console](https://console.cloud.google.com/) ein Projekt und verknüpfen Sie unter **Billing** ein aktives Abrechnungskonto. Aktivieren Sie die benötigten APIs unter **APIs & Services → Library**. API-Aktivierung und die Freigabe in den Einschränkungen eines API-Schlüssels sind zwei getrennte Schritte. Aufrufe können Kosten verursachen.

Diese Tabelle beschreibt die von dieser Projektversion verwendeten Endpunkte:

| Google-API | Zu aktivierender Dienst | Hier verwendete Authentifizierung |
| --- | --- | --- |
| [Places API (New)](https://developers.google.com/maps/documentation/places/web-service/get-api-key) | `places.googleapis.com` | API-Schlüssel oder OAuth |
| [Routes API](https://developers.google.com/maps/documentation/routes/get-api-key) | `routes.googleapis.com` | API-Schlüssel oder OAuth |
| [Weather API](https://developers.google.com/maps/documentation/weather/get-api-key) | `weather.googleapis.com` | API-Schlüssel oder OAuth |
| [Air Quality API](https://developers.google.com/maps/documentation/air-quality/get-api-key) | `airquality.googleapis.com` | API-Schlüssel oder OAuth |
| [Address Validation API](https://developers.google.com/maps/documentation/address-validation/get-api-key) | `addressvalidation.googleapis.com` | API-Schlüssel oder OAuth |
| [Places Aggregate API](https://developers.google.com/maps/documentation/places-aggregate/get-api-key) | `areainsights.googleapis.com` | API-Schlüssel oder OAuth |
| [Pollen API](https://developers.google.com/maps/documentation/pollen/get-api-key) | `pollen.googleapis.com` | API-Schlüssel |
| [Roads API](https://developers.google.com/maps/documentation/roads/get-api-key) | `roads.googleapis.com` | API-Schlüssel |
| [Geocoding API v3](https://developers.google.com/maps/documentation/geocoding/guides-v3/get-api-key) | `geocoding-backend.googleapis.com` | API-Schlüssel |
| [Elevation API](https://developers.google.com/maps/documentation/elevation/get-api-key) | `elevation-backend.googleapis.com` | API-Schlüssel |
| [Time Zone API](https://developers.google.com/maps/documentation/timezone/get-api-key) | `timezone-backend.googleapis.com` | API-Schlüssel |
| [Maps Static API](https://developers.google.com/maps/documentation/maps-static/get-api-key) | `static-maps-backend.googleapis.com` | API-Schlüssel |

Mit einer administrativ berechtigten `gcloud`-Anmeldung aktivieren Sie alle zwölf Dienste so:

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

Die CLI-Werkzeuge `directions` und `distance-matrix` verwenden die **Routes API**; die älteren APIs Directions und Distance Matrix werden nicht benötigt. Geocoding verwendet in diesem Projekt weiterhin v3. Die [OAuth-Unterstützung in Googles Geocoding v4](https://developers.google.com/maps/documentation/geocoding/geocoding-v4-overview) verändert die Authentifizierung dieser v3-Aufrufe nicht. Zusammengesetzte Werkzeuge können mehrere APIs verwenden. Die Auflösung benannter Orte oder ein Fallback kann Geocoding benötigen, obwohl die Hauptoperation Places oder Routes verwendet. Nutzen Sie Koordinaten, um diese Abhängigkeit zu vermeiden, oder behalten Sie einen eingeschränkten Schlüssel für Endpunkte mit ausschließlicher Schlüsselunterstützung.

## Eingeschränkten API-Schlüssel erstellen

1. Öffnen Sie im gewählten Projekt [Google Maps Platform → Keys & Credentials](https://console.cloud.google.com/google/maps-apis/credentials) und wählen Sie **Create credentials → API key**.
2. Wählen Sie ausschließlich die APIs aus der Tabelle, die diese Instanz verwenden soll. Bei einem vorhandenen Schlüssel ändern Sie die API-Einschränkungen und speichern sie.
3. Stellen Sie für diese serverseitige Anwendung die Anwendungseinschränkung auf **IP addresses**. Erlauben Sie die öffentlichen Ausgangs-IP-Adressen, die Google sieht. Kubernetes-Pod- oder private IP-Adressen sind normalerweise nicht die öffentliche Quelladresse; eine feste Cloud-NAT- oder andere Ausgangsadresse macht die Einschränkung vorhersehbar.
4. Speichern Sie den Schlüssel in einer ignorierten `.env`, über Umgebungsinjektion oder in einem Kubernetes-Secret. Siehe [Googles Hinweise zur Schlüsseleinschränkung](https://developers.google.com/maps/api-security-best-practices#restrict-your-api-keys).

Beispiel für `.env`:

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

`api-key` ist der Standard. Ein Dienstkontopfad allein ändert den Authentifizierungsmodus nicht. Die Werkzeugerkennung kontaktiert Google nicht; prüfen Sie Projektabrechnung, API-Aktivierung und Einschränkungen mit einem tatsächlichen Aufruf:

```sh
mcp-gmap-rs geocode --address 'Paris, France'
```

## Nächste Schritte

Informationen zu Dienstkonto-JSON, ADC, lokaler Identitätsübernahme und GKE-Identität finden Sie unter [Authentifizierung](./authentication). Wählen Sie anschließend [CLI](./cli) oder [MCP](./mcp). Die [Werkzeugreferenz](../reference/tools) nennt die Anforderungen an Zugangsdaten für jedes Werkzeug.


## Zusätzliche Dienste und Grenzen {#expanded-services}

Solar und Map Tiles unterstützen auch OAuth. Datasets und Route Optimization erfordern OAuth sowie die jeweiligen IAM-Berechtigungen. Street View Publish verlangt eine ausdrücklich gewählte authorized_user-Datei und die Einwilligung für den Scope streetviewpublish.

| Dienst | API | Authentifizierung |
| --- | --- | --- |
| Geolocation API | `geolocation.googleapis.com` | API-Schlüssel |
| Solar API | `solar.googleapis.com` | API-Schlüssel / OAuth |
| Aerial View API | `aerialview.googleapis.com` | API-Schlüssel |
| Map Tiles API | `tile.googleapis.com` | API-Schlüssel / OAuth |
| Street View Static API | `street-view-image-backend.googleapis.com` | API-Schlüssel |
| Maps Platform Datasets API | `mapsplatformdatasets.googleapis.com` | OAuth / IAM |
| Route Optimization API | `routeoptimization.googleapis.com` | OAuth / IAM |
| Street View Publish API | `streetviewpublish.googleapis.com` | Nutzer-OAuth mit Einwilligung; Dienstkonten werden abgelehnt |

Speed Limits benötigt beschränkten Asset-Tracking-Zugang. Aerial View gilt für unterstützte US-Adressen und verbietet das Herunterladen oder Zwischenspeichern von Videos. Map Tiles dient der Visualisierung; maschinelle Analyse und Datenextraktion sind untersagt. Weather minute/map sind experimentell, mit höchstens einer Stunde Cache und zusätzlichen Nutzungsgrenzen. Die Werkzeugreferenzen erläutern Bedingungen, Quellenangaben und Grenzen.

[Google: Speed Limits](https://developers.google.com/maps/documentation/roads/speed-limits), [Map Tiles](https://developers.google.com/maps/documentation/tile/policies), [Aerial View](https://developers.google.com/maps/documentation/aerial-view/policies), [Street View Publish](https://developers.google.com/streetview/publish/authorizing).
