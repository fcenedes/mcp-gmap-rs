# Usare la CLI

La CLI condivide i parametri con MCP. Ogni comando stampa JSON su stdout; errori e diagnostica vanno su stderr. Gli errori di chiamata producono un codice d'uscita diverso da zero. I risultati compositi possono comunque contenere errori parziali con codice d'uscita zero.

## Scoprire comandi e schemi

```sh
mcp-gmap-rs --help
mcp-gmap-rs tools
mcp-gmap-rs schema directions
mcp-gmap-rs directions --help
```

La scoperta non richiede credenziali. Usa lo schema della versione installata: un server remoto può esporre soltanto un sottoinsieme del catalogo.

## Opzioni dirette o JSON condiviso

```sh
mcp-gmap-rs geocode --address 'Roma, Italia'
mcp-gmap-rs search-places --query 'caffè a Roma' --open-now --min-rating 4
mcp-gmap-rs call maps_geocode --args '{"address":"Roma, Italia"}'
printf '%s' '{"address":"Roma, Italia"}' | mcp-gmap-rs call geocode --args -
```

I nomi camelCase diventano opzioni con trattini, per esempio `placeId` → `--place-id`. Solo i parametri di primo livello hanno opzioni proprie. Oggetti e array si passano come JSON:

```sh
mcp-gmap-rs search-nearby   --center '{"value":"41.8902,12.4922","isCoordinates":true}'   --keyword cafe --radius 500
```

Un'opzione booleana senza valore significa `true`; usa `--open-now false` per impostare `false`. Gli orari di percorso usano RFC 3339 con fuso esplicito. Il parametro `timestamp` di `timezone` usa invece millisecondi Unix.

## Salvare i risultati

```sh
mcp-gmap-rs geocode --address Roma --output roma.json
mcp-gmap-rs static-map --center '41.8902,12.4922' --zoom 14 --output roma.png
```

`--output` salva JSON, oppure PNG grezzo per Static Maps. Senza tale opzione, la mappa CLI usa JSON `{mimeType,base64}`. Non stampare credenziali negli script e non inserirle negli oggetti JSON degli strumenti.

Per batch, matrici e compositi, controlla `success`, `error`, `errors`, `details_error` e campi analoghi prima di usare il risultato. La [referenza strumenti](../reference/tools) descrive esattamente output, limiti ed errori per ogni operazione.
