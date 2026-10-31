# Fehlerbehebung

Bestimmen Sie zuerst die betroffene Verbindung: lokale Konfiguration, MCP-Verbindung zwischen Client und Server oder ausgehende Google-Anfrage. Korrigieren Sie die passende Ebene und wiederholen Sie die betroffene Operation.

## Offline beginnen

```sh
mcp-gmap-rs --version
mcp-gmap-rs tools
mcp-gmap-rs doctor
```

`doctor` gibt auch bei fehlgeschlagenen Prüfungen JSON aus. Prüfen Sie Exitstatus und `checks[].status`: `pass`, `fail` oder `not_tested`. Offline-Prüfungen beweisen weder IAM, API-Aktivierung, Abrechnung, Zertifikatsvertrauen und -ablauf noch ADC-Zugriff. `/health` belegt nur die Prozesslebendigkeit.

Blockiert eine fehlerhafte lokale Dotenv-Datei die Erkennung, überspringt `--no-env-file tools` sie. Dies kann die Werkzeugauswahl ändern und ist deshalb kein Nachweis der beabsichtigten Dateikonfiguration.

## Häufige Fehler einordnen

| Symptom | Nächste Prüfung |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | API-Schlüsselmodus gewählt oder ausschließlich schlüsselbasierter Endpunkt erreicht. |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | Verbraucherprojekt-ID oder -Nummer für OAuth setzen. |
| Google 403 / API deaktiviert | Die konkrete API im Verbraucherprojekt aktivieren; Abrechnung und Schlüsseleinschränkungen prüfen. |
| `serviceusage.services.use` verweigert | Dem Dienstkonto `roles/serviceusage.serviceUsageConsumer` im Kontingentprojekt gewähren; IAM-Übertragung abwarten und erneut versuchen. |
| IP-Ablehnung des API-Schlüssels | Öffentliche Ausgangs-IP von Server oder Cluster erlauben, nicht die private Pod-IP. |
| MCP HTTP 401 | Bearer-Token des Servers senden; ein Google-Zugriffstoken authentifiziert eine andere Verbindung. |
| Host/Origin abgewiesen | Konfigurierten Host und Browser-Origin einschließlich relevanter Ports exakt abgleichen. |
| HTTP 429 vom MCP-Server | Prozessweites Limit erschöpft; `Retry-After` beachten und Clientparallelität prüfen. |
| OAuth-Zeitüberschreitung | Verbindung zu Token-/STS-/IAM- oder Metadatenendpunkten, Systemzeit und Anfragezeitlimit prüfen. |
| TLS-Verbindung schlägt fehl | Zertifikatskette, Hostname, Vertrauen und optionales mTLS-Clientzertifikat getrennt von der HTTP-Autorisierung prüfen. |
| Exit 0, aber unvollständige Daten | Elementfehler in Matrix-/Kompositergebnissen prüfen, nicht nur den Exitstatus. |

## Live-Zugriff gezielt überprüfen

Live-Proben können kostenpflichtig sein. Begrenzen Sie sie auf tatsächlich benötigte API-Abhängigkeiten:

```sh
mcp-gmap-rs --enabled-tools maps_weather doctor --live
```

Eine bestandene Probe garantiert nicht jede Region oder Operation. Wiederholen Sie nach Google-Cloud-Änderungen denselben fehlgeschlagenen Aufruf, statt Erfolg aus Console-Einstellungen abzuleiten. Siehe [Authentifizierung](./authentication) für IAM und Zugangsdatenquellen, [Konfiguration](./configuration) für Prioritäten und [Produktionsbetrieb](./production) für Budgets, Fristen und Metriken. Geben Sie niemals Zugangsdaten-JSON oder die gesamte Umgebung in einem Fehlerbericht aus.
