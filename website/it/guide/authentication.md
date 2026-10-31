# Chiavi API e OAuth 2.0

L'autenticazione verso Google è separata da quella dei client MCP. `GOOGLE_MAPS_AUTH_MODE` seleziona la credenziale in uscita; `MCP_HTTP_TOKEN` protegge il server HTTP in ingresso.

## Service account con file JSON

Crea un service account dedicato e concedigli **Service Usage Consumer** sul progetto di fatturazione/quota. Il ruolo include `serviceusage.services.use`; non concede il diritto di abilitare servizi.

```sh
export PROJECT_ID='your-project-id'
export SA_EMAIL="mcp-gmap@${PROJECT_ID}.iam.gserviceaccount.com"
gcloud iam service-accounts create mcp-gmap --project="$PROJECT_ID"
gcloud projects add-iam-policy-binding "$PROJECT_ID"   --member="serviceAccount:${SA_EMAIL}"   --role='roles/serviceusage.serviceUsageConsumer'
```

Se la tua organizzazione ammette chiavi JSON, creane una da **IAM e amministrazione → Service account → Chiavi**. Conserva il file fuori dal repository, leggibile solo dall'identità del processo:

```sh
chmod 600 /secure/service-account.json
```

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_APPLICATION_CREDENTIALS=/secure/service-account.json
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
```

Il percorso punta a un file, non al contenuto JSON. I percorsi relativi partono dalla directory di lavoro. La libreria Google acquisisce, memorizza temporaneamente e rinnova i token; le richieste supportate inviano anche `X-Goog-User-Project`. Riavvia il processo dopo una modifica delle credenziali.

## ADC e identità senza chiavi

Lascia `GOOGLE_APPLICATION_CREDENTIALS` non impostata sia nell'ambiente del processo **sia** nel file dotenv o Secret/ConfigMap selezionato per usare Application Default Credentials locali o l'identità collegata al workload. `unset` da solo non rimuove un valore nel file: elimina la voce invece di impostare un percorso vuoto. Per sviluppo con impersonazione:

```sh
gcloud services enable iamcredentials.googleapis.com --project="$PROJECT_ID"
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL"   --project="$PROJECT_ID" --member='user:you@example.com'   --role='roles/iam.serviceAccountTokenCreator'
gcloud auth application-default login --impersonate-service-account="$SA_EMAIL"
```

Imposta comunque modalità `oauth` e progetto quota. Su GKE usa Workload Identity Federation, un Kubernetes ServiceAccount associato all'identità IAM e il metadata server; non impostare un percorso JSON per questo flusso. Questi comandi assumono cluster e service account IAM nello stesso `PROJECT_ID`, con il Deployment nel namespace `default`. Abilita prima Workload Identity sul cluster e sul node pool; GKE Autopilot lo abilita già.

```sh
gcloud services enable iamcredentials.googleapis.com --project="$PROJECT_ID"
kubectl create serviceaccount mcp-gmap-rs --namespace default
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
  --project="$PROJECT_ID" --role='roles/iam.workloadIdentityUser' \
  --member="serviceAccount:${PROJECT_ID}.svc.id.goog[default/mcp-gmap-rs]"
kubectl annotate serviceaccount mcp-gmap-rs --namespace default \
  "iam.gke.io/gcp-service-account=${SA_EMAIL}"
kubectl patch deployment mcp-gmap-rs --namespace default --type=merge \
  -p '{"spec":{"template":{"spec":{"serviceAccountName":"mcp-gmap-rs"}}}}'
kubectl set env deployment/mcp-gmap-rs --namespace default \
  GOOGLE_MAPS_AUTH_MODE=oauth GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
```

Adatta namespace e progetti alla tua installazione. Il pod acquisisce token dal metadata server GKE; non impostare `GOOGLE_APPLICATION_CREDENTIALS` per questo flusso. Su Kubernetes generico puoi montare il JSON in sola lettura con le tre variabili della sezione precedente. All'esterno di GKE, Workload Identity Federation può usare una configurazione ADC `external_account` e il token proiettato del provider.

Il loader ammette anche configurazioni ADC JSON `impersonated_service_account` e `authorized_user`. Non impostare il percorso delle credenziali a una stringa vuota. La scoperta `tools` e `schema` è lazy: non acquisisce token. Il primo accesso OAuth richiede invece quota project, identità valida e connettività agli endpoint del flusso scelto.

Per creare una chiave JSON da CLI, se consentito dalle policy:

```sh
mkdir -p "$HOME/.config/mcp-gmap-rs"
gcloud iam service-accounts keys create \
  "$HOME/.config/mcp-gmap-rs/service-account.json" \
  --iam-account="$SA_EMAIL" --project="$PROJECT_ID"
chmod 600 "$HOME/.config/mcp-gmap-rs/service-account.json"
```

Le chiavi dei service account sono credenziali di lunga durata: escludile da Git e dalle immagini, controllane gli accessi e revoca quelle inutilizzate. Dove disponibile, un'identità del workload o l'impersonazione evita la distribuzione di chiavi private.

## Confini del supporto

OAuth copre Places (New), Routes, Weather, Air Quality, Address Validation e Places Aggregate. Geocoding v3, Elevation, Time Zone, Static Maps, Pollen e Roads richiedono ancora una chiave in questa implementazione. Mantieni una chiave limitata se utilizzi questi endpoint; coordinate esatte evitano alcune risoluzioni Geocoding nei compositi.

Un errore OAuth non provoca mai un ripiego silenzioso sulla chiave. Per un'istanza esclusivamente OAuth, rimuovi la chiave e limita strumenti e input alle API supportate. Verifica una chiamata reale con `--auth-mode oauth search-places --query 'caffè a Roma'`; è potenzialmente fatturabile.
