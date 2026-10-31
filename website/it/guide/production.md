# Gestire il servizio in produzione

Configura separatamente identità Google, autenticazione MCP e TLS. Tutti i client di un processo condividono le stesse credenziali Google, le sessioni in memoria e i limiti configurati.

## TLS, mTLS e rinnovo

```sh
mcp-gmap-rs --log-format json serve http --listen 0.0.0.0:3443   --tls-cert /etc/mcp-gmap/tls/tls.crt   --tls-key /etc/mcp-gmap/tls/tls.key   --allowed-host maps.example.com:3443 --metrics --rate-limit 120
```

Fornisci `MCP_HTTP_TOKEN` separatamente. I client verificano il certificato e inviano il bearer token a `https://maps.example.com:3443/mcp`. TLS 1.2/1.3 è nativo; l'emissione e il rinnovo dei certificati spettano alla tua PKI, a cert-manager o al gateway.

Aggiungi `--tls-client-ca /etc/mcp-gmap/tls/client-ca.pem` per richiedere certificati client su tutti i percorsi, inclusi `/health` e `/metrics`. mTLS non sostituisce il bearer token. Non sono presenti permessi per singolo certificato né controlli delle liste di revoca.

Su Linux/macOS sostituisci i file atomicamente, quindi invia SIGHUP. I nuovi handshake usano la configurazione aggiornata; le connessioni esistenti mantengono la precedente. Se il caricamento fallisce, la configurazione valida resta attiva. Le altre impostazioni richiedono un riavvio.

## Budget, retry e scadenze

I valori predefiniti sono 200 tentativi Google per chiamata, 2 retry aggiuntivi, 30 secondi per richiesta e 120 secondi complessivi per strumento. Il budget include i retry ed è condiviso dai rami concorrenti di un composito. Sono attive al massimo cinque richieste Google per processo. I retry coprono errori di rete e HTTP 429/500/502/503/504, rispettando `Retry-After`, backoff, budget e scadenza; gli errori 400/401/403 non vengono ritentati.

Il limite HTTP è 120 richieste autenticate `/mcp` al minuto per processo, con burst equivalente; `0` lo disabilita. Include inizializzazione, POST, GET e DELETE. Health e metriche sono escluse. È un limite globale, non per IP o utente.

## Osservabilità e arresto

Con `--metrics`, `/metrics` espone `mcp_gmap_tool_calls_total`, `mcp_gmap_tool_duration_seconds`, `mcp_gmap_google_requests_total` e `mcp_gmap_http_rate_limited_total`. Usa lo stesso bearer token di MCP quando configurato. Le etichette non contengono indirizzi, coordinate o credenziali; i contatori si azzerano al riavvio. Con `RUST_LOG=info` vengono registrati nome, esito e durata degli strumenti senza argomenti né risposte Google.

Ctrl-C e SIGTERM avviano l'arresto ordinato. Il manifest Kubernetes concede 30 secondi. Con TLS nativo usa probe HTTPS; con mTLS, kubelet non invia certificati client: un probe TCP controlla soltanto il listener. Un gateway deve preservare autorizzazione/header MCP, evitare buffering e cache SSE e gestire timeout di streaming adeguati. Il [riferimento configurazione](../reference/configuration) elenca i controlli disponibili.


## Gateway HTTPS su Kubernetes

Il [manifest Gateway](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/gateway.yaml) aggiunge Gateway e HTTPRoute al Service base. Installa CRD Gateway API e controller conforme, sostituisci `your-gateway-class` e `maps.example.com`, quindi crea il Secret TLS nello stesso namespace:

```sh
kubectl create secret tls mcp-gmap-tls \
  --cert=/secure/fullchain.pem --key=/secure/key.pem
kubectl set env deployment/mcp-gmap-rs \
  MCP_HTTP_ALLOWED_HOSTS='maps.example.com,mcp-gmap-rs:3000,localhost:3000,127.0.0.1:3000'
kubectl apply -f deploy/gateway.yaml
kubectl get gateway,httproute
```

Rendi persistente l'allowlist anche nella ConfigMap o nel manifest sorgente; aggiungi le vere origini browser quando necessarie. La route pubblica soltanto `/mcp`: health e metriche rimangono sul Service privato. Il file usa campi v1 e timeout HTTPRoute supportati da Gateway API dalla v1.2; verifica il supporto del controller.

Questo esempio termina TLS sul gateway e usa HTTP dal gateway al Service ClusterIP. Se occorre cifrare anche quel tratto, combina TLS nativo e BackendTLSPolicy supportata dal controller, oppure un service mesh. Limita l'accesso diretto a pod e Service con NetworkPolicy. Il gateway non introduce automaticamente un login OAuth MCP: l'eseguibile usa un bearer token condiviso.

## Interpretare doctor e metriche

```sh
mcp-gmap-rs doctor
mcp-gmap-rs --enabled-tools maps_pollen,maps_area_insights doctor --live
```

Senza `--live`, doctor controlla configurazione delle credenziali, impostazioni HTTP e corrispondenza tra file TLS e chiave senza rete. Non verifica validità della chiave Google, IAM, API abilitate, fiducia/hostname/scadenza del certificato o accesso ADC. Le sonde live effettuano una chiamata rappresentativa per ogni API distinta necessaria agli strumenti abilitati, usando normali limiti e retry. Usano luoghi pubblici parigini, evitando compositi costosi. Un successo prova l'accettazione di quella sonda, non tutte le operazioni o regioni.

Le metriche Prometheus hanno etichette fisse `tool`, `outcome`, `api` e `status` secondo la famiglia. Un composito con errori per elemento può risultare riuscito a livello principale: confronta il suo risultato con le metriche delle risposte Google. Proteggi gli scraper con bearer token e, quando configurato, certificato client. Su un listener loopback senza token, anche `/metrics` è senza autenticazione. Non pubblicarlo indiscriminatamente.
