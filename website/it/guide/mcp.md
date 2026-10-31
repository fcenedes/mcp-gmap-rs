# Collegare un client MCP

MCP espone gli stessi 23 strumenti della CLI. Scegli stdio quando il client avvia il processo locale; scegli Streamable HTTP per un servizio condiviso o remoto. Non serve un plugin specifico per l'assistente.

## Trasporto stdio

```sh
mcp-gmap-rs serve stdio
```

Anche l'eseguibile senza argomenti avvia stdio. Il protocollo JSON-RPC MCP occupa stdin/stdout; la diagnostica resta su stderr. Esempio di configurazione generica da adattare al client:

```json
{
  "mcpServers": {
    "google-maps": {
      "command": "/absolute/path/to/mcp-gmap-rs",
      "args": ["serve", "stdio"],
      "env": { "GOOGLE_MAPS_API_KEY": "your-restricted-api-key" }
    }
  }
}
```

I client possono avere un `PATH` e una directory di lavoro diversi dal terminale. Usa un percorso assoluto e configura esplicitamente l'ambiente o `--env-file`. Non avviare stdio con un TTY quando lo esegui in Docker.

## Streamable HTTP

```sh
mcp-gmap-rs serve http --listen 127.0.0.1:3000
```

Configura il client sull'URL `http://127.0.0.1:3000/mcp`. Un client MCP gestisce inizializzazione, versione del protocollo, sessioni e SSE; una semplice richiesta GET non equivale a una chiamata di strumento. Il trasporto supporta POST, GET e DELETE secondo il protocollo negoziato.

Per un client che supporta una connessione MCP remota, la voce di configurazione può avere questa forma; il contenitore esterno dipende dal client:

```json
{
  "url": "https://maps.example.com/mcp",
  "headers": {
    "Authorization": "Bearer your-mcp-http-token"
  }
}
```

Per un bind non loopback, imposta un token casuale lungo e gli host consentiti:

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3000   --allowed-host maps.example.com   --allowed-origin https://app.example.com:443
```

Fornisci `MCP_HTTP_TOKEN` tramite ambiente o Secret; il client invia `Authorization: Bearer <token>`. Gli header Origin presenti sono rifiutati se non autorizzati esplicitamente. Un proxy deve preservare Host consentito, autorizzazione, header MCP e streaming SSE.

`GET /health` verifica solo la disponibilità del processo, non l'accesso Google. Il corpo delle richieste è limitato a 1 MiB. Le sessioni risiedono in memoria: più repliche richiedono affinità e i client devono riconnettersi dopo un riavvio. Per TLS e sicurezza operativa, continua con [produzione](./production).
