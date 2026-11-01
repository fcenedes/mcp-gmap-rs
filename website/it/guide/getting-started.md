# Primi passi

La versione 0.4.0 offre 42 strumenti in un unico eseguibile, disponibili dal terminale o da un client MCP. Questo manuale descrive installazione, accesso a Google e interfaccia completa degli strumenti.

Dalla copia locale del repository:

```sh
cargo install --path . --locked
```

## Installare

Con Rust 1.88 o successivo:

```sh
cargo install mcp-gmap-rs --version 0.4.0 --locked
mcp-gmap-rs --version
```

In alternativa, scarica l'archivio Linux x86_64, macOS Apple Silicon o macOS Intel dalle [release](https://github.com/fcenedes/mcp-gmap-rs/releases/tag/v0.4.0), verifica `SHA256SUMS` e colloca il binario nel `PATH`. L'host deve disporre di certificati CA e accesso HTTPS a Google.

## Esplorare senza credenziali

```sh
mcp-gmap-rs tools
mcp-gmap-rs schema search-nearby
mcp-gmap-rs doctor
```

La scoperta degli strumenti e degli schemi non contatta Google. `doctor` esegue controlli locali; non dimostra che una chiave sia autorizzata.

## Eseguire la prima chiamata

Configura prima progetto, fatturazione, API e chiave seguendo [Google Cloud](./google-cloud). Salva una chiave limitata in un file `.env` locale escluso da Git:

```dotenv
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

Poi esegui una chiamata reale, potenzialmente fatturabile:

```sh
mcp-gmap-rs geocode --address 'Roma, Italia'
```

Il risultato JSON contiene coordinate, indirizzo e identificativo del primo luogo corrispondente. Per [OAuth con service account](./authentication), scegli esplicitamente `oauth` e usa un'operazione supportata, ad esempio `search-places`.

## Scegliere l'interfaccia

Usa la [CLI](./cli) per script e automazioni, [stdio](./mcp) per un agente locale, oppure HTTP per client remoti. Tutte le interfacce condividono gli stessi schemi, limiti e risultati. Il [riferimento degli strumenti](../reference/tools) descrive ogni parametro, valore predefinito ed esempio.
