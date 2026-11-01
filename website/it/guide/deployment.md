# Docker e Kubernetes

Il [Dockerfile](https://github.com/fcenedes/mcp-gmap-rs/blob/main/Dockerfile) compila un eseguibile Rust e usa un runtime Debian con certificati CA. Il processo gira come UID/GID 10001; le credenziali vengono fornite all'esecuzione, mai durante la build.

## Costruire ed eseguire l'immagine

```sh
docker build -t mcp-gmap-rs:local .
docker run --rm --read-only --cap-drop ALL   --security-opt no-new-privileges --env-file .env   -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

L'immagine predefinita avvia HTTP su `0.0.0.0:3000`: `.env` deve quindi contenere `MCP_HTTP_TOKEN`, oltre alla configurazione Google. Docker `--env-file` inietta variabili; per provare il loader applicativo, monta invece un file leggibile e usa `--env-file /etc/mcp-gmap/runtime.env` dopo il nome dell'immagine.

```sh
docker run --rm mcp-gmap-rs:local tools
docker run --rm -i --env-file .env mcp-gmap-rs:local serve stdio
```

Per pubblicare la tua immagine multiarchitettura:

```sh
docker buildx build --platform linux/amd64,linux/arm64   -t registry.example.com/team/mcp-gmap-rs:0.4.0 --push .
```

La pubblicazione automatica in un registro di container non è configurata.

## Distribuire su Kubernetes

Il [manifest base](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml) usa ConfigMap, Secret esistente, Deployment e Service ClusterIP. Da un checkout, sostituisci l'immagine locale con la tua immagine di registro:

```sh
kubectl create secret generic mcp-gmap-rs --from-env-file=/secure/credentials.env
kubectl apply -f deploy/kubernetes.yaml
kubectl rollout status deployment/mcp-gmap-rs
kubectl port-forward service/mcp-gmap-rs 3000:3000
```

Il file del Secret contiene le credenziali Google e `MCP_HTTP_TOKEN`. Il manifest usa `--no-env-file` e `envFrom`; i valori modificati richiedono il riavvio dei pod. Per OAuth JSON, monta la credenziale in sola lettura con permessi adatti a UID 10001. Su GKE preferisci [Workload Identity](./authentication).

Mantieni una replica con il session store predefinito, oppure configura affinità. Consenti DNS e HTTPS in uscita; le restrizioni IP della chiave devono riconoscere l'egress pubblico del cluster. Adatta gli host consentiti al namespace e al proxy effettivi. Il [Gateway HTTPS facoltativo](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/gateway.yaml) richiede CRD Gateway API, controller conforme, dominio e Secret TLS. I dettagli su streaming, probe e cifratura del collegamento al pod sono in [produzione](./production).


## OAuth e file montati nel container

Un file dotenv montato viene letto dall'applicazione, mentre le variabili iniettate con `-e` lo sovrascrivono. L'immagine imposta già `MCP_HTTP_LISTEN=0.0.0.0:3000`, quindi un valore diverso nel file non prevale su quello dell'immagine: usa ambiente oppure `--listen`.

```sh
docker run --rm --read-only \
  --mount type=bind,src="$(pwd)/runtime.env",dst=/etc/mcp-gmap/runtime.env,readonly \
  --mount type=bind,src=/secure/service-account.json,dst=/credentials/service-account.json,readonly \
  -e GOOGLE_MAPS_AUTH_MODE=oauth \
  -e GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json \
  -e GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id \
  -p 127.0.0.1:3000:3000 mcp-gmap-rs:local \
  --env-file /etc/mcp-gmap/runtime.env serve http
```

`runtime.env` deve contenere il token MCP; il JSON deve essere leggibile da UID 10001. Una chiave API rimane necessaria per gli endpoint supportati solo con chiave se vengono usati. Non copiare il JSON nel Dockerfile e non inserire credenziali nella riga di comando.

Per salvare risultati CLI con `--output`, monta una directory scrivibile dall'utente del container. Le impostazioni di risorse del manifest sono valori iniziali da adattare al carico. Per rollout con sessioni esistenti prevedi riconnessione dei client: lo stato in memoria non migra tra pod. Le probe HTTP verificano il processo, non validità della credenziale o disponibilità dell'API Google.
