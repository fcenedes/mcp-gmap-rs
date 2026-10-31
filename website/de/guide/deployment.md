# Bereitstellung

Für API-Schlüsselerstellung, Dienstaktivierung und Dienstkonto-OAuth verwenden Sie die [Google-Cloud-Anleitung](./google-cloud). Dieselbe mehrschichtige Konfiguration funktioniert in Containern. OAuth kann eine eingebundene JSON-Datei oder die ADC-Identität der Workload verwenden; Zugangsdaten werden zur Laufzeit bereitgestellt.

Laden oder klonen Sie das [Quellrepository](https://github.com/fcenedes/mcp-gmap-rs) und bauen Sie im Repository-Hauptverzeichnis:

```sh
docker build -t mcp-gmap-rs:local .
```

Das mehrstufige Dockerfile kompiliert mit Rust 1.94.1 und der Lockdatei. Das Laufzeit-Image enthält Programm, Debian-CA-Zertifikate und Lizenzen, läuft mit UID/GID 10001 und unterstützt ein schreibgeschütztes Root-Dateisystem. Der zugelassene Build-Kontext schließt lokale `.env`-Dateien, Git-Daten und Build-Ausgaben aus. Für den Build werden keine Zugangsdaten benötigt. BuildKit speichert Cargo-Downloads und Build-Ausgaben für spätere Builds zwischen.

Dasselbe Dockerfile baut nativ für Linux amd64 und arm64. Mit einem konfigurierten Buildx-Builder bauen und veröffentlichen Sie beide Architekturen in Ihrer Registry so:

```sh
docker buildx build --platform linux/amd64,linux/arm64 \
  -t registry.example.com/your-team/mcp-gmap-rs:YOUR_VERSION --push .
```

Eine automatische Veröffentlichung in einer Container-Registry ist nicht konfiguriert. Ersetzen Sie Image-Namen und Versionen durch Ihr gebautes oder veröffentlichtes Image.

## HTTP, CLI oder stdio starten

Das Image startet standardmäßig `serve http` auf `0.0.0.0:3000`. Stellen Sie zur Laufzeit ein Bearer-Token bereit, etwa über dasselbe `.env`-Format wie in `.env.example`:

```sh
docker run --rm --read-only --cap-drop ALL --security-opt no-new-privileges \
  --env-file .env -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

Die `.env` muss `MCP_HTTP_TOKEN` enthalten, da der Container außerhalb von Loopback lauscht. Google-Aufrufe verwenden abhängig von den aktivierten Werkzeugen `GOOGLE_MAPS_API_KEY` oder die OAuth-Einstellungen und Zugangsdaten-Mounts aus der Einrichtungsanleitung. Clients verwenden `http://127.0.0.1:3000/mcp` und `Authorization: Bearer <token>`. `GET /health` benötigt kein Token und kontaktiert Google nicht.

Dockers `--env-file` injiziert Umgebungsvariablen. Um stattdessen den Dateilader der Anwendung zu verwenden, binden Sie eine lesbare Datei ein und wählen sie ausdrücklich:

```sh
docker run --rm --read-only \
  --mount type=bind,src="$(pwd)/.env",dst=/etc/mcp-gmap/runtime.env,readonly \
  -e GOOGLE_MAPS_LANGUAGE=fr -p 127.0.0.1:3000:3000 \
  mcp-gmap-rs:local --env-file /etc/mcp-gmap/runtime.env serve http
```

Die Umgebung überschreibt die eingebundene Datei, und CLI-Optionen überschreiben beide. Auch `MCP_HTTP_LISTEN=0.0.0.0:3000` des Images ist ein Umgebungswert; ändern Sie ihn gegebenenfalls mit `-e MCP_HTTP_LISTEN=...` oder `--listen`. Für ausschließliche Umgebungsnutzung übergeben Sie `--no-env-file serve http`.

Überschreiben Sie den Image-Befehl für CLI oder stdio:

```sh
docker run --rm mcp-gmap-rs:local tools
docker run --rm --env-file .env mcp-gmap-rs:local geocode --address Paris
docker run --rm -i --env-file .env mcp-gmap-rs:local serve stdio
```

Nutzen Sie für stdio `-i`, ohne ein TTY zuzuweisen. Für CLI-Ausgabedateien binden Sie ein für UID 10001 beschreibbares Verzeichnis ein und verwenden darin `--output`.

## Kubernetes

Das [Kubernetes-Basismanifest](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml) liefert nicht geheime Einstellungen über eine ConfigMap und Zugangsdaten über ein bestehendes Secret namens `mcp-gmap-rs`. Erstellen Sie dieses Secret aus einer lokalen, nicht eingecheckten Datei mit `GOOGLE_MAPS_API_KEY` und `MCP_HTTP_TOKEN`:

```sh
kubectl create secret generic mcp-gmap-rs --from-env-file=/secure/credentials.env
```

Ersetzen Sie `mcp-gmap-rs:local` im Deployment durch Ihr Registry-Image oder laden Sie Ihr lokales Image in den Entwicklungscluster. Wenden Sie dann das Beispiel an:

```sh
kubectl apply -f deploy/kubernetes.yaml
kubectl rollout status deployment/mcp-gmap-rs
kubectl port-forward service/mcp-gmap-rs 3000:3000
```

Das Beispiel verwendet den Namespace `default`. Passen Sie konfigurierte Service-DNS-Hostnamen bei einem anderen Namespace oder Ingress an. Der ClusterIP-Service ist clusterintern; verwenden Sie für entfernte Clients einen TLS-Ingress oder Proxy, der einen in `MCP_HTTP_ALLOWED_HOSTS` erlaubten Host sowie Autorisierungs- und MCP/SSE-Header weiterleitet. Erlauben Sie bei Bedarf ausdrücklich Browser-Origins einschließlich Ports.

Das Deployment nutzt `--no-env-file`, weil alle Einstellungen über `envFrom` injiziert werden. Alternativ binden Sie ein Secret mit Dotenv-Datei ein, entfernen diese Option und setzen `args: ["--env-file", "/etc/mcp-gmap/runtime.env", "serve", "http"]`. Die projizierte Datei muss für UID 10001 lesbar sein; Umgebungsvariablen aus Secrets oder ConfigMaps überschreiben sie weiterhin. Siehe die Kubernetes-Anleitungen zu [Umgebungsinjektion](https://kubernetes.io/docs/tasks/inject-data-application/define-environment-variable-container/) und [Secret-Volumes](https://kubernetes.io/docs/concepts/configuration/secret/).

Anwendungseinstellungen werden beim Start gelesen. Starten Sie Pods nach Änderungen dieser Secret-/ConfigMap-Werte neu; native TLS-Dateien unterstützen außerdem SIGHUP-Neuladen. Probes verwenden `/health` für HTTP-Verfügbarkeit: Ein gesunder Pod beweist weder API-Aktivierung noch Google-Berechtigung. Google-Aufrufe benötigen ausgehendes HTTPS und funktionierendes DNS. API-Schlüsseleinschränkungen müssen die öffentliche Ausgangsadresse des Clusters erlauben; OAuth benötigt Zugriff auf Token- und Identitätsendpunkte.

Behalten Sie bei der standardmäßigen MCP-Sitzungsspeicherung im Arbeitsspeicher ein Replikat bei. Mehrere Replikate benötigen Sitzungsaffinität zum zuständigen Pod; rollende Neustarts verwerfen Sitzungen, sodass sich Clients neu verbinden müssen. Die Ressourcenanforderungen und -limits im Beispiel sind an Ihre Last anzupassende Ausgangswerte.

SIGTERM löst das geordnete Herunterfahren von MCP/HTTP aus; Kubernetes gewährt im Beispiel 30 Sekunden. Das Image läuft außerdem ohne zusätzliche Capabilities und ohne Rechteausweitung.

## HTTPS, Metriken und Limits in Produktion

Die [Produktionsanleitung](./production) behandelt natives TLS/mTLS, Zertifikatsneuladen, das optionale HTTPS-Gateway `deploy/gateway.yaml`, Metriken, Anfragebudgets, HTTP-Ratenlimits, JSON-Protokolle und Offline-/Live-Prüfungen mit `doctor`. Übergeben Sie dieselben TLS-Optionen hinter dem Image-Namen oder injizieren Sie `MCP_TLS_CERT`, `MCP_TLS_KEY` und optional `MCP_TLS_CLIENT_CA` mit schreibgeschützten PEM-Mounts. Echte Schlüssel gehören niemals ins Image. Die Standard-ConfigMap setzt Anfragelimits und JSON-Protokollierung; Metriken bleiben bis zur ausdrücklichen Aktivierung ausgeschaltet. Der unverschlüsselte Standard-Service ist für den privaten Gateway-zu-Pod-Abschnitt gedacht.

## OAuth-Zugangsdaten im Container

Binden Sie eine Dienstkonto-JSON-Datei schreibgeschützt ein und setzen Sie den Pfad im Container, nicht den Hostpfad:

```sh
docker run --rm --read-only --env-file /secure/oauth.env \
  --mount type=bind,src=/secure/service-account.json,dst=/credentials/service-account.json,readonly \
  -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

Die OAuth-Umgebungsdatei setzt `GOOGLE_MAPS_AUTH_MODE=oauth`, `GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json`, `GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id` und `MCP_HTTP_TOKEN`. UID 10001 muss die eingebundene Datei lesen können. Behalten Sie einen API-Schlüssel nur für Werkzeuge, die ausschließlich schlüsselbasierte Endpunkte verwenden. Verwenden Sie auf GKE die [Workload-Identity-Anleitung](./authentication#c-kubernetes-and-gke-workload-identity), statt einen langlebigen Schlüssel einzubinden.
