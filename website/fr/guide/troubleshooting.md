# Dépannage

Commencez par la configuration effective, puis distinguez l’accès au serveur MCP de l’autorisation des appels Google. Une réponse de santé positive ne valide ni IAM, ni facturation, ni restrictions de clé.

## Diagnostic hors ligne

```sh
mcp-gmap-rs doctor
```

Le rapport JSON est écrit sur stdout même en cas d’échec ; le statut de sortie vaut alors 1. Inspectez `checks[].status` : `pass`, `fail` ou `not_tested`. Sans réseau, ce contrôle vérifie les réglages et la cohérence locale des fichiers TLS, mais pas leur confiance, nom DNS ou expiration, ni la validité des identifiants Google. La découverte ADC reste `not_tested` jusqu’à un appel réel.

Pour sonder seulement les dépendances utiles :

```sh
mcp-gmap-rs --enabled-tools maps_search_places,maps_weather doctor --live
```

`--live` autorise explicitement des requêtes facturables ; les sondes utilisent des lieux publics parisiens et les réglages habituels de délai/budget. Un succès ne garantit pas toute opération ou couverture géographique.

| Symptôme | Vérification |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | Mode `api-key` ou endpoint nécessitant une clé, parfois dans un composite. |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | Définir le projet consommateur Maps pour OAuth. |
| `403` Google / API désactivée | Activer le service exact dans le projet de la clé ou le projet quota OAuth, vérifier la facturation et les restrictions. |
| `serviceusage.services.use` | Accorder `roles/serviceusage.serviceUsageConsumer` au compte sur le projet quota, attendre la propagation puis réessayer. |
| Refus de clé lié à l’IP | Autoriser l’IP publique de sortie du serveur/NAT, pas l’IP privée du pod. |
| Échec OAuth | Vérifier type/chemin des identifiants, liaison d’identité, API IAM Credentials si utilisée, connectivité aux endpoints de jetons et horloge. |
| `401` MCP | Envoyer le `MCP_HTTP_TOKEN` entrant, distinct des identifiants Google. |
| Host/Origin refusé | Ajouter l’hôte transmis par le proxy et les origines exactes, ports compris. |
| `429` MCP | Réduire le débit ou adapter la limite globale ; respecter `Retry-After`. |
| Erreurs au sein d’un résultat réussi | Examiner les champs d’erreur par élément, les budgets et les métriques d’appels Google. |

En cas de rotation TLS invalide, le serveur conserve la configuration précédente et journalise l’erreur. En cas de redémarrage, les sessions en mémoire sont perdues : reconnectez les clients.
