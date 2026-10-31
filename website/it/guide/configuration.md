# Configurazione a livelli

L'ordine di precedenza è **valori predefiniti → file dotenv facoltativo → variabili d'ambiente → opzioni CLI**. Lo stesso modello vale per CLI, stdio e HTTP.

## Selezionare il file

Per impostazione predefinita, viene cercato solo `.env` nella directory di lavoro, senza risalire le directory superiori. La sua assenza è consentita. Un file scelto esplicitamente deve esistere:

```sh
mcp-gmap-rs --env-file /etc/mcp-gmap/runtime.env serve http
mcp-gmap-rs --no-env-file tools
```

`MCP_ENV_FILE` seleziona un percorso, ma `--env-file` ha precedenza. `--no-env-file` disabilita completamente il caricamento. File illeggibili o malformati interrompono l'avvio senza stamparne il contenuto. Il parser non modifica l'ambiente del processo.

## Sovrascrivere solo ciò che serve

```dotenv
GOOGLE_MAPS_LANGUAGE=it
GOOGLE_MAPS_TIMEOUT=30
GOOGLE_MAPS_MAX_REQUESTS=100
MCP_LOG_FORMAT=json
```

```sh
GOOGLE_MAPS_LANGUAGE=fr mcp-gmap-rs --language it tools
```

In questo esempio prevale `it`. Anche una variabile d'ambiente esplicitamente vuota sostituisce il valore nel file: può cancellare un token facoltativo, ma un timeout vuoto non è valido. Opzioni ripetute come `--allowed-host` e `--allowed-origin` sostituiscono l'intera lista configurata, senza aggiungersi ai valori del file.

## Limitare gli strumenti esposti

```sh
mcp-gmap-rs --enabled-tools maps_search_places,maps_place_details serve stdio
```

I nomi sono quelli MCP `maps_*`; nomi sconosciuti impediscono l'avvio. `*` abilita l'intero catalogo. Ridurre il catalogo aiuta il client a scegliere gli strumenti pertinenti; non sostituisce le restrizioni della credenziale Google.

Le impostazioni si leggono all'avvio: riavvia il processo o i pod dopo una modifica a Secret e ConfigMap. I file di certificato, chiave TLS e CA client fanno eccezione: supportano il ricaricamento con SIGHUP, descritto in [produzione](./production). Il [riferimento completo](../reference/configuration) elenca tutte le variabili, i valori predefiniti e le opzioni corrispondenti.
