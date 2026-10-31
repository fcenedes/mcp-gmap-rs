# Exploiter en production

L’authentification MCP, le chiffrement TLS et les identifiants Google remplissent des fonctions distinctes. Tous les clients d’un même processus partagent les réglages et l’identité Google.

## TLS et mTLS natifs

```sh
mcp-gmap-rs --log-format json serve http --listen 0.0.0.0:3443   --tls-cert /etc/mcp-gmap/tls/tls.crt --tls-key /etc/mcp-gmap/tls/tls.key   --allowed-host maps.example.com:3443 --metrics --rate-limit 120
```

Fournissez `MCP_HTTP_TOKEN` séparément. La chaîne PEM commence par le certificat serveur ; sa clé privée doit correspondre. Le serveur prend en charge TLS 1.2/1.3 avec rustls, sans émettre de certificats ni fournir ACME. Ajoutez `--tls-client-ca /etc/mcp-gmap/tls/client-ca.pem` pour exiger un certificat client ; mTLS concerne aussi `/health` et `/metrics` et ne remplace pas le bearer token. Il n’existe pas de permissions par certificat ni de contrôle de liste de révocation.

Remplacez atomiquement les fichiers puis envoyez `SIGHUP` sur Linux/macOS. Les nouveaux handshakes utilisent la nouvelle configuration ; un chargement invalide conserve l’ancienne. Les connexions existantes gardent leur état. Les autres réglages nécessitent un redémarrage. Avec mTLS, utilisez une sonde Kubernetes TCP, car les sondes HTTP du kubelet ne présentent pas de certificat client.

## Budgets et observation

Par défaut : 200 tentatives Google par appel d’outil, 2 nouvelles tentatives par requête, délai global 120 s, délai de requête 30 s et 5 requêtes Google simultanées par processus. Les nouvelles tentatives couvrent les erreurs réseau et HTTP 429/500/502/503/504, respectent `Retry-After` et consomment le budget. Ce budget limite les tentatives, pas un montant monétaire.

Le limiteur MCP est global au processus : 120 requêtes authentifiées/minute avec rafale équivalente ; `0` le désactive. Les refus donnent HTTP 429 et `Retry-After`. Santé et métriques en sont exemptées. Un gateway doit fournir les limites par client si nécessaire.

`--metrics` active `/metrics`, protégé par le même jeton lorsqu’il est configuré. Il expose `mcp_gmap_tool_calls_total`, `mcp_gmap_tool_duration_seconds`, `mcp_gmap_google_requests_total` et `mcp_gmap_http_rate_limited_total`. Les métriques sont en mémoire et repartent à zéro au redémarrage. `RUST_LOG=info` active les événements de fin d’outil, sans arguments ni corps Google.

## Gateway et arrêt

Le [Gateway/HTTPRoute fourni](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/gateway.yaml) nécessite les CRD Gateway API, un contrôleur conforme, un certificat et l’adaptation du domaine/Host. Il termine TLS puis utilise HTTP vers le pod. Préservez les en-têtes MCP/Authorization, désactivez le buffering et adaptez les délais de streaming. Ctrl-C ou SIGTERM déclenche l’arrêt gracieux des sessions ; le manifeste accorde 30 s.

## Configurer le gateway Kubernetes

Installez les CRD Gateway API et un contrôleur compatible avant d’appliquer le manifeste. Remplacez `your-gateway-class` et `maps.example.com`, puis créez le Secret TLS dans le namespace du Gateway :

```sh
kubectl create secret tls mcp-gmap-tls --cert=/secure/fullchain.pem --key=/secure/key.pem
kubectl set env deployment/mcp-gmap-rs \
  MCP_HTTP_ALLOWED_HOSTS='maps.example.com,mcp-gmap-rs:3000,localhost:3000,127.0.0.1:3000'
kubectl apply -f deploy/gateway.yaml
kubectl get gateway,httproute
```

Conservez aussi ces réglages dans vos sources de déploiement/ConfigMap. La route expose seulement `/mcp` ; santé et métriques restent privées. Elle désactive les délais de requête et de requête backend ; adaptez les limites de connexion et d’inactivité à votre contrôleur. Vérifiez la prise en charge des timeouts HTTPRoute, disponibles depuis Gateway API v1.2. Si le segment gateway-pod doit aussi être chiffré, ajoutez TLS natif et une BackendTLSPolicy prise en charge, ou un mesh. Restreignez l’accès direct au Service/pod par les politiques réseau du cluster.

## Lire les métriques et les erreurs

| Série Prometheus | Étiquettes | Interprétation |
| --- | --- | --- |
| `mcp_gmap_tool_calls_total` | `tool,outcome` | Appels d’outil terminés. |
| `mcp_gmap_tool_duration_seconds` | `tool,outcome` | Histogramme de durée des outils. |
| `mcp_gmap_google_requests_total` | `api,status` | Tentatives HTTP Google, nouvelles tentatives comprises. |
| `mcp_gmap_http_rate_limited_total` | — | Requêtes refusées par le limiteur HTTP. |

Les étiquettes n’incluent ni adresses, coordonnées, identifiants, jetons ou sessions. Un composite contenant des erreurs partielles peut compter comme un succès global : inspectez la réponse et les statuts Google. Sur une écoute loopback sans jeton, `/metrics` est également sans authentification ; n’exposez cet endpoint qu’à vos collecteurs de confiance. mTLS natif reste applicable si activé.

Les délais s’emboîtent : la limite de requête comprend file d’attente, nouvelles tentatives, backoff et lecture de réponse ; l’authentification OAuth a sa propre borne utilisant la même valeur ; le délai global englobe l’ensemble. Les HTTP 400/401/403 ne sont pas réessayés. Sans `Retry-After`, les reprises utilisent un délai exponentiel avec dispersion. Le budget est partagé par toutes les branches concurrentes d’une invocation, pas entre invocations. Le limiteur MCP couvre initialisation, POST, GET et DELETE sur `/mcp` ; les requêtes non autorisées ne consomment pas son quota, et les en-têtes d’IP transférée ne déterminent pas une limite par utilisateur.
