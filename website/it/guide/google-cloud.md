# Preparare Google Cloud

Le chiamate usano il tuo progetto Google Cloud e possono generare costi. Abilitazione dell'API, fatturazione e restrizioni della chiave sono controlli distinti.

## Progetto e servizi

Nella [console Google Cloud](https://console.cloud.google.com/), seleziona o crea un progetto, collega un account di fatturazione attivo e apri **API e servizi → Libreria**. Abilita solo i servizi necessari.

| API | Servizio | Autenticazione in questa versione |
| --- | --- | --- |
| Places API (New) | `places.googleapis.com` | Chiave oppure OAuth |
| Routes | `routes.googleapis.com` | Chiave oppure OAuth |
| Weather | `weather.googleapis.com` | Chiave oppure OAuth |
| Air Quality | `airquality.googleapis.com` | Chiave oppure OAuth |
| Address Validation | `addressvalidation.googleapis.com` | Chiave oppure OAuth |
| Places Aggregate | `areainsights.googleapis.com` | Chiave oppure OAuth |
| Pollen | `pollen.googleapis.com` | Chiave |
| Roads | `roads.googleapis.com` | Chiave |
| Geocoding v3 | `geocoding-backend.googleapis.com` | Chiave |
| Elevation | `elevation-backend.googleapis.com` | Chiave |
| Time Zone | `timezone-backend.googleapis.com` | Chiave |
| Maps Static | `static-maps-backend.googleapis.com` | Chiave |

Un amministratore può abilitare tutti e dodici i servizi:

```sh
export PROJECT_ID='your-project-id'
gcloud services enable --project="$PROJECT_ID"   places.googleapis.com routes.googleapis.com weather.googleapis.com   airquality.googleapis.com addressvalidation.googleapis.com   areainsights.googleapis.com pollen.googleapis.com roads.googleapis.com   geocoding-backend.googleapis.com elevation-backend.googleapis.com   timezone-backend.googleapis.com static-maps-backend.googleapis.com
```

`directions` e `distance-matrix` usano Routes API: non occorrono le API legacy omonime. La risoluzione di indirizzi negli strumenti compositi può richiedere anche Geocoding.

## Creare una chiave

Apri [Google Maps Platform → Chiavi e credenziali](https://console.cloud.google.com/google/maps-apis/credentials), scegli **Crea credenziali → Chiave API**, quindi modifica subito le restrizioni. Consenti solo le API utilizzate. Per questa applicazione server, limita gli indirizzi IP agli IP pubblici di uscita del server o del cluster, non agli IP privati dei pod.

Salva la chiave in un Secret o in `.env`, mai negli argomenti degli strumenti:

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

Esegui `mcp-gmap-rs doctor` per la configurazione locale e una chiamata mirata per verificare l'accesso effettivo. `doctor --live` attiva esplicitamente sonde potenzialmente fatturabili. Un errore di API disabilitata va corretto nel progetto consumatore indicato da Google, che può differire dal progetto del service account. Continua con [autenticazione](./authentication) per OAuth e identità dei workload.


## Servizi aggiuntivi e limiti {#expanded-services}

Solar e Map Tiles supportano anche OAuth. Datasets e Route Optimization richiedono OAuth e permessi IAM per operazione. Street View Publish richiede un file authorized_user esplicito e il consenso per lo scope streetviewpublish.

| Servizio | API | Autenticazione |
| --- | --- | --- |
| Geolocation API | `geolocation.googleapis.com` | Chiave API |
| Solar API | `solar.googleapis.com` | Chiave API / OAuth |
| Aerial View API | `aerialview.googleapis.com` | Chiave API |
| Map Tiles API | `tile.googleapis.com` | Chiave API / OAuth |
| Street View Static API | `street-view-image-backend.googleapis.com` | Chiave API |
| Maps Platform Datasets API | `mapsplatformdatasets.googleapis.com` | OAuth / IAM |
| Route Optimization API | `routeoptimization.googleapis.com` | OAuth / IAM |
| Street View Publish API | `streetviewpublish.googleapis.com` | OAuth utente con consenso; account di servizio rifiutato |

Speed Limits richiede accesso Asset Tracking limitato. Aerial View copre indirizzi statunitensi supportati e vieta download e cache dei video. Map Tiles è destinato alla visualizzazione: analisi automatica ed estrazione non sono consentite. Weather minute/map sono sperimentali, con cache massima di un’ora e restrizioni d’uso. I riferimenti dei singoli strumenti descrivono condizioni, attribuzione e limiti.

[Google: Speed Limits](https://developers.google.com/maps/documentation/roads/speed-limits), [Map Tiles](https://developers.google.com/maps/documentation/tile/policies), [Aerial View](https://developers.google.com/maps/documentation/aerial-view/policies), [Street View Publish](https://developers.google.com/streetview/publish/authorizing).
