# Parcours pratiques

Ces exemples effectuent de vrais appels Google, potentiellement facturables. Commencez avec peu de résultats, vérifiez les erreurs partielles et conservez les informations d’attribution.

## Trouver un café puis comparer les trajets

```sh
mcp-gmap-rs search-places --query 'cafes near Eiffel Tower, Paris' --min-rating 4
mcp-gmap-rs compare-places --query 'cafes near Eiffel Tower, Paris'   --user-location '{"latitude":48.8584,"longitude":2.2945}' --limit 3
```

`compare-places` enrichit une courte liste et utilise la **voiture** pour les distances. Pour un trajet piéton précis, reprenez les coordonnées choisies :

```sh
mcp-gmap-rs directions --origin '48.8584,2.2945'   --destination '48.8606,2.3376' --mode walking
```

## Vérifier une adresse et les conditions locales

```sh
mcp-gmap-rs validate-address --address-lines '["5 Avenue Anatole France"]'   --region-code FR --locality Paris --postal-code 75007
mcp-gmap-rs weather --latitude 48.8584 --longitude 2.2945 --type forecast_daily --forecast-days 3
mcp-gmap-rs air-quality --latitude 48.8584 --longitude 2.2945 --include-pollutants
mcp-gmap-rs pollen --latitude 48.8584 --longitude 2.2945 --days 3
```

Le verdict d’adresse ne garantit pas la livraison. Préservez les unités météo et les recommandations du fournisseur ; leur disponibilité varie selon la zone et la saison.

## Compter plutôt qu’énumérer

```sh
mcp-gmap-rs area-insights   --area '{"circle":{"latLng":{"latitude":48.8584,"longitude":2.2945},"radius":500}}'   --included-types '["restaurant"]' --min-rating 4
```

Ce décompte agrégé diffère des 20 résultats maximum d’une page Places. `--include-places` peut ajouter des identifiants uniquement pour un total de 100 ou moins.

## Rechercher le long d’un trajet

```sh
mcp-gmap-rs search-along-route --text-query cafes   --origin '48.8584,2.2945' --destination '48.8606,2.3376' --mode walking --max-results 3
```

La recherche suit la polyligne calculée. Aucune pagination ou expansion de détails n’est automatique. Consultez les [23 outils](../reference/tools) avant d’ajouter des étapes : matrices, photos et grilles de classement multiplient les requêtes ; une grille de taille 7 avec trois mots-clés atteint 147 recherches avant nouvelles tentatives.

## Planifier des arrêts, recaler une trace et exporter une carte

```sh
mcp-gmap-rs plan-route \
  --stops '["48.8584,2.2945","48.8606,2.3376","48.8530,2.3499"]' \
  --mode walking --optimize false
mcp-gmap-rs nearest-roads --points '[{"latitude":48.8584,"longitude":2.2945}]'
mcp-gmap-rs static-map --center '48.8584,2.2945' --zoom 14 --size 320x240 --output paris.png
```

L’optimisation de l’ordre des arrêts ne s’applique qu’à partir de quatre arrêts au total ; les extrémités restent fixes. Pour une trace GPS ordonnée, utilisez `snap-to-roads` avec 2 à 100 points plutôt que `nearest-roads`, qui traite 1 à 100 points indépendants. Conservez `originalIndex` pour relier les segments aux entrées ; les points interpolés peuvent ne pas l’avoir.

Pour traiter plusieurs adresses, `batch-geocode --addresses '["Paris","London","Rome"]'` conserve l’ordre et renvoie un succès ou une erreur par adresse. Pour comparer plusieurs trajets, `distance-matrix` renvoie des cellules en mètres/secondes avec erreurs séparées ; un tableau partiellement rempli n’est pas une comparaison complète. Ces opérations ne garantissent pas un tarif monétaire fixe malgré leur budget de requêtes.
