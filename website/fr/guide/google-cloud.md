# Préparer Google Cloud

Choisissez ou créez un projet dans la [console Google Cloud](https://console.cloud.google.com/), associez-lui un compte de facturation actif, puis activez les API utilisées dans **API et services → Bibliothèque**. Autoriser un service dans les restrictions d’une clé et activer ce service sont deux opérations distinctes.

## Services utilisés

| API | Service à activer | Authentification prise en charge |
| --- | --- | --- |
| Places API (New) | `places.googleapis.com` | Clé ou OAuth |
| Routes API | `routes.googleapis.com` | Clé ou OAuth |
| Weather API | `weather.googleapis.com` | Clé ou OAuth |
| Air Quality API | `airquality.googleapis.com` | Clé ou OAuth |
| Address Validation API | `addressvalidation.googleapis.com` | Clé ou OAuth |
| Places Aggregate API | `areainsights.googleapis.com` | Clé ou OAuth |
| Pollen API | `pollen.googleapis.com` | Clé |
| Roads API | `roads.googleapis.com` | Clé |
| Geocoding API v3 | `geocoding-backend.googleapis.com` | Clé |
| Elevation API | `elevation-backend.googleapis.com` | Clé |
| Time Zone API | `timezone-backend.googleapis.com` | Clé |
| Maps Static API | `static-maps-backend.googleapis.com` | Clé |

Les outils `directions` et `distance-matrix` utilisent Routes API ; les anciennes API Directions et Distance Matrix ne sont pas nécessaires. Les composites peuvent appeler plusieurs services.

## Créer une clé

Dans [Google Maps Platform → Clés et identifiants](https://console.cloud.google.com/google/maps-apis/credentials), créez une clé API. Limitez ses **API autorisées** aux services utilisés. Pour cet exécutable serveur, appliquez une restriction d’application par **adresses IP** correspondant aux adresses publiques de sortie vues par Google. L’adresse privée d’un pod ne correspond généralement pas à cette adresse ; une sortie NAT fixe facilite cette restriction.

Stockez la clé dans un Secret, une variable d’environnement ou un `.env` ignoré par Git :

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

Testez un outil dont l’API est activée, par exemple `mcp-gmap-rs geocode --address Paris`. Les appels sont potentiellement facturables ; n’activez que les services nécessaires, surveillez les quotas et préservez les attributions Google dans les résultats affichés. Pour un compte de service ou une identité de charge de travail, passez au guide [Authentification](./authentication).

## Activer les douze services en ligne de commande

Avec une session administrateur disposant des droits d’activation :

```sh
export PROJECT_ID='your-project-id'
gcloud services enable --project="$PROJECT_ID" \
  places.googleapis.com routes.googleapis.com \
  weather.googleapis.com airquality.googleapis.com \
  geocoding-backend.googleapis.com elevation-backend.googleapis.com \
  timezone-backend.googleapis.com static-maps-backend.googleapis.com \
  addressvalidation.googleapis.com areainsights.googleapis.com \
  pollen.googleapis.com roads.googleapis.com
```

Vous pouvez ne conserver que les services requis par votre cas d’usage. La découverte du catalogue ne prouve pas qu’ils sont activés ; utilisez `doctor --live` avec une liste d’outils limitée ou un appel réel. Les endpoints Geocoding de ce projet sont en v3 : la disponibilité d’OAuth dans Geocoding v4 ne change pas leur authentification. Les champs facturés et quotas varient selon les API, et une seule matrice peut représenter plusieurs éléments facturables.


## Services supplémentaires et limites {#expanded-services}

Solar et Map Tiles acceptent aussi OAuth. Datasets et Route Optimization exigent OAuth et les autorisations IAM propres aux opérations. Street View Publish exige un fichier authorized_user explicite et le consentement au scope streetviewpublish.

| Service | API | Authentification |
| --- | --- | --- |
| Geolocation API | `geolocation.googleapis.com` | Clé API |
| Solar API | `solar.googleapis.com` | Clé API / OAuth |
| Aerial View API | `aerialview.googleapis.com` | Clé API |
| Map Tiles API | `tile.googleapis.com` | Clé API / OAuth |
| Street View Static API | `street-view-image-backend.googleapis.com` | Clé API |
| Maps Platform Datasets API | `mapsplatformdatasets.googleapis.com` | OAuth / IAM |
| Route Optimization API | `routeoptimization.googleapis.com` | OAuth / IAM |
| Street View Publish API | `streetviewpublish.googleapis.com` | OAuth utilisateur avec consentement ; compte de service refusé |

Speed Limits exige un accès Asset Tracking restreint. Aerial View est limité aux adresses américaines prises en charge et interdit le téléchargement ou la mise en cache des vidéos. Map Tiles est réservé à la visualisation : analyse automatique et extraction interdites. Les outils Weather minute/map sont expérimentaux, avec une limite de cache d’une heure et des restrictions d’usage. Les références de chaque outil détaillent les conditions, l’attribution et les limites.

[Google: Speed Limits](https://developers.google.com/maps/documentation/roads/speed-limits), [Map Tiles](https://developers.google.com/maps/documentation/tile/policies), [Aerial View](https://developers.google.com/maps/documentation/aerial-view/policies), [Street View Publish](https://developers.google.com/streetview/publish/authorizing).
