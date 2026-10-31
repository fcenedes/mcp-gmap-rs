# Riferimento della configurazione

Queste 21 impostazioni applicative valgono per CLI, MCP stdio e Streamable HTTP secondo il contesto. La precedenza è **valori predefiniti → dotenv facoltativo → ambiente → opzioni CLI**. La [guida alla configurazione](../guide/configuration) descrive selezione del file e semantica delle sovrascritture.

Le opzioni globali precedono il comando di strumento o `serve`; quelle esclusivamente HTTP seguono `serve http`. Un trattino nella colonna delle opzioni indica l'assenza di un'opzione CLI: usa ambiente o file dotenv.

## Accesso Google ed esecuzione degli strumenti

| Variabile d'ambiente | Valore predefinito | Opzione CLI | Significato e valori ammessi |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_API_KEY` | Non impostata | — | Chiave API Google limitata. Necessaria in modalità `api-key` e per gli endpoint supportati solo con chiave, anche quando è selezionato OAuth. Non è un parametro di strumento. |
| `GOOGLE_MAPS_AUTH_MODE` | `api-key` | `--auth-mode` | `api-key` oppure `oauth`. OAuth copre Places (New), Routes, Weather, Air Quality, Address Validation e Places Aggregate. |
| `GOOGLE_APPLICATION_CREDENTIALS` | Ricerca ADC | — | Percorso di un file JSON leggibile di service account o ADC supportato. I percorsi relativi partono dalla directory di lavoro. Lascia non impostata per ADC locale o ricerca nel metadata server. |
| `GOOGLE_CLOUD_QUOTA_PROJECT` | Non impostata | — | ID o numero del progetto consumatore di fatturazione/quota richiesto per OAuth. Il service account deve avere `serviceusage.services.use` in tale progetto. |
| `GOOGLE_MAPS_LANGUAGE` | `en` | `--language` | Codice lingua per le risposte Google che supportano la localizzazione. È indipendente dal selettore lingua di questo manuale. |
| `GOOGLE_MAPS_TIMEOUT` | `30` | `--timeout` | Timeout per richiesta in secondi, intero 1–600. Include attesa, retry, backoff e lettura della risposta; l'acquisizione OAuth ha un limite proprio basato sullo stesso valore. |
| `GOOGLE_MAPS_ENABLED_TOOLS` | Tutti (`*`) | `--enabled-tools` | Nomi `maps_*` separati da virgole oppure `*`. Nomi sconosciuti impediscono l'avvio. Gli strumenti disabilitati non compaiono nella scoperta e non possono essere chiamati. |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | Budget di tentativi HTTP Google per invocazione, intero 1–10000. Condiviso tra i rami di un composito; anche i retry consumano tentativi. |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | Tentativi aggiuntivi per errori transitori di una richiesta Google, intero 0–5. Zero significa un solo tentativo iniziale. I retry rispettano timeout e budget. |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | Scadenza complessiva dello strumento in secondi, intero 1–3600. Include acquisizione delle credenziali, attesa, chiamate API e retry. |

Al massimo cinque richieste HTTP Google possono essere attive contemporaneamente per processo. Il budget limita il numero di richieste, non la spesa: una matrice può fatturare più elementi con una sola richiesta. Alcuni compositi restituiscono errori per elemento quando un ramo esaurisce budget o scadenza. Consulta [produzione](../guide/production).

## Listener HTTP e accesso

| Variabile d'ambiente | Valore predefinito | Opzione HTTP | Significato e valori ammessi |
| --- | --- | --- | --- |
| `MCP_HTTP_TOKEN` | Non impostata | — | Bearer token condiviso dai client MCP. Obbligatorio per bind non loopback. Separato dalle credenziali Google e necessario anche con mTLS nativo. |
| `MCP_HTTP_LISTEN` | `127.0.0.1:3000` | `--listen` | Indirizzo IP e porta del listener. L'immagine Docker imposta il valore d'ambiente `0.0.0.0:3000`. |
| `MCP_HTTP_ALLOWED_HOSTS` | Host loopback | `--allowed-host` | Valori HTTP Host consentiti separati da virgole. Ripeti l'opzione per più valori; le opzioni sostituiscono l'intera lista da file e ambiente. Includi la porta pertinente. |
| `MCP_HTTP_ALLOWED_ORIGINS` | Nessuna | `--allowed-origin` | Origini browser consentite separate da virgole. Gli header Origin presenti sono rifiutati se non autorizzati. Specifica esplicitamente le porte; le opzioni ripetute sostituiscono la lista. |
| `MCP_HTTP_RATE_LIMIT` | `120` | `--rate-limit` | Richieste autenticate `/mcp` al minuto per processo, intero 0–1000000. Zero disabilita il limite. La capacità di burst equivale alla frequenza, con ricarica continua. |
| `MCP_METRICS_ENABLED` | `false` | `--metrics [true\|false]` | Abilita Prometheus `/metrics`. `--metrics` senza valore significa true. Usa la stessa credenziale bearer quando configurata; si applica anche mTLS nativo. |
| `MCP_TLS_CERT` | Non impostata | `--tls-cert` | File PEM con certificato server e catena intermedia. Deve essere fornito insieme alla chiave privata corrispondente. |
| `MCP_TLS_KEY` | Non impostata | `--tls-key` | Percorso della chiave privata server PEM. Deve essere leggibile dall'identità di esecuzione e restare fuori da repository e immagini. |
| `MCP_TLS_CLIENT_CA` | Non impostata | `--tls-client-ca` | Bundle PEM di CA client che rende obbligatori i certificati client su tutti i percorsi del listener. Non sostituisce il bearer token. |

Il token bucket HTTP è globale al processo, non per utente o IP. Le richieste non autorizzate non lo consumano; health e metriche sono escluse. I rifiuti restituiscono 429 con `Retry-After`. Il corpo della richiesta è limitato a 1 MiB. Consulta [accesso MCP](../guide/mcp) e [operazioni TLS](../guide/production).

## Eventi diagnostici

| Variabile d'ambiente | Valore predefinito | Opzione CLI | Significato e valori ammessi |
| --- | --- | --- | --- |
| `RUST_LOG` | `warn` | — | Filtro dei log diagnostici. `info` abilita gli eventi di completamento degli strumenti; argomenti e corpi delle risposte Google non vengono registrati. |
| `MCP_LOG_FORMAT` | `text` | `--log-format` | `text` oppure `json`. La diagnostica va sempre su stderr, lasciando stdout al JSON CLI o al traffico MCP. |

## Selezione del file e controlli di uscita

La selezione del file precede i livelli applicativi. `MCP_ENV_FILE` sceglie il percorso dotenv; l'opzione globale `--env-file PATH` ha precedenza. Un file selezionato esplicitamente deve esistere. `--no-env-file` disabilita tutto il caricamento dotenv ed è incompatibile con `--env-file`. Senza selezione si legge soltanto l'eventuale `.env` della directory di lavoro.

Una variabile d'ambiente vuota prevale comunque sul file. Può cancellare un token, percorso TLS o allowlist facoltativi, ma non rende valida un'impostazione numerica vuota. I valori caricati da file non modificano l'ambiente del processo.

`doctor --live` abilita esplicitamente sonde potenzialmente fatturabili; `doctor` ordinario è offline. L'opzione di strumento `--output PATH` salva JSON oppure PNG grezzo con `static-map`. Sono controlli di comando, non ulteriori impostazioni d'ambiente.

Impostazioni applicative e modifiche ai file delle credenziali richiedono un riavvio. Su Linux/macOS SIGHUP ricarica insieme i file di certificato, chiave e CA client TLS; un caricamento fallito conserva la configurazione precedente. Consulta [configurazione](../guide/configuration) e [produzione](../guide/production).
