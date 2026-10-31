# Rezepte

Kombinieren Sie kleine, ausdrücklich gewählte Aufrufe rund um einen Ort oder eine Auswahlliste zurückgegebener Orts-IDs. Diese Beispiele stellen möglicherweise kostenpflichtige Google-Anfragen. Prüfen Sie zuerst die [Authentifizierung](./authentication); zusammengesetzte Werkzeuge können mehrere APIs nutzen und Teilfehler zurückgeben.

## Café finden und Anfahrt vergleichen

Suchen Sie innerhalb eines tatsächlichen Radius, laden Sie Details nur für ausgewählte IDs und vergleichen Sie anschließend die Wege. Nearby verwendet `keyword` als Google-Ortstyp, nicht als Freitext.

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now --min-rating 4
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --max-photos 0
mcp-gmap-rs distance-matrix --origins '["48.8584,2.2945"]' \
  --destinations '["48.8606,2.3376","48.8530,2.3499"]' --mode walking
```

Verwenden Sie [`search-places`](../reference/tools#maps_search_places) für natürlichsprachliche Suchabsichten, [`compare-places`](../reference/tools#maps_compare_places) für detaillierte Treffer mit optionaler Fahrentfernung oder [`explore-area`](../reference/tools#maps_explore_area) für Kategorien. Eine Suchergebnisseite ist kein vollständiges Verzeichnis. Erhalten Sie Google-Quelllinks, Bewertungsautoren und KI-Hinweise.

## Stopps planen und einen Umweg finden

```sh
mcp-gmap-rs plan-route \
  --stops '["48.8584,2.2945","48.8606,2.3376","48.8530,2.3499"]' \
  --mode walking --optimize false
mcp-gmap-rs search-along-route --text-query cafes \
  --origin '48.8584,2.2945' --destination '48.8606,2.3376' \
  --mode walking --max-results 3
```

[`directions`](../reference/tools#maps_directions) berechnet eine Fahrt mit Zeit- und Vermeidungsoptionen; [`plan-route`](../reference/tools#maps_plan_route) hält bei der Optimierung von Zwischenstopps die Endpunkte fest. Transit erlaubt nur zwei Gesamtstopps. Die Suche entlang einer Route folgt einer Polylinie, statt einen Kreis um den Mittelpunkt zu verwenden. Ihr Standardmodus ist Gehen; wählen Sie den gewünschten Verkehrsmodus daher ausdrücklich.

## Einmal auflösen, örtliche Bedingungen prüfen

```sh
mcp-gmap-rs geocode --address 'Eiffel Tower, Paris, France'
mcp-gmap-rs weather --latitude 48.8584 --longitude 2.2945 \
  --type forecast_daily --forecast-days 3
mcp-gmap-rs air-quality --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs pollen --latitude 48.8584 --longitude 2.2945 --days 3
mcp-gmap-rs timezone --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs elevation --locations '[{"latitude":48.8584,"longitude":2.2945}]'
```

Verwenden Sie Koordinaten erneut, statt Namen mehrfach aufzulösen. Trennen Sie Beobachtungen, Vorhersagen und Anbieterempfehlungen; die Abdeckung kann variieren. [`reverse-geocode`](../reference/tools#maps_reverse_geocode) übersetzt Koordinaten zurück in eine Adresse; [`batch-geocode`](../reference/tools#maps_batch_geocode) löst bis zu 50 Adressen mit Fehlern je Adresse auf. Der optionale Time-Zone-Zeitstempel verwendet Unix-Millisekunden; Höhen werden in Metern angegeben.

## Postadresse validieren

```sh
mcp-gmap-rs validate-address --address-lines '["5 Avenue Anatole France"]' \
  --locality Paris --postal-code 75007 --region-code FR
```

Prüfen Sie Urteil, Korrekturen und nicht aufgelöste Bestandteile. Adressvalidierung ist von Geocoding verschieden und garantiert keine Zustellung. USPS CASS ist optional und verlangt in dieser Implementierung `regionCode` US oder PR.

## Orte zählen statt Suchtreffer zählen

```sh
mcp-gmap-rs area-insights \
  --area '{"circle":{"latLng":{"latitude":48.8584,"longitude":2.2945},"radius":500}}' \
  --included-types '["restaurant"]' --min-rating 4
```

Aggregierte Zählwerte verwenden gewöhnlich Googles int64-Zeichenkettendarstellung. Optionale Orts-IDs sind nur bei höchstens 100 Treffern verfügbar; Details werden nicht automatisch abgerufen. Für ausdrücklich gewünschte Sichtbarkeitsanalysen misst [`local-rank-tracker`](../reference/tools#maps_local_rank_tracker) Schlüsselwortränge auf einem Raster. Beginnen Sie mit einem Schlüsselwort und 3×3 Punkten. Suchränge sind keine Benutzerbewertungen; fehlgeschlagene Punkte müssen sichtbar bleiben.

## GPS-Daten zuordnen und Karte speichern

Verwenden Sie [`snap-to-roads`](../reference/tools#maps_snap_to_roads) für eine geordnete Spur und [`nearest-roads`](../reference/tools#maps_nearest_roads) für unabhängige Punkte. Erhalten Sie `originalIndex`; Interpolation ergänzt Punkte, die keinen solchen Index haben müssen. Geschwindigkeitslimits werden nicht zurückgegeben.

```sh
mcp-gmap-rs nearest-roads --points '[{"latitude":48.8584,"longitude":2.2945}]'
mcp-gmap-rs static-map --center '48.8584,2.2945' --zoom 14 --output area.png
```

Die [Werkzeugreferenz](../reference/tools) nennt für jede Operation genaue Grenzen, Pflichtfelder, Standardwerte und Ergebnisformate.
