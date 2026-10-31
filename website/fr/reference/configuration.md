# Référence de configuration

Les 21 réglages ci-dessous sont communs à la CLI et au serveur, sauf les options indiquées **HTTP** qui s’utilisent après `serve http`. Une option CLI remplace l’environnement, qui remplace le fichier dotenv facultatif, qui remplace les valeurs par défaut. Les identifiants restent dans la configuration du processus et ne sont jamais des arguments d’outil MCP.

| Variable | Défaut | Option CLI | Fonction et limites |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_API_KEY` | Non définie | — | Clé Google des endpoints utilisant ce mode. |
| `GOOGLE_MAPS_AUTH_MODE` | `api-key` | `--auth-mode` | `api-key` ou `oauth` ; voir les API compatibles dans [Authentification](../guide/authentication). |
| `GOOGLE_APPLICATION_CREDENTIALS` | Découverte ADC | — | Chemin d’un fichier JSON ADC/compte de service lisible ; ne pas y placer le JSON ni une chaîne vide. |
| `GOOGLE_CLOUD_QUOTA_PROJECT` | Non défini | — | ID ou numéro du projet de facturation/quota, obligatoire pour les appels Maps OAuth. |
| `GOOGLE_MAPS_LANGUAGE` | `en` | `--language` | Langue des réponses Google lorsque l’API la prend en charge. |
| `GOOGLE_MAPS_TIMEOUT` | `30` | `--timeout` | Délai en secondes par requête Google, 1–600 ; acquisition d’identifiants également bornée avec cette valeur. |
| `GOOGLE_MAPS_ENABLED_TOOLS` | Tous | `--enabled-tools` | Noms MCP `maps_*` séparés par des virgules, ou `*` ; un nom inconnu est refusé. |
| `MCP_HTTP_TOKEN` | Non défini | — | Bearer token des clients HTTP ; obligatoire pour une écoute hors loopback. |
| `MCP_HTTP_LISTEN` | `127.0.0.1:3000` | HTTP `--listen` | Adresse d’écoute ; l’image Docker définit `0.0.0.0:3000` dans son environnement. |
| `MCP_HTTP_ALLOWED_HOSTS` | Hôtes loopback | HTTP `--allowed-host` | Liste de valeurs Host autorisées séparées par des virgules ; répéter l’option pour remplacer la liste. |
| `MCP_HTTP_ALLOWED_ORIGINS` | Aucune | HTTP `--allowed-origin` | Origines de navigateur autorisées, avec ports explicites ; tout Origin présent est refusé par défaut. |
| `RUST_LOG` | `warn` | — | Filtre des journaux sur stderr ; `info` active les événements de fin d’outil. |
| `MCP_LOG_FORMAT` | `text` | `--log-format` | Format de diagnostic `text` ou `json`. |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | Budget de tentatives HTTP Google par appel d’outil, nouvelles tentatives incluses ; 1–10000. |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | Tentatives supplémentaires par requête Google ; 0–5. |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | Délai global d’un outil en secondes, authentification, attente et nouvelles tentatives comprises ; 1–3600. |
| `MCP_HTTP_RATE_LIMIT` | `120` | HTTP `--rate-limit` | Requêtes `/mcp` authentifiées/minute par processus, rafale équivalente ; 0 désactive, maximum 1000000. |
| `MCP_METRICS_ENABLED` | `false` | HTTP `--metrics [true\|false]` | Active `/metrics` ; l’option sans valeur signifie `true`. |
| `MCP_TLS_CERT` | Non défini | HTTP `--tls-cert` | Fichier PEM du certificat serveur et de sa chaîne ; nécessite la clé correspondante. |
| `MCP_TLS_KEY` | Non défini | HTTP `--tls-key` | Fichier PEM de la clé privée serveur. |
| `MCP_TLS_CLIENT_CA` | Non défini | HTTP `--tls-client-ca` | Autorités PEM de confiance pour exiger un certificat client mTLS. |

## Sélection du fichier

`MCP_ENV_FILE` ou `--env-file PATH` sélectionne un fichier dotenv ; l’option a priorité. Sans sélection explicite, seul `.env` dans le répertoire de travail est recherché. `--no-env-file` désactive ce chargement. Le fichier par défaut absent est accepté ; un fichier explicitement sélectionné doit exister. Les variables d’environnement explicitement vides remplacent le fichier, mais doivent rester valides pour le réglage concerné.

## Options de commande

`tools` affiche le catalogue ; `schema TOOL` affiche un schéma ; `call TOOL --args JSON` transmet un objet d’arguments (`-` lit stdin). Les commandes d’outil acceptent `--output PATH` pour enregistrer le JSON, ou le PNG brut pour `static-map`. `doctor --live` active les sondes réseau potentiellement facturables. Les paramètres de premier niveau et leurs options sont décrits dans la [référence des outils](./tools).

## Rechargement

Redémarrez après modification des réglages, y compris les Secrets/ConfigMaps Kubernetes et les identifiants JSON. Sur Linux/macOS, les seuls fichiers certificat/clé/CA TLS peuvent être rechargés ensemble par `SIGHUP` ; les connexions existantes ne sont pas renégociées. Consultez [Production](../guide/production) pour les contraintes mTLS et les métriques.
