# Configuration par couches

La priorité est **valeurs par défaut → fichier `.env` facultatif → environnement → options CLI**. Cette règle s’applique de la même manière à la CLI et aux deux transports MCP.

## Charger un fichier

Par défaut, le programme lit seulement `.env` dans le répertoire de travail. Il ne remonte pas dans les répertoires parents. L’absence de ce fichier par défaut est acceptée ; un fichier explicitement sélectionné doit exister. Un fichier illisible ou mal formé interrompt le démarrage sans afficher son contenu.

```sh
mcp-gmap-rs --env-file /etc/mcp-gmap/runtime.env serve http
mcp-gmap-rs --no-env-file serve stdio
```

`MCP_ENV_FILE` sélectionne également un fichier ; `--env-file` a priorité. `--no-env-file` désactive toute lecture de fichier. Le parseur utilise directement les valeurs sans modifier l’environnement du processus.

## Surcharger une valeur

Dans le fichier :

```dotenv
GOOGLE_MAPS_LANGUAGE=en
GOOGLE_MAPS_TIMEOUT=30
MCP_LOG_FORMAT=text
```

Dans le shell :

```sh
export GOOGLE_MAPS_LANGUAGE=fr
mcp-gmap-rs --language it --log-format json tools
```

La langue effective est ici `it`. Un environnement explicitement vide remplace aussi le fichier : il peut vider un jeton facultatif ou une liste, mais un délai vide est invalide. Répéter `--allowed-host` ou `--allowed-origin` remplace la liste entière du fichier/de l’environnement.

Pour limiter les capacités exposées, utilisez les noms MCP exacts :

```sh
mcp-gmap-rs --enabled-tools maps_search_places,maps_weather serve stdio
```

Un nom inconnu provoque un échec au démarrage. La découverte ne nécessite pas d’identifiants, mais les appels réels doivent disposer de l’authentification de leurs API. La [référence de configuration](../reference/configuration) recense les variables, options et valeurs par défaut.

Les réglages sont lus au démarrage : redémarrez les processus ou pods après modification des Secrets/ConfigMaps. Seuls les fichiers TLS disposent d’un rechargement groupé par `SIGHUP`, décrit dans [Production](./production).
