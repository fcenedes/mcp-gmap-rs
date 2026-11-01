# Authentifizierung

Konfigurieren Sie ausgehende Google-Zugangsdaten getrennt von den Zugangsdaten der MCP-HTTP-Clients. API-Schlüssel sind der Standardmodus; ein Dienstkontopfad allein aktiviert OAuth nicht.

## API-Schlüssel und OAuth-Abdeckung

| Endpunktfamilie | API-Schlüssel | OAuth in diesem Programm |
| --- | --- | --- |
| Places (New), Routes, Weather, Air Quality, Address Validation, Places Aggregate | Ja | Ja |
| Geocoding v3, Elevation, Time Zone, Maps Static, Pollen, Roads | Ja | Nein |

Die Auflösung benannter Orte und zusammengesetzte Aufrufe können einen Schlüssel benötigen, selbst wenn ihre Hauptoperation OAuth unterstützt. Exakte Koordinaten vermeiden unnötiges Geocoding. OAuth-Fehler führen niemals stillschweigend zu einem API-Schlüssel-Fallback. Unter [Google Cloud einrichten](./google-cloud) erfahren Sie, welche Dienste zu aktivieren und wie Schlüssel einzuschränken sind.

## OAuth mit Dienstkonto

Im OAuth-Modus erhalten Aufrufe an Places (New), Routes, Weather, Air Quality, Address Validation und Places Aggregate ein OAuth-**Zugriffstoken** und einen Header `X-Goog-User-Project`. Das Projekt fordert den von diesen APIs dokumentierten Bereich `https://www.googleapis.com/auth/cloud-platform` an. Googles Rust-Authentifizierungsbibliothek beschafft, speichert und erneuert die Tokens. `GOOGLE_MAPS_TIMEOUT` begrenzt auch die Initialisierung der Zugangsdaten und die Tokenbeschaffung. Für den Dienstkontoablauf sind weder manuell eingefügte Zugriffstokens noch ein OAuth-Zustimmungsbildschirm erforderlich. Siehe [Maps-Hinweise zu serverseitigem OAuth](https://developers.google.com/maps/api-security-best-practices#use-oauth-for-server-side-apps).

Erstellen Sie ein eigenes Dienstkonto und gewähren Sie ihm auf dem für Maps-Abrechnung und Kontingente verwendeten Projekt die Dienstnutzungsberechtigung:

```sh
export PROJECT_ID='your-project-id'
export SA_EMAIL="mcp-gmap@${PROJECT_ID}.iam.gserviceaccount.com"
gcloud iam service-accounts create mcp-gmap --project="$PROJECT_ID" \
  --display-name='MCP Google Maps'
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:${SA_EMAIL}" \
  --role='roles/serviceusage.serviceUsageConsumer'
```

Die Rolle Service Usage Consumer enthält `serviceusage.services.use` für Kontingente und Abrechnung. Sie erlaubt weder das Aktivieren von Diensten noch das Erstellen von Schlüsseln; dafür ist ein gesondert berechtigter Administrator nötig. Aktivieren Sie die Maps-APIs im Kontingentprojekt anhand der [Google-Cloud-Anleitung](./google-cloud). Siehe [Service-Usage-Berechtigungen](https://docs.cloud.google.com/service-usage/docs/access-control).

Wählen Sie eine der folgenden Zugangsdatenquellen. Dieselben Einstellungen gelten für CLI, stdio und HTTP. Die Priorität lautet Datei → Umgebung → Option.

### A. Eingebundener JSON-Dienstkontoschlüssel {#a-mounted-service-account-json-key}

Falls Ihre Organisation JSON-Schlüssel zulässt, öffnen Sie **IAM & Admin → Service Accounts**, wählen Sie das Konto und dann **Keys → Add key → Create new key → JSON**. Speichern Sie die Datei außerhalb des Quellcode-Checkouts. Alternativ:

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

Für eine eingebundene Datei lautet die entsprechende `.env`:

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
# Optional, for Geocoding v3 / Elevation / Time Zone / Static Maps / Pollen / Roads:
# GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

Setzen Sie den Zugangsdatenpfad nur für eine lesbare JSON-Datei, niemals auf eine leere Zeichenkette oder den JSON-Inhalt. Relative Pfade beziehen sich auf das Arbeitsverzeichnis. Dateiwerte werden direkt verwendet, ohne die Prozessumgebung zu verändern. Der Lader akzeptiert auch ADC-JSON-Konfigurationen vom Typ `external_account`, `impersonated_service_account` und `authorized_user`. Starten Sie den Prozess nach einer Änderung der Zugangsdaten neu.

Dienstkontoschlüssel sind langlebige Zugangsdaten. Bevorzugen Sie nach Möglichkeit die schlüssellosen Varianten unten; halten Sie JSON-Schlüssel aus Git und Image-Schichten heraus und rotieren oder löschen Sie ungenutzte Schlüssel. Google dokumentiert [Schlüsselerstellung und Organisationsrichtlinien](https://docs.cloud.google.com/iam/docs/keys-create-delete).

### B. Application Default Credentials ohne JSON-Schlüssel {#b-application-default-credentials-without-a-json-key}

Lassen Sie `GOOGLE_APPLICATION_CREDENTIALS` sowohl in der Prozessumgebung als auch in der gewählten Dotenv-Datei oder dem Kubernetes-Secret/ConfigMap ungesetzt. Ein Shell-`unset` allein entfernt keinen Wert aus einer Dateiebene; entfernen Sie den Eintrag, statt einen leeren Pfad zu setzen. Konfigurieren Sie anschließend:

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
```

Die offizielle Bibliothek verwendet [Application Default Credentials (ADC)](https://docs.cloud.google.com/docs/authentication/application-default-credentials): vorhandene lokale ADC-Zugangsdaten, andernfalls das angehängte Workload-/Dienstkonto über den Metadatenserver. Die Erkennung erfolgt verzögert: `tools`, `schema` und Hilfe funktionieren ohne Tokenbeschaffung. Ein OAuth-API-Aufruf benötigt ein Kontingentprojekt und gültige Zugangsdaten.

Aktivieren Sie für lokale Entwicklung mit Dienstkonto-Identitätsübernahme die IAM Credentials API, gewähren Sie Ihrem Entwicklerkonto Token Creator auf dem Dienstkonto und erstellen Sie lokale ADC-Zugangsdaten:

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

Damit werden kurzlebige Dienstkonto-Tokens für Maps-Aufrufe verwendet, ohne dessen privaten Schlüssel herunterzuladen. Siehe [Dienstkonto-Identitätsübernahme](https://docs.cloud.google.com/iam/docs/service-account-impersonation).

### C. Kubernetes- und GKE-Workload-Identität {#c-kubernetes-and-gke-workload-identity}

Binden Sie bei allgemeinem Kubernetes die JSON-Zugangsdaten als Secret-Datei ein und setzen Sie die drei OAuth-Variablen aus Variante A. Die Datei muss für die im Container verwendete UID 10001 lesbar sein. Binden Sie sie schreibgeschützt ein und bewahren Sie `MCP_HTTP_TOKEN` in einem eigenen Laufzeit-Secret auf. Die [Containeranleitung](./deployment) erläutert das Basis-Deployment.

Bevorzugen Sie auf GKE [Workload Identity Federation for GKE](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/workload-identity) mit einem an das IAM-Dienstkonto gebundenen Kubernetes-ServiceAccount. Aktivieren Sie diese Funktion zuerst für Cluster und Node-Pool; bei Autopilot ist sie bereits aktiviert. Anschließend:

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

Die Befehle setzen voraus, dass Cluster und IAM-Dienstkonto in `PROJECT_ID` liegen und das Basis-Deployment im Namespace `default` läuft. Passen Sie Projekt und Namespace an Ihre Installation an. Setzen Sie `GOOGLE_APPLICATION_CREDENTIALS` für diesen Metadatenablauf nicht. Der Pod erhält Zugriffstokens über den GKE-Metadatenserver. Außerhalb von GKE kann eine `external_account`-Konfiguration zusammen mit dem projizierten Identitätstoken des Anbieters über Googles Workload Identity Federation verwendet werden.

Entfernen Sie für einen reinen OAuth-Betrieb den Google-API-Schlüssel aus dem Laufzeit-Secret und aktivieren Sie ausschließlich Werkzeuge und Eingaben, deren Aufrufe auf OAuth-fähigen APIs bleiben. Behalten Sie den Schlüssel für die ausschließlich schlüsselbasierten Endpunkte der Tabelle oder für entsprechende zusammengesetzte Abläufe. OAuth-Fehler werden auf unterstützten APIs ausdrücklich gemeldet und führen niemals stillschweigend zur Schlüsselverwendung.

## Prüfen und Fehler beheben

```sh
mcp-gmap-rs --auth-mode oauth search-places --query 'coffee in Paris'
mcp-gmap-rs --auth-mode oauth weather --latitude 48.8584 --longitude 2.2945
```

Dies sind reale, möglicherweise kostenpflichtige Aufrufe. Prüfen Sie beim Testen den wirksamen Modus und das Kontingentprojekt. OAuth rechnet über `GOOGLE_CLOUD_QUOTA_PROJECT` ab; schlüsselbasierte Aufrufe über das Projekt des Schlüssels. Diese Projekte können unterschiedlich sein.

`mcp-gmap-rs doctor` prüft die lokale Konfiguration; `doctor --live` führt ausdrücklich kostenpflichtige API-Proben aus. Die [Produktionsanleitung](./production) beschreibt Bericht und Werkzeugfilter.

Mit exportierten OAuth-Variablen starten Sie die sechs freiwilligen Live-Akzeptanztests so:

```sh
cargo test --locked --test live live_oauth_ -- --ignored --test-threads=1
```

Die Tests entfernen den API-Schlüssel ausdrücklich aus jedem Kindprozess; ein bestandener Test prüft somit OAuth. Sie decken Places, Routes, Weather, Air Quality, Address Validation und Places Aggregate ab und benötigen aktivierte APIs. Sie laden keine `.env`; exportieren Sie die OAuth-Einstellungen oder verwenden Sie eine konfigurierte Workload. Normale Tests und CI verwenden lokale Token-/API-Fixtures und benötigen weder Dienstkonto noch Google-Zugriff.

| Symptom | Prüfung |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | Die Operation erreicht einen ausschließlich schlüsselbasierten Endpunkt oder der API-Schlüsselmodus ist gewählt. |
| OAuth-Authentifizierung fehlgeschlagen | Pfad und Typ der Zugangsdaten, Schlüsselgültigkeit, Workload-/Identitätsübernahmebindung, gegebenenfalls IAM Credentials API, Token-Endpunktverbindung und Systemzeit prüfen. Anwendungsfehler enthalten weder Zugangsdateninhalte noch Tokens. |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | Die ID oder Nummer des Maps-Verbraucherprojekts für OAuth-Aufrufe setzen. |
| Google `403` / API deaktiviert | Den konkreten Dienst im Verbraucherprojekt aktivieren; Abrechnung und Service-Usage-Berechtigung prüfen. |
| IP-Ablehnung des API-Schlüssels | Die Einschränkung muss zur öffentlichen Ausgangs-IP des Servers oder Clusters passen. |
| Token-Zeitüberschreitung | Zugriff auf Google-Token-/STS-/IAM-Endpunkte oder den Workload-Metadatenserver erlauben und `GOOGLE_MAPS_TIMEOUT` prüfen. |
| HTTP `401` von diesem MCP-Server | Dessen `MCP_HTTP_TOKEN` senden; Google-Dienstkonto-Tokens authentifizieren eine andere ausgehende Verbindung. |


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
