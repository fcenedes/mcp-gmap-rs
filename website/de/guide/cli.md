# Kommandozeile

Verwenden Sie die CLI für einzelne Abfragen, Shell-Automatisierung und strukturierte Unterprozessaufrufe. Jedes Werkzeug teilt Eingabeschema und Implementierung mit MCP.

## Installierte Schnittstelle erkunden

```sh
mcp-gmap-rs --version
mcp-gmap-rs --help
mcp-gmap-rs tools
mcp-gmap-rs schema search-nearby
mcp-gmap-rs search-nearby --help
```

`tools` meldet die aktivierten Werkzeuge und ihre Schemas. Prüfen Sie Ihre installierte Version, statt sämtliche Funktionen des neuesten Handbuchs vorauszusetzen. Ohne Unterbefehl startet der stdio-Server; geben Sie für einmalige Automatisierungsaufrufe deshalb immer einen Befehl an.

## Optionen oder gemeinsames JSON

Werkzeugbefehle verwenden Optionen in Kebab-Schreibweise; Objekt- und Arraywerte sind JSON. Boolesche Optionen akzeptieren `true` oder `false`; eine alleinstehende Option bedeutet `true`.

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now
```

Die allgemeine Schnittstelle `call` akzeptiert entweder einen logischen MCP-Werkzeugnamen oder einen CLI-Befehlsnamen und verwendet exakt dasselbe MCP-Argumentobjekt:

```sh
mcp-gmap-rs call maps_geocode --args '{"address":"Paris, France"}'
printf '%s' '{"address":"Paris, France"}' | mcp-gmap-rs call geocode --args -
```

Übergeben Sie bei programmatisch gestarteten Unterprozessen serialisiertes JSON als eigenes Argument. Fügen Sie nicht vertrauenswürdigen Text nicht in Shell-Befehlszeichenketten ein. Nur Parameter der obersten Ebene haben Optionen; verschachtelte Felder bleiben in JSON-Objekten.

## Ausgabe und Fehlerbehandlung

Normale Werkzeugergebnisse werden als JSON auf stdout ausgegeben. Fehler erscheinen auf stderr und führen zu einem Exitstatus ungleich null. Auch bei fehlgeschlagenen Prüfungen schreibt `doctor` seinen JSON-Bericht auf stdout. Batch-, Matrix- und zusammengesetzte Ergebnisse können Fehler pro Element enthalten, obwohl der Befehl erfolgreich endet: Prüfen Sie diese Felder, bevor Sie eine Operation als vollständig abgeschlossen betrachten.

```sh
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --output place.json
mcp-gmap-rs static-map --center Paris --zoom 12 --output paris.png
```

`--output` schreibt bei normalen Werkzeugen JSON und bei einer statischen Karte rohe PNG-Daten. Ohne diese Option enthält die CLI-Ausgabe statischer Karten Base64-Daten. Zeitangaben verwenden gewöhnlich RFC 3339 mit ausdrücklicher Zeitzone; der optionale `timestamp` des Zeitzonenwerkzeugs verwendet Unix-**Millisekunden**. Siehe [alle Werkzeuge und Beispiele](../reference/tools).
