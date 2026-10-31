# Clés, comptes de service et OAuth

Deux connexions ont des identifiants distincts : `GOOGLE_MAPS_API_KEY` ou OAuth authentifie les appels **sortants vers Google** ; `MCP_HTTP_TOKEN` authentifie les clients **entrants sur le serveur MCP HTTP**. Un jeton de compte de service Google ne remplace pas le jeton MCP.

## Choisir le mode

`api-key` est le mode par défaut. Le mode `oauth` couvre Places (New), Routes, Weather, Air Quality, Address Validation et Places Aggregate. Geocoding v3, Elevation, Time Zone, Static Maps, Pollen et Roads utilisent toujours une clé dans cette implémentation. Une résolution d’adresse dans un composite peut donc nécessiter les deux types d’identifiants ; des coordonnées exactes évitent cette résolution. Un échec OAuth ne déclenche jamais de repli silencieux vers une clé.

## Compte de service avec fichier JSON

Créez un compte dédié dans **IAM et administration → Comptes de service**. Accordez-lui `roles/serviceusage.serviceUsageConsumer` sur le projet de facturation/quota Maps et activez les API dans ce projet. Ce rôle inclut `serviceusage.services.use`, mais n’autorise pas à activer les services.

Si votre organisation autorise les clés JSON, créez-en une depuis l’onglet **Clés** du compte et stockez-la hors du dépôt, avec des permissions `0600` :

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
```

Le chemin désigne un fichier lisible, jamais le contenu JSON. Redémarrez le processus après modification des identifiants. La bibliothèque Google acquiert, met en cache et renouvelle les jetons ; les requêtes incluent `X-Goog-User-Project`.

## ADC et identité sans clé privée

Laissez `GOOGLE_APPLICATION_CREDENTIALS` non défini dans l’environnement du processus **et** dans le fichier dotenv ou Secret/ConfigMap sélectionné pour utiliser les Application Default Credentials locales ou l’identité attachée à la charge de travail. `unset` seul ne retire pas une valeur du fichier : supprimez l’entrée au lieu de définir un chemin vide. Une usurpation autorisée de compte de service est possible localement :

```sh
gcloud auth application-default login --impersonate-service-account=mcp-gmap@your-project-id.iam.gserviceaccount.com
export GOOGLE_MAPS_AUTH_MODE=oauth
export GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
```

Cette option nécessite IAM Credentials API et le rôle Token Creator approprié. Sur GKE, configurez Workload Identity Federation et le lien entre Kubernetes ServiceAccount et compte IAM ; ne définissez pas de chemin JSON pour ce flux de métadonnées. Hors GKE, montez un fichier JSON ou une configuration `external_account` conforme à votre fournisseur.

`doctor` reste hors ligne ; `doctor --live` effectue des sondes facturables. Voir [Dépannage](./troubleshooting) pour les erreurs IAM, quota et restriction IP.

## Créer le compte et autoriser la consommation

Avec une session administrateur `gcloud` autorisée à effectuer ces opérations :

```sh
export PROJECT_ID='your-project-id'
export SA_EMAIL="mcp-gmap@${PROJECT_ID}.iam.gserviceaccount.com"
gcloud iam service-accounts create mcp-gmap --project="$PROJECT_ID" \
  --display-name='MCP Google Maps'
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:${SA_EMAIL}" \
  --role='roles/serviceusage.serviceUsageConsumer'
```

Pour créer et sélectionner un fichier JSON, si la politique de votre organisation le permet :

```sh
mkdir -p "$HOME/.config/mcp-gmap-rs"
gcloud iam service-accounts keys create \
  "$HOME/.config/mcp-gmap-rs/service-account.json" \
  --iam-account="$SA_EMAIL" --project="$PROJECT_ID"
chmod 600 "$HOME/.config/mcp-gmap-rs/service-account.json"
export GOOGLE_APPLICATION_CREDENTIALS="$HOME/.config/mcp-gmap-rs/service-account.json"
export GOOGLE_MAPS_AUTH_MODE=oauth
export GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
```

Les chemins relatifs sont résolus depuis le répertoire de travail. Le chargeur accepte aussi les configurations ADC JSON `external_account`, `impersonated_service_account` et `authorized_user`. Le scope demandé est `https://www.googleapis.com/auth/cloud-platform`. Aucune saisie manuelle de jeton d’accès ou configuration d’écran de consentement utilisateur n’est nécessaire pour le flux de compte de service. `GOOGLE_MAPS_TIMEOUT` borne également l’initialisation et l’acquisition du jeton.

## Préparer une usurpation locale sans clé

Avant `gcloud auth application-default login --impersonate-service-account`, activez IAM Credentials API et donnez à votre compte développeur le droit d’usurper le compte cible :

```sh
gcloud services enable iamcredentials.googleapis.com --project="$PROJECT_ID"
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
  --project="$PROJECT_ID" --member='user:you@example.com' \
  --role='roles/iam.serviceAccountTokenCreator'
unset GOOGLE_APPLICATION_CREDENTIALS
gcloud auth application-default login --impersonate-service-account="$SA_EMAIL"
```

Les identifiants ADC obtenus fournissent des jetons de compte de service à durée limitée, sans téléchargement de sa clé privée. La découverte reste paresseuse : l’aide, `tools` et `schema` n’acquièrent pas de jeton.

## Relier Kubernetes et GKE

Pour un cluster générique, montez le JSON en Secret en lecture seule, lisible par UID 10001. Ne confondez pas ce Secret d’accès Google avec le jeton HTTP entrant. Sur un cluster GKE disposant de Workload Identity Federation, voici le lien classique entre un Kubernetes ServiceAccount et le compte IAM :

```sh
kubectl create serviceaccount mcp-gmap-rs --namespace default
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
  --project="$PROJECT_ID" --role='roles/iam.workloadIdentityUser' \
  --member="serviceAccount:${PROJECT_ID}.svc.id.goog[default/mcp-gmap-rs]"
kubectl annotate serviceaccount mcp-gmap-rs --namespace default \
  "iam.gke.io/gcp-service-account=${SA_EMAIL}"
kubectl patch deployment mcp-gmap-rs --namespace default --type=merge \
  -p '{"spec":{"template":{"spec":{"serviceAccountName":"mcp-gmap-rs"}}}}'
kubectl set env deployment/mcp-gmap-rs --namespace default \
  GOOGLE_MAPS_AUTH_MODE=oauth GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
```

Ces commandes supposent un compte IAM et un cluster dans le même projet, le manifeste dans `default` et la fonctionnalité activée sur le cluster/pool. Adaptez les projets et namespaces. Ne définissez pas `GOOGLE_APPLICATION_CREDENTIALS` pour le flux via métadonnées GKE. Voir le [guide Google Workload Identity](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/workload-identity).

Pour vérifier OAuth, appelez un outil compatible comme `mcp-gmap-rs --auth-mode oauth search-places --query 'coffee in Paris'`. L’appel est facturable. Le projet OAuth provient de `GOOGLE_CLOUD_QUOTA_PROJECT`, tandis que les appels par clé sont imputés au projet de cette clé ; ces projets peuvent différer.


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
