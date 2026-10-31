# Erste Schritte

Dieses Handbuch beschreibt die 42 Werkzeuge des aktuellen Quellstands. Das veröffentlichte Paket v0.3.1 enthält 23 Werkzeuge. Bauen Sie diesen Checkout, um die Erweiterungen zu nutzen; die Installation von v0.3.1 enthält sie nicht.

Im lokalen Repository:

```sh
cargo install --path . --locked
```

## Installation

Vorgefertigte Binärdateien für Linux x86_64, macOS Apple Silicon und macOS Intel finden Sie unter [GitHub Releases](https://github.com/fcenedes/mcp-gmap-rs/releases). Laden Sie das passende Archiv herunter, prüfen Sie es anhand von `SHA256SUMS`, entpacken Sie es und legen Sie `mcp-gmap-rs` in Ihrem `PATH` ab. Die Linux-Binärdateien benötigen glibc 2.35 oder neuer. Die macOS-Binärdateien sind weder signiert noch notarisiert.

Mit Rust 1.88 oder neuer installieren Sie das Programm über crates.io:

```sh
cargo install mcp-gmap-rs --version 0.3.1 --locked
mcp-gmap-rs --version
```

Nach dem Kompilieren wird keine Rust-, Python- oder JavaScript-Laufzeit benötigt. Der Host benötigt CA-Zertifikate, DNS und ausgehenden HTTPS-Zugriff auf Google.

## Zugriff konfigurieren

[Erstellen Sie einen eingeschränkten Schlüssel und aktivieren Sie die Dienste](./google-cloud), oder richten Sie [OAuth mit einem Dienstkonto](./authentication) ein. Speichern Sie Zugangsdaten in einer von Git ignorierten `.env` im Arbeitsverzeichnis:

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

Umgebungsvariablen überschreiben die Datei. Halten Sie Zugangsdaten aus der Versionsverwaltung und aus Image-Schichten heraus. Das Arbeitsverzeichnis ist entscheidend: Übergeordnete Verzeichnisse werden nicht nach `.env` durchsucht.

## Erkunden, prüfen, aufrufen

Werkzeugerkennung und Offline-Diagnose stellen keine Google-Anfragen:

```sh
mcp-gmap-rs tools
mcp-gmap-rs schema search-places
mcp-gmap-rs doctor
```

Führen Sie anschließend Ihren ersten möglicherweise kostenpflichtigen Aufruf aus:

```sh
mcp-gmap-rs search-places --query 'coffee near Gare de Lyon, Paris'
```

Das Ergebnis ist JSON. Bewahren Sie zurückgegebene Orts-IDs für spätere Detailabfragen auf, statt dieselben Namen wiederholt aufzulösen. Lesen Sie [CLI-Nutzung](./cli) für Automatisierung oder [MCP-Integration](./mcp), um einen Agenten anzubinden. [Rezepte](./recipes) zeigen kombinierte Abläufe; die [vollständige Referenz](../reference/tools) beschreibt jeden Parameter, Standardwert und Ergebnistyp.
