# Konfigurationsreferenz

Diese 21 Anwendungseinstellungen gelten entsprechend für CLI, MCP stdio und Streamable HTTP. Die Priorität lautet **Standardwerte → optionale Dotenv-Datei → Umgebung → Optionen**. Siehe [Konfiguration](../guide/configuration) für Dateiauswahl und Überschreibungsverhalten.

Globale Optionen stehen vor Werkzeug oder `serve`; reine HTTP-Optionen hinter `serve http`. Ein Strich in der Optionsspalte bedeutet, dass keine entsprechende CLI-Option existiert: Stellen Sie den Wert über Umgebung oder gewählte Dotenv-Datei bereit.

## Google-Zugriff und Werkzeugausführung

| Umgebungsvariable | Standard | CLI-Option | Bedeutung und zulässige Werte |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_API_KEY` | Ungesetzt | — | Eingeschränkter Google-API-Schlüssel. Im Modus `api-key` und für ausschließlich schlüsselbasierte Endpunkte dieser Implementierung erforderlich, auch bei gewähltem OAuth-Modus. Niemals ein Werkzeugargument. |
| `GOOGLE_MAPS_AUTH_MODE` | `api-key` | `--auth-mode` | `api-key` oder `oauth`. OAuth unterstützt Places (New), Routes, Weather, Air Quality, Address Validation und Places Aggregate. |
| `GOOGLE_APPLICATION_CREDENTIALS` | ADC-Erkennung | — | Pfad einer lesbaren Dienstkonto- oder unterstützten ADC-JSON-Datei. Relative Pfade verwenden das Arbeitsverzeichnis. Für lokale ADC- oder Metadatenerkennung ungesetzt lassen. |
| `GOOGLE_CLOUD_QUOTA_PROJECT` | Ungesetzt | — | Erforderliche ID oder Nummer des Verbraucherprojekts für OAuth-Abrechnung/Kontingente. Das Dienstkonto benötigt dort `serviceusage.services.use`. |
| `GOOGLE_MAPS_LANGUAGE` | `en` | `--language` | Sprachcode für sprachabhängige Google-Antworten. Unabhängig von der Sprachauswahl dieses Handbuchs. |
| `GOOGLE_MAPS_TIMEOUT` | `30` | `--timeout` | Zeitlimit je Anfrage in Sekunden, Ganzzahl 1–600. Umfasst Warteschlange, Wiederholungen, Backoff und Antwortlesen; OAuth-Beschaffung hat eine eigene Grenze mit demselben Wert. |
| `GOOGLE_MAPS_ENABLED_TOOLS` | Alle (`*`) | `--enabled-tools` | Kommagetrennte `maps_*`-Namen oder `*`. Unbekannte Namen verhindern den Start. Deaktivierte Werkzeuge fehlen in der Erkennung und lassen sich nicht aufrufen. |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | Google-HTTP-Versuchsbudget je Werkzeugaufruf, Ganzzahl 1–10000. Zusammengesetzte Zweige teilen es; Wiederholungen verbrauchen Versuche. |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | Zusätzliche transiente Versuche je Google-Anfrage, Ganzzahl 0–5. Null bedeutet einen ersten Versuch. Wiederholungen bleiben innerhalb von Zeitlimit und Budget. |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | Gesamtfrist je Werkzeug in Sekunden, Ganzzahl 1–3600. Umfasst Zugangsdatenbeschaffung, Warteschlange, API-Aufrufe und Wiederholungen. |

Pro Prozess können fünf Google-HTTP-Anfragen gleichzeitig ausgeführt werden. Das Versuchsbudget begrenzt Anfragen, nicht Geldbeträge; eine Matrixanfrage kann mehrere Elemente abrechnen. Manche zusammengesetzten Werkzeuge liefern Elementfehler, wenn ein Zweig Budget oder Frist überschreitet. Siehe [Produktionslimits](../guide/production#limits-retries-and-deadlines).

## HTTP-Listener und Zugriff

| Umgebungsvariable | Standard | CLI-Option | Bedeutung und zulässige Werte |
| --- | --- | --- | --- |
| `MCP_HTTP_TOKEN` | Ungesetzt | — | Gemeinsames Bearer-Token für MCP-Clients. Bei Bindeadressen außerhalb von Loopback erforderlich. Getrennt von Google-Zugangsdaten und auch mit nativem mTLS nötig. |
| `MCP_HTTP_LISTEN` | `127.0.0.1:3000` | `--listen` | Listener-IP-Adresse und Port. Das Docker-Image setzt den Umgebungswert auf `0.0.0.0:3000`. |
| `MCP_HTTP_ALLOWED_HOSTS` | Loopback-Hosts | `--allowed-host` | Kommagetrennte akzeptierte HTTP-Host-Werte. Option für mehrere Werte wiederholen; Optionen ersetzen die gesamte Datei-/Umgebungsliste. Relevanten Port einschließen. |
| `MCP_HTTP_ALLOWED_ORIGINS` | Keine | `--allowed-origin` | Kommagetrennte erlaubte Browser-Origins. Vorhandene Origin-Header werden sonst abgewiesen. Ports ausdrücklich angeben; wiederholte Optionen ersetzen die Liste. |
| `MCP_HTTP_RATE_LIMIT` | `120` | `--rate-limit` | Authentifizierte `/mcp`-Anfragen pro Minute und Prozess, Ganzzahl 0–1000000. Null deaktiviert das Limit. Burst-Kapazität entspricht dem Minutenwert; kontinuierliche Auffüllung. |
| `MCP_METRICS_ENABLED` | `false` | `--metrics [true\|false]` | Prometheus unter `/metrics` aktivieren. Alleinstehendes `--metrics` bedeutet true. Verwendet gegebenenfalls dasselbe Bearer-Token; natives mTLS gilt ebenfalls. |
| `MCP_TLS_CERT` | Ungesetzt | `--tls-cert` | PEM-Server-Endzertifikat mit Zwischenzertifikatskette. Muss gemeinsam mit einem passenden privaten Schlüssel gesetzt werden. |
| `MCP_TLS_KEY` | Ungesetzt | `--tls-key` | Pfad zum privaten PEM-Serverschlüssel. Für die Laufzeitidentität lesbar halten und nicht in Quellcode oder Images speichern. |
| `MCP_TLS_CLIENT_CA` | Ungesetzt | `--tls-client-ca` | PEM-Client-CA-Bündel für obligatorische Clientzertifikate auf allen Listener-Pfaden. Ersetzt das Bearer-Token nicht. |

Der HTTP-Token-Bucket gilt prozessweit, nicht pro Benutzer oder IP. Nicht autorisierte Anfragen verbrauchen ihn nicht; Health und Metriken sind ausgenommen. Ablehnungen liefern 429 mit `Retry-After`. Die Körpergröße ist auf 12 MiB begrenzt. Siehe [MCP-Zugriff](../guide/mcp#bind-addresses-and-access) und [TLS-Betrieb](../guide/production#native-tls-and-mutual-tls).

## Diagnoseereignisse

| Umgebungsvariable | Standard | CLI-Option | Bedeutung und zulässige Werte |
| --- | --- | --- | --- |
| `RUST_LOG` | `warn` | — | Diagnose-Protokollfilter. `info` aktiviert Werkzeugabschlussereignisse; Argumente und Google-Antwortinhalte werden nicht protokolliert. |
| `MCP_LOG_FORMAT` | `text` | `--log-format` | `text` oder `json`. Diagnosen gehen immer an stderr; stdout bleibt für CLI-JSON oder MCP-Verkehr. |

## Dateiauswahl und Ausgabesteuerung

Die Dateiauswahl erfolgt vor den obigen Anwendungsebenen. `MCP_ENV_FILE` wählt einen Dotenv-Pfad; die globale Option `--env-file PATH` hat Vorrang. Eine ausdrücklich gewählte Datei muss existieren. `--no-env-file` deaktiviert jedes Dotenv-Laden und darf nicht mit `--env-file` kombiniert werden. Ohne Auswahl wird nur die optionale `.env` des Arbeitsverzeichnisses gelesen.

Auch ein ausdrücklich leerer Umgebungswert überschreibt einen Dateiwert. Er kann ein optionales Token, einen TLS-Pfad oder eine Freigabeliste leeren, macht aber einen leeren numerischen Wert nicht gültig. Geladene Dateiwerte verändern die Prozessumgebung nicht.

`doctor --live` aktiviert möglicherweise kostenpflichtige Proben; normales `doctor` arbeitet offline. Werkzeugoption `--output PATH` speichert JSON oder bei `static-map` rohe PNG-Daten. Dies sind Befehlssteuerungen, keine zusätzlichen Umgebungseinstellungen.

Änderungen an Anwendungseinstellungen und Zugangsdaten-Dateien erfordern einen Neustart. Unter Linux/macOS lädt SIGHUP native TLS-Zertifikat-, Schlüssel- und Client-CA-Dateien gemeinsam neu; bei Fehlern bleibt die vorherige TLS-Konfiguration aktiv. Siehe [Konfiguration](../guide/configuration) und [Produktionsbetrieb](../guide/production).
