# Produktionsbetrieb

Diese Anleitung behandelt v0.3.x. Alle Google-Werkzeuge verwenden dieselbe Implementierung für CLI, MCP stdio und Streamable HTTP.

## Natives TLS und gegenseitiges TLS {#native-tls-and-mutual-tls}

Der HTTP-Listener kann über rustls mit TLS 1.2/1.3 direkt HTTPS bereitstellen:

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3443 \
  --tls-cert /etc/mcp-gmap/tls/tls.crt \
  --tls-key /etc/mcp-gmap/tls/tls.key \
  --allowed-host maps.example.com:3443
```

Stellen Sie `MCP_HTTP_TOKEN` getrennt über die Umgebung oder ein Secret bereit. Clients verbinden sich mit `https://maps.example.com:3443/mcp`, prüfen das Serverzertifikat und senden `Authorization: Bearer <token>`. Zertifikatsdateien enthalten das PEM-Endzertifikat mit anschließender Zwischenzertifikatskette; der private Schlüssel muss dazu passen. Das Programm stellt keine Zertifikate aus und implementiert kein ACME. Verwenden Sie für Ausstellung und Erneuerung Ihre PKI, cert-manager oder ein TLS-Gateway.

Mit `--tls-client-ca /etc/mcp-gmap/tls/client-ca.pem` fordern Sie ein von dieser CA signiertes Clientzertifikat. Natives mTLS gilt für jeden Pfad, einschließlich `/health` und `/metrics`, und ersetzt das HTTP-Bearer-Token nicht. Auf TLS-Ebene wird jede gültige Clientidentität unter den konfigurierten Vertrauenswurzeln akzeptiert; es gibt weder werkzeugbezogene Zertifikatsberechtigungen noch Sperrlistenprüfungen.

Ersetzen Sie unter Linux/macOS die Dateien atomar und senden Sie `SIGHUP`, um Zertifikat, Schlüssel und optionale Client-CA gemeinsam neu zu laden. Bestehende Verbindungen behalten ihren TLS-Zustand; neue Handshakes verwenden die Ersatzdateien. Schlägt das Laden fehl, bleibt die bisherige Konfiguration aktiv und der Fehler wird ohne Schlüsselmaterial protokolliert. Eingebundene Kubernetes-Secret-Dateien lassen sich nach ihrer Projektionsaktualisierung so neu laden. Andere Einstellungen erfordern weiterhin einen Neustart.

Binden Sie Schlüssel für die Container-UID/GID 10001 schreibgeschützt mit passenden Leserechten ein; verwenden Sie für projizierte Secrets eine geeignete Gruppe und `fsGroup`. Bei nativem TLS müssen Kubernetes-HTTP-Probes `scheme: HTTPS` verwenden. Kubelet-HTTP-Probes können kein mTLS-Clientzertifikat liefern: Verwenden Sie dafür eine TCP-Probe und beachten Sie, dass sie nur den Listener, nicht TLS oder Google prüft.

## Kubernetes-TLS-Gateway

Das [Gateway-Manifest](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/gateway.yaml) liefert ein HTTPS-Gateway und eine HTTPRoute für das [Basis-Deployment](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml). Installieren Sie zuerst die Gateway-API-CRDs und einen konformen Controller. Ersetzen Sie `your-gateway-class` und `maps.example.com` und stellen Sie im selben Namespace ein `kubernetes.io/tls`-Secret namens `mcp-gmap-tls` bereit. Das Beispiel nutzt stabile Gateway-/HTTPRoute-v1-Felder und HTTPRoute-Zeitlimits, die seit Gateway API v1.2 unterstützt werden. Prüfen Sie die Unterstützung Ihres Controllers.

```sh
kubectl create secret tls mcp-gmap-tls --cert=/secure/fullchain.pem --key=/secure/key.pem
kubectl set env deployment/mcp-gmap-rs \
  MCP_HTTP_ALLOWED_HOSTS='maps.example.com,mcp-gmap-rs:3000,localhost:3000,127.0.0.1:3000'
kubectl apply -f deploy/gateway.yaml
kubectl get gateway,httproute
```

Hinterlegen Sie die entsprechende Host-Freigabe auch dauerhaft in der ConfigMap oder Deployment-Quelle. Ergänzen Sie bei Bedarf tatsächliche Browser-Origins mit `MCP_HTTP_ALLOWED_ORIGINS`. Die Route veröffentlicht nur `/mcp`; Health und Metriken bleiben auf dem privaten Service. Autorisierungs-, MCP-Sitzungs- und Protokollheader müssen unverändert passieren. Deaktivieren Sie Antwortpufferung und Caching und konfigurieren Sie Streaming-/Idle-Zeitlimits im Controller. Die Route deaktiviert Anfrage- und Backend-Anfragefristen; setzen Sie am Gateway passende Verbindungs- und Idle-Limits für Ihre Clients.

Diese Konfiguration terminiert TLS am Gateway und verwendet HTTP zum ClusterIP-Backend. Soll auch die Gateway-zu-Pod-Verbindung verschlüsselt werden, verwenden Sie natives TLS mit einer vom Controller unterstützten BackendTLSPolicy oder einem Mesh. Beschränken Sie direkten Pod-/Service-Zugriff über die Netzwerkrichtlinien Ihres Clusters. TLS bietet keine MCP-OAuth-Anmeldung: Das Programm verwendet ein gemeinsames Bearer-Token und eine Google-Identität pro Prozess.

Siehe die [Gateway-TLS-Anleitung](https://gateway-api.sigs.k8s.io/guides/user-guides/tls/) und die [HTTPRoute-Zeitlimit-Anleitung](https://gateway-api.sigs.k8s.io/guides/http-timeouts/).

## Limits, Wiederholungen und Fristen {#limits-retries-and-deadlines}

| Variable | Standard | Option | Bedeutung |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | Google-HTTP-Versuche pro Werkzeugaufruf, 1–10000, einschließlich Wiederholungen |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | Zusätzliche Versuche pro Google-Anfrage, 0–5 |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | Gesamtfrist des Werkzeugs in Sekunden, 1–3600 |
| `MCP_HTTP_RATE_LIMIT` | `120` | HTTP `--rate-limit` | Authentifizierte HTTP-Anfragen an `/mcp` pro Minute und Prozess; 0 deaktiviert |
| `MCP_METRICS_ENABLED` | `false` | HTTP `--metrics [true\|false]` | Geschützte Prometheus-Metriken unter `/metrics` aktivieren |
| `MCP_LOG_FORMAT` | `text` | `--log-format text\|json` | Diagnoseformat auf stderr |
| `MCP_TLS_CERT` | ungesetzt | HTTP `--tls-cert` | PEM-Datei der Serverzertifikatskette |
| `MCP_TLS_KEY` | ungesetzt | HTTP `--tls-key` | Passende private PEM-Serverschlüsseldatei |
| `MCP_TLS_CLIENT_CA` | ungesetzt | HTTP `--tls-client-ca` | PEM-CA-Bündel; fordert Clientzertifikate |

Es gilt Standardwerte → optionale Dotenv-Datei → Umgebung → Optionen. TLS-Zertifikat und -Schlüssel müssen gemeinsam gesetzt werden. Leere TLS-Pfade in der Umgebung löschen Dateiwerte. `GOOGLE_MAPS_TIMEOUT` begrenzt jede Google-HTTP-Anfrage einschließlich Semaphore-Wartezeit, Wiederholungen, Backoff und Antwortlesen; die OAuth-Beschaffung hat eine eigene Grenze mit demselben Wert. Die Gesamtfrist des Werkzeugs umfasst beides.

Jeder Aufruf erhält ein neues Anfragebudget, das seine parallel ausgeführten Zweige gemeinsam verwenden. Pro Prozess laufen höchstens fünf Google-API-Anfragen gleichzeitig. Nach Budgeterschöpfung scheitern weitere Versuche lokal. Zusammengesetzte Werkzeuge können erfolgreich zurückkehren und dennoch Budgetfehler pro Element enthalten: Prüfen Sie diese Felder. Das Budget begrenzt Anfragen, nicht Geldbeträge. Google-Anfragen und Felder haben unterschiedliche Preise; eine Matrixanfrage kann mehrere abrechenbare Elemente enthalten.

Lesende Google-Anfragen wiederholen HTTP 429, 500, 502, 503 und 504 sowie Netzwerkfehler. Wiederholungen berücksichtigen `Retry-After` als Sekundenwert oder HTTP-Datum; andernfalls verwenden sie exponentiellen Backoff mit Zufallsstreuung. Sie bleiben innerhalb von Anfragefrist und Aufrufbudget. Andere HTTP-Fehler einschließlich 400/401/403 werden nicht wiederholt. Authentifizierungsfehler wechseln nie stillschweigend zum API-Schlüsselmodus. Null Wiederholungen bedeutet genau einen ersten Versuch. Legacy-JSON-Fehlerstatus innerhalb von Google-Antworten werden ohne Wiederholung gemeldet.

Der HTTP-Begrenzer ist ein kontinuierlich nachgefüllter Token-Bucket mit einer Burst-Kapazität entsprechend dem Minutenlimit. Er umfasst Initialisierung, POST, GET und DELETE auf `/mcp` über alle Sitzungen. Nicht autorisierte Anfragen verbrauchen keine Tokens. Abweisungen liefern HTTP 429 mit `Retry-After`; Health und Metrikabrufe sind ausgenommen. Das Limit gilt global pro Prozess, nicht pro Benutzer oder IP; weitergeleiteten IP-Headern wird nicht vertraut. Nutzen Sie bei gemeinsam verwendeten Deployments Gateway-Authentifizierung und Limits pro Client.

## Metriken und Diagnoseereignisse

```sh
mcp-gmap-rs --log-format json serve http --metrics --rate-limit 120
```

`GET /metrics` verwendet dasselbe Bearer-Token wie `/mcp`, sofern konfiguriert. Auf einem nicht authentifizierten Loopback-Listener ist auch der Metrikzugriff nicht authentifiziert. Er ist standardmäßig deaktiviert; natives mTLS gilt bei Aktivierung ebenfalls. Erlauben Sie ausschließlich vertrauenswürdigen Scrapern Zugriff. Der Prometheus-Textendpunkt exportiert:

- `mcp_gmap_tool_calls_total{tool,outcome}` für abgeschlossene Werkzeugaufrufe;
- `mcp_gmap_tool_duration_seconds{tool,outcome}` als Histogramm;
- `mcp_gmap_google_requests_total{api,status}` für Google-HTTP-Versuche einschließlich Wiederholungen;
- `mcp_gmap_http_rate_limited_total` für abgewiesene HTTP-Anfragen.

Labels enthalten eingebaute Werkzeugnamen, API-Hostnamen, HTTP-Statuscodes und festgelegte Ergebnisse. Sie enthalten niemals Adressen, Koordinaten, Zugangsdaten, Tokens oder Sitzungs-IDs. Ein zusammengesetzter Aufruf mit Elementfehlern kann als erfolgreicher Gesamtaufruf zählen; prüfen Sie Ergebnis und vorgelagerte Statusmetriken. Metriken werden beim Neustart zurückgesetzt.

Setzen Sie `RUST_LOG=info`, um Abschlussereignisse mit Werkzeugname, Erfolg und Dauer zu sehen. Argumente und Google-Antwortinhalte werden nicht protokolliert. Die Listener-Startadresse wird getrennt auf stderr ausgegeben. Stdout bleibt CLI-JSON oder MCP-Verkehr vorbehalten.

## Doctor

```sh
mcp-gmap-rs doctor
# Explicit opt-in: makes potentially billable requests.
mcp-gmap-rs doctor --live
# Check only the API dependencies of selected tools:
mcp-gmap-rs --enabled-tools maps_pollen,maps_area_insights doctor --live
```

Ohne `--live` prüft doctor die lokale Zugangsdatenkonfiguration, HTTP-Einstellungen und zusammenpassende TLS-Dateien/Schlüssel ohne Netzwerkanfragen. Er beweist weder Schlüsselgültigkeit, IAM, API-Aktivierung, Zertifikatsvertrauen, Hostnamen, Ablaufdatum noch ADC-Zugriff. ADC-Erkennung wird bis zu einem Live-Aufruf als `not_tested` gemeldet. Doctor gibt niemals Zugangsdatenwerte oder deren JSON aus. Der JSON-Bericht geht an stdout; eine fehlgeschlagene Prüfung ergibt Exitstatus 1. Werten Sie `checks[].status` aus: `pass`, `fail` oder `not_tested`.

Der Live-Modus führt für jede von aktivierten Werkzeugen verwendete Google-API einen repräsentativen Werkzeugaufruf mit normalen Wiederholungs-, Frist- und Budgeteinstellungen aus. Teure zusammengesetzte Abläufe werden vermieden. Routes verwendet exakte Koordinaten; Roads eine Nearest-Road-Abfrage. Die Proben verwenden öffentliche Pariser Sehenswürdigkeiten statt Benutzeradressen. Ein Erfolg bedeutet, dass Google diese Probe akzeptiert hat, nicht dass jede Operation oder Region verfügbar ist.

## Herunterfahren und Konfigurationsänderungen

Ctrl-C und SIGTERM beenden den HTTP-Server und seine aktiven MCP-Sitzungen. Das Kubernetes-Beispiel gewährt dafür 30 Sekunden. Andere Anwendungseinstellungen, einschließlich per Umgebung injizierter Secret-Änderungen, benötigen einen Neustart; nur native TLS-Zertifikat-, Schlüssel- und Client-CA-Dateien unterstützen SIGHUP. Sitzungen und Metriken liegen im Arbeitsspeicher und gehen beim Neustart verloren. Verwenden Sie ein Replikat oder Sitzungsaffinität; Clients müssen sich neu verbinden, wenn der zuständige Prozess endet.

Siehe [Bereitstellung](./deployment) für Containerrechte, Probes und Zugangsdaten-Mounts sowie die [Konfigurationsreferenz](../reference/configuration) für alle Einstellungen und Optionen.
