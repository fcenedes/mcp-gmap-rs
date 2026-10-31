# Démarrage rapide

Un seul exécutable Rust donne accès à Google Maps en ligne de commande, en MCP stdio ou en MCP Streamable HTTP. Les trois interfaces partagent les mêmes 23 outils et les mêmes contrôles. Aucun environnement Node.js, Python ou module spécifique à un assistant n’est nécessaire pour exécuter le programme.

## Installer

Téléchargez l’archive correspondant à Linux x86_64, macOS Apple Silicon ou macOS Intel depuis les [versions GitHub](https://github.com/fcenedes/mcp-gmap-rs/releases). Vérifiez son empreinte avec `SHA256SUMS`, extrayez-la et placez l’exécutable dans votre `PATH`. Sur macOS, les binaires ne sont pas signés ni notariés.

Avec Rust 1.88 ou une version plus récente :

```sh
cargo install mcp-gmap-rs --version 0.3.1 --locked
mcp-gmap-rs --version
```

Le système hôte doit disposer de certificats d’autorité valides et d’un accès HTTPS sortant vers Google.

## Découvrir sans identifiants

```sh
mcp-gmap-rs tools
mcp-gmap-rs schema search-nearby
mcp-gmap-rs doctor
```

`tools`, `schema` et l’aide ne contactent pas Google. `doctor` vérifie la configuration locale ; il peut échouer si les identifiants attendus manquent, sans effectuer de requête réseau.

## Effectuer un premier appel

Créez une clé restreinte et activez les services nécessaires en suivant [Google Cloud](./google-cloud). Conservez-la dans un fichier `.env` non suivi par Git :

```dotenv
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

```sh
mcp-gmap-rs geocode --address 'Paris, France'
```

Cet appel réel peut être facturé. Le résultat JSON contient les coordonnées, l’adresse formatée et un identifiant de lieu. Choisissez ensuite votre interface : [CLI](./cli), [MCP](./mcp) ou [conteneur/Kubernetes](./deployment). Consultez la [référence des outils](../reference/tools) pour les paramètres et limites exacts.
