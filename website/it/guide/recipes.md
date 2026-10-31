# Ricette operative

Questi esempi effettuano richieste Google potenzialmente fatturabili. Coordinate, categorie e numero di risultati restano espliciti per rendere prevedibile il lavoro. Adatta la zona e verifica sempre la copertura restituita.

## Cercare e approfondire

```sh
mcp-gmap-rs search-nearby   --center '{"value":"41.8902,12.4922","isCoordinates":true}'   --keyword cafe --radius 500 --min-rating 4
mcp-gmap-rs place-details --place-id PLACE_ID --max-photos 1
```

Sostituisci `PLACE_ID` con un identificativo restituito dalla ricerca. Conserva link Google, autori delle recensioni/foto e avvertenze relative ai riepiloghi AI. Una pagina di ricerca non costituisce un censimento completo delle attività.

## Confrontare tragitti a piedi

```sh
mcp-gmap-rs distance-matrix   --origins '["41.8902,12.4922"]'   --destinations '["41.9009,12.4833","41.9028,12.4964"]' --mode walking
```

Controlla ogni cella e `errors`: le celle mancanti o fallite rimangono `null`. Le durate sono secondi e le distanze metri. Le coordinate evitano risoluzioni di indirizzi; `compare-places` calcola invece distanze in auto, senza un parametro per cambiare modalità.

## Pianificare tappe e soste

```sh
mcp-gmap-rs plan-route   --stops '["41.8902,12.4922","41.9009,12.4833","41.9028,12.4964","41.9109,12.4764"]'   --mode walking --optimize
mcp-gmap-rs search-along-route --origin '41.8902,12.4922'   --destination '41.9109,12.4764' --text-query cafes --max-results 3
```

L'ottimizzazione mantiene fissi gli estremi e diventa effettiva con almeno quattro tappe totali, fuori da `transit`. Non assegna veicoli o incarichi a una flotta. `search-along-route` usa `walking` come modalità predefinita; gli altri strumenti di percorso usano `driving`.

## Aggiungere il contesto ambientale

```sh
mcp-gmap-rs weather --latitude 41.8902 --longitude 12.4922 --type forecast_daily --forecast-days 3
mcp-gmap-rs air-quality --latitude 41.8902 --longitude 12.4922 --include-pollutants
mcp-gmap-rs pollen --latitude 41.8902 --longitude 12.4922 --days 3
```

Mantieni le unità e gli orari presenti nei risultati. Pollen richiede una chiave anche in modalità OAuth; le raccomandazioni ambientali non sono una valutazione sanitaria personalizzata. Per verificare componenti postali usa [Address Validation](../reference/tools#maps_validate_address), non la sola geocodifica. Per un conteggio aggregato in un'area usa [Area Insights](../reference/tools#maps_area_insights), distinguendolo dal numero di risultati di una ricerca limitata.


## Validare un indirizzo e contare attività

```sh
mcp-gmap-rs validate-address --address-lines '["Piazza del Colosseo, 1"]' \
  --locality Roma --postal-code 00184 --region-code IT
mcp-gmap-rs area-insights \
  --area '{"circle":{"latLng":{"latitude":41.8902,"longitude":12.4922},"radius":500}}' \
  --included-types '["restaurant"]' --min-rating 4
```

La validazione restituisce verdetto, correzioni e componenti irrisolti: non accettare automaticamente un suggerimento e non garantire la consegna. USPS CASS è facoltativo e questa implementazione richiede regione US o PR. Il conteggio aggregato usa normalmente una stringa int64; gli identificativi facoltativi dei luoghi sono disponibili solo fino a 100 corrispondenze e non comportano richieste automatiche di dettagli.

## Riutilizzare coordinate e abbinare punti stradali

```sh
mcp-gmap-rs timezone --latitude 41.8902 --longitude 12.4922
mcp-gmap-rs elevation --locations '[{"latitude":41.8902,"longitude":12.4922}]'
mcp-gmap-rs nearest-roads --points '[{"latitude":41.8902,"longitude":12.4922}]'
mcp-gmap-rs static-map --center '41.8902,12.4922' --zoom 14 --output zona.png
```

`timezone` accetta millisecondi Unix quando viene specificato `timestamp`; Elevation restituisce metri. Usa Snap to Roads per una traccia ordinata e Nearest Roads per osservazioni indipendenti. Conserva `originalIndex`; i punti interpolati possono non averlo. I limiti di velocità non sono esposti.

Per un'analisi esplicita di visibilità, [Local Rank Tracker](../reference/tools#maps_local_rank_tracker) campiona una griglia per parola chiave. Inizia con una parola e una griglia 3×3; una griglia più grande moltiplica le chiamate. Posizionamento e valutazione degli utenti sono metriche diverse, e i punti falliti devono restare visibili. Non usare una ricerca di categoria come sostituto di un conteggio aggregato o una previsione come se fosse un'osservazione corrente.
