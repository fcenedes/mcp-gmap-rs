# Diagnosi e risoluzione dei problemi

Parti dalla configurazione locale, poi verifica una sola API. Una scoperta degli strumenti riuscita non dimostra che fatturazione, API e IAM siano corretti.

```sh
mcp-gmap-rs doctor
mcp-gmap-rs --enabled-tools maps_search_places doctor --live
```

La prima chiamata è offline; la seconda abilita sonde Google potenzialmente fatturabili. `doctor` stampa JSON anche in caso di errore e restituisce codice 1 se un controllo fallisce. Leggi `checks[].status`: `pass`, `fail` oppure `not_tested`. L'accesso ADC resta `not_tested` finché non viene effettuata una chiamata reale.

| Sintomo | Verifica |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | Modalità `api-key` o endpoint supportato solo con chiave; controlla anche i rami Geocoding dei compositi. |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | Imposta il progetto consumatore per le chiamate OAuth. |
| Google 403 / API disabilitata | Abilita il servizio esatto nel progetto consumatore, collega la fatturazione e controlla le restrizioni API. |
| `serviceusage.services.use` negato | Concedi `roles/serviceusage.serviceUsageConsumer` al service account sul progetto quota; attendi la propagazione e riprova. |
| Chiave rifiutata per IP | Consenti l'indirizzo pubblico di uscita del server, NAT o cluster. |
| Autenticazione OAuth fallita | Controlla tipo/percorso JSON, validità della chiave, associazioni Workload Identity, clock e accesso agli endpoint token. |
| MCP HTTP 401 | Invia `MCP_HTTP_TOKEN`, distinto dal token Google. |
| Host o Origin rifiutato | Confronta i valori effettivi inoltrati dal proxy con le liste autorizzate; includi la porta dell'Origin. |
| HTTP 429 locale | Riduci la frequenza o modifica il limite per processo; rispetta `Retry-After`. |
| Chiamata riuscita ma dati incompleti | Leggi gli errori per elemento, le celle `null` e i limiti della pagina. |

Per timeout, verifica rete/DNS/CA, metadata server o endpoint OAuth, quindi considera `--timeout`, `--tool-timeout` e `--max-requests`. Aumentarli non risolve un servizio disabilitato né IAM errato.

Per stdio, controlla percorso assoluto, directory di lavoro e stdout privo di log estranei. Per Kubernetes, una risposta `/health` positiva prova solo che il processo risponde. Dopo modifiche a Secret o ConfigMap riavvia i pod; per file TLS usa il ricaricamento SIGHUP descritto in [produzione](./production). Non condividere chiavi, token o contenuto del JSON durante una diagnosi.
