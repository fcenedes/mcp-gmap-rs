# Utiliser la CLI

Chaque outil possède une commande directe et une interface JSON générique. Les paramètres sont identiques à ceux du serveur MCP ; la [référence des outils](../reference/tools) donne les champs obligatoires, limites et valeurs par défaut.

## Découvrir et appeler

```sh
mcp-gmap-rs tools
mcp-gmap-rs schema directions
mcp-gmap-rs directions --help
mcp-gmap-rs search-places --query 'coffee in Paris' --open-now --min-rating 4
```

Les noms camelCase deviennent des options séparées par des tirets : `placeId` donne `--place-id`. Les objets et tableaux prennent des valeurs JSON ; seuls les paramètres de premier niveau ont leur propre option :

```sh
mcp-gmap-rs search-nearby   --center '{"value":"48.8584,2.2945","isCoordinates":true}'   --keyword cafe --radius 500
```

Un booléen sans valeur signifie `true` ; `--open-now false` transmet `false`. Les propriétés inconnues et les valeurs hors schéma sont rejetées avant l’appel Google.

## JSON et pipelines

```sh
mcp-gmap-rs call maps_geocode --args '{"address":"Paris"}'
printf '%s' '{"address":"Paris"}' | mcp-gmap-rs call geocode --args -
mcp-gmap-rs static-map --center Paris --zoom 12 --output paris.png
```

`call` accepte le nom MCP ou la commande CLI ; `--args` reçoit l’objet JSON, et `--args -` le lit depuis stdin. Les résultats vont sur stdout et les erreurs/diagnostics sur stderr. Un échec de commande produit un statut non nul. `--output` écrit le JSON dans un fichier, ou directement les octets PNG pour `static-map`. Sans cette option, une carte est représentée par un objet JSON avec `mimeType` et `base64`.

Un composite ou une matrice peut réussir tout en contenant des erreurs par élément. Inspectez `success`, `error`, `errors`, `details_error` et les champs similaires, plutôt que le seul code de sortie. Les distances sont généralement en mètres, les durées en secondes. Les horaires de trajet utilisent RFC 3339 avec fuseau ; le `timestamp` de `timezone` est exprimé en millisecondes Unix.
