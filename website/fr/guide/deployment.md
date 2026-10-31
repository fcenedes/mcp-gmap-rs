# Docker et Kubernetes

Les identifiants sont fournis à l’exécution. Le Dockerfile multiétape compile avec Rust 1.94.1 et le lockfile ; l’image finale contient le binaire, les certificats CA Debian et les licences. Elle utilise UID/GID `10001` et accepte une racine en lecture seule.

## Construire et lancer

Depuis un checkout :

```sh
docker build -t mcp-gmap-rs:local .
docker run --rm --read-only --cap-drop ALL --security-opt no-new-privileges   --env-file .env -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

L’image démarre `serve http` sur `0.0.0.0:3000` ; `MCP_HTTP_TOKEN` est donc obligatoire. Docker `--env-file` injecte l’environnement. Pour utiliser le chargeur de fichier de l’application, montez un fichier en lecture seule et passez `--env-file /etc/mcp-gmap/runtime.env serve http` après le nom de l’image. Les variables d’environnement conservent leur priorité sur ce fichier.

```sh
docker run --rm mcp-gmap-rs:local tools
docker run --rm -i --env-file .env mcp-gmap-rs:local serve stdio
```

Aucun registre de conteneurs n’est publié automatiquement. Remplacez l’image d’exemple par votre image ; le Dockerfile peut aussi être construit nativement pour Linux amd64 et arm64.

## Déployer dans Kubernetes

Téléchargez ou copiez le [manifeste de base](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml), adaptez l’image et créez le Secret attendu :

```sh
kubectl create secret generic mcp-gmap-rs --from-env-file=/secure/credentials.env
kubectl apply -f deploy/kubernetes.yaml
kubectl rollout status deployment/mcp-gmap-rs
kubectl port-forward service/mcp-gmap-rs 3000:3000
```

Le ConfigMap fournit les réglages publics ; le Secret contient les identifiants. Le manifeste utilise `--no-env-file` et `envFrom`. Pour OAuth JSON, montez le fichier en lecture seule avec des permissions lisibles par UID 10001 et définissez les trois variables OAuth. Sur GKE, privilégiez une identité de charge de travail configurée selon le guide [Authentification](./authentication).

Gardez un réplica sans mécanisme d’affinité de session. Un déploiement roulant impose une reconnexion des clients. Les sondes `/health` ne prouvent pas l’accès Google. Prévoyez DNS/HTTPS sortants et une IP de sortie autorisée par la clé. Redémarrez après modification des réglages injectés ; les fichiers TLS peuvent être rechargés par signal. Le [gateway optionnel](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/gateway.yaml) est détaillé dans [Production](./production).

## Montage OAuth et configuration explicite

Le chemin d’identifiants doit correspondre au chemin **dans** le conteneur :

```sh
docker run --rm --read-only --env-file /secure/oauth.env \
  --mount type=bind,src=/secure/service-account.json,dst=/credentials/service-account.json,readonly \
  -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

Le fichier d’environnement contient le mode `oauth`, `GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json`, le projet quota et `MCP_HTTP_TOKEN`. La clé privée ne doit pas être ajoutée aux couches de l’image. Si vous utilisez le chargeur dotenv applicatif, l’exemple complet est :

```sh
docker run --rm --read-only \
  --mount type=bind,src="$(pwd)/.env",dst=/etc/mcp-gmap/runtime.env,readonly \
  -e GOOGLE_MAPS_LANGUAGE=fr -p 127.0.0.1:3000:3000 \
  mcp-gmap-rs:local --env-file /etc/mcp-gmap/runtime.env serve http
```

L’environnement de l’image définit déjà `MCP_HTTP_LISTEN=0.0.0.0:3000` : cette valeur remplace donc celle du fichier monté. Utilisez `-e MCP_HTTP_LISTEN=...` ou `--listen` pour la modifier. Un fichier de sortie CLI nécessite un volume accessible en écriture par UID 10001.

Pour publier vos propres images multiarchitecture avec un builder Buildx configuré :

```sh
docker buildx build --platform linux/amd64,linux/arm64 \
  -t registry.example.com/your-team/mcp-gmap-rs:YOUR_VERSION --push .
```

Dans Kubernetes, le manifeste vise `default`. Adaptez les noms DNS Service lors d’un changement de namespace, l’image et les limites de ressources. Si vous remplacez `envFrom` par un dotenv monté, retirez `--no-env-file`, puis utilisez les arguments `--env-file /etc/mcp-gmap/runtime.env serve http`. Les valeurs d’environnement gardent leur priorité. Les montages projetés doivent permettre la lecture par le groupe approprié, par exemple via `fsGroup`.
