# Konfiguration

Die CLI und beide MCP-Transporte verwenden dieselbe Startkonfiguration. Nutzen Sie lokal eine Datei, in Containern die Umgebung und für ausdrückliche prozessbezogene Überschreibungen Kommandozeilenoptionen.

## Priorität

Werte werden in dieser Reihenfolge angewendet; spätere Ebenen haben Vorrang:

**Eingebaute Standardwerte → optionale Dotenv-Datei → Prozessumgebung → CLI-Optionen**

Die Standarddatei ist `.env` im aktuellen Arbeitsverzeichnis. Übergeordnete Verzeichnisse werden nicht durchsucht. Eine fehlende Standarddatei ist zulässig; eine unlesbare oder fehlerhafte Datei verhindert den Start, ohne ihren Inhalt auszugeben.

```sh
mcp-gmap-rs --env-file /etc/mcp-gmap/runtime.env serve http
mcp-gmap-rs --no-env-file tools
```

Auch `MCP_ENV_FILE` wählt eine Datei; `--env-file` hat Vorrang. Eine ausdrücklich gewählte Datei muss existieren. `--no-env-file` deaktiviert das Laden einschließlich `MCP_ENV_FILE` und darf nicht zusammen mit `--env-file` verwendet werden. Die Dotenv-Verarbeitung verändert die Prozessumgebung nicht.

## Eingebundene Konfiguration überschreiben

```dotenv
GOOGLE_MAPS_LANGUAGE=en
GOOGLE_MAPS_TIMEOUT=30
GOOGLE_MAPS_ENABLED_TOOLS=maps_search_places,maps_place_details
```

Ist diese Datei eingebunden, ändert eine Umgebungsvariable die Sprache und eine Option das Zeitlimit:

```sh
GOOGLE_MAPS_LANGUAGE=fr mcp-gmap-rs \
  --env-file /etc/mcp-gmap/runtime.env --timeout 15 tools
```

Wiederholte Optionen `--allowed-host` und `--allowed-origin` ersetzen ihre jeweiligen Datei-/Umgebungslisten vollständig. Auch ein ausdrücklich leerer Umgebungswert überschreibt die Datei: Er kann ein optionales Token oder eine Liste leeren; ein leeres Zeitlimit ist dagegen ungültig.

## Verfügbare Werkzeuge begrenzen

`GOOGLE_MAPS_ENABLED_TOOLS` beziehungsweise `--enabled-tools` akzeptiert kommagetrennte logische Namen wie `maps_weather,maps_air_quality` oder `*` für alle Werkzeuge. Unbekannte Namen verhindern den Start. Deaktivierte Werkzeuge erscheinen nicht in der Erkennung und lassen sich nicht aufrufen. Erkennung und Schemaanzeige benötigen keine Zugangsdaten; eine tatsächliche API-Operation benötigt die passende Google-Berechtigung.

Anwendungseinstellungen werden beim Start gelesen. Änderungen an Umgebungswerten, JSON-Zugangsdaten oder Kubernetes-Secrets/ConfigMaps erfordern einen Neustart. Native TLS-Zertifikat-, Schlüssel- und Client-CA-Dateien sind die Ausnahme: Sie unterstützen das [Neuladen per SIGHUP](./production#native-tls-and-mutual-tls).

Die [vollständige Konfigurationsreferenz](../reference/configuration) enthält alle 21 Einstellungen, Standardwerte, Optionen und Wertebereiche. Die Sprachauswahl dieses Handbuchs verändert `GOOGLE_MAPS_LANGUAGE` nicht; diese Variable steuert den Google-Antworttext.
