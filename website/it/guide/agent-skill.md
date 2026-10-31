# Usare la skill per agenti

La skill riutilizzabile `mcp-gmap` aiuta gli agenti a scegliere gli strumenti Maps, usare MCP o CLI, controllare errori parziali e mantenere le attribuzioni Google. Contiene istruzioni e riferimenti: non installa il binario e non crea una connessione MCP.

## Installazione

Gli archivi di release e il pacchetto sorgente includono `skills/mcp-gmap/`. Da un checkout o archivio estratto, installala nella directory di skill supportata dal tuo agente. Per Codex:

```sh
mkdir -p "${CODEX_HOME:-$HOME/.codex}/skills"
cp -R skills/mcp-gmap "${CODEX_HOME:-$HOME/.codex}/skills/"
```

Il [file principale](https://github.com/fcenedes/mcp-gmap-rs/blob/main/skills/mcp-gmap/SKILL.md) è indipendente dall'agente. `agents/openai.yaml` aggiunge metadati facoltativi per l'interfaccia Codex; non trasforma il progetto in un plugin.

Configura prima la [CLI](./cli) o la [connessione MCP](./mcp), con credenziali Google appropriate. La skill deve scoprire gli strumenti effettivamente disponibili prima di usare funzionalità dipendenti dalla versione. Un'istanza può esporre solo la propria allowlist.

## Esempio di richiesta

> Usa $mcp-gmap per cercare tre caffè vicino al Colosseo e confrontare i tragitti a piedi. Conserva i link Google e segnala eventuali dati mancanti.

L'agente può cercare luoghi, recuperare dettagli soltanto per i candidati utili e confrontare percorsi con modalità esplicita `walking`. Non deve presumere che `compare-places` permetta una modalità diversa da `driving`, né interpretare un risultato composito senza controllarne gli errori per elemento.

Per riprodurre una chiamata usa `schema TOOL` e il [riferimento completo](../reference/tools). I parametri di strumento non contengono chiavi o token: le credenziali restano configurazione del processo. Per richieste ambientali, l'agente deve conservare unità, orari e limiti di copertura, senza presentare le raccomandazioni del fornitore come una diagnosi personale.
