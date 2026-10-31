# Connecter un client MCP

Le programme utilise le SDK MCP Rust pour négocier le protocole, traiter JSON-RPC et prendre en charge les flux SSE et les sessions. Les outils partagent exactement l’implémentation de la CLI.

## Transport stdio

```sh
mcp-gmap-rs serve stdio
```

Lancer l’exécutable sans argument démarre aussi stdio. stdin/stdout transportent uniquement le protocole MCP JSON-RPC, une ligne par message ; les diagnostics restent sur stderr. Exemple de configuration générique d’un client :

```json
{
  "mcpServers": {
    "google-maps": {
      "command": "/absolute/path/to/mcp-gmap-rs",
      "args": ["--env-file", "/secure/mcp-gmap.env", "serve", "stdio"]
    }
  }
}
```

Le fichier doit être lisible par le processus lancé par le client. Cette configuration est une connexion MCP, pas un plugin spécifique à un assistant. En conteneur, gardez stdin ouvert avec `-i` sans TTY.

## Transport Streamable HTTP

```sh
mcp-gmap-rs serve http --listen 127.0.0.1:3000
```

Configurez l’URL du client sur `http://127.0.0.1:3000/mcp`. Le endpoint accepte POST, GET et DELETE selon la version négociée. `GET /health` indique seulement la disponibilité du processus, sans vérifier Google.

Pour une écoute distante, fournissez un jeton et les noms d’hôtes autorisés :

```sh
export MCP_HTTP_TOKEN='replace-with-a-long-random-secret'
mcp-gmap-rs serve http --listen 0.0.0.0:3000   --allowed-host maps.example.com   --allowed-origin https://app.example.com:443
```

Le client envoie `Authorization: Bearer <token>`. Les Origin présents sont refusés par défaut ; autorisez les origines réelles, port compris. Les noms Host doivent correspondre à la liste autorisée. Utilisez HTTPS natif ou un [gateway TLS](./production) pour les clients distants.

Les requêtes ont une limite de corps de 1 Mio. Les sessions sont en mémoire : un redémarrage les supprime et plusieurs réplicas nécessitent de l’affinité. Tous les clients d’un processus partagent l’identité Google et les budgets configurés. Ctrl-C et SIGTERM déclenchent l’arrêt gracieux.
