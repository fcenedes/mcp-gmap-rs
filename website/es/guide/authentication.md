# Autenticación

Configure las credenciales salientes de Google por separado de las que usan los clientes MCP HTTP. El modo de clave API es el predeterminado; indicar solo una ruta de cuenta de servicio no habilita OAuth.

## Compatibilidad de claves API y OAuth

| Familia de endpoints | Clave API | OAuth en este ejecutable |
| --- | --- | --- |
| Places (New), Routes, Weather, Air Quality, Address Validation, Places Aggregate | Sí | Sí |
| Geocoding v3, Elevation, Time Zone, Maps Static, Pollen, Roads | Sí | No |

La resolución de lugares por nombre y las llamadas compuestas pueden requerir una clave aunque su operación principal admita OAuth. Use coordenadas exactas para evitar geocodificaciones innecesarias. Los fallos OAuth nunca provocan una alternativa silenciosa con clave API. Consulte [la configuración de Google Cloud](./google-cloud) para habilitar los servicios exactos y restringir una clave.

## OAuth con una cuenta de servicio

En modo OAuth, las solicitudes a Places (New), Routes, Weather, Air Quality, Address Validation y Places Aggregate reciben un **token de acceso** OAuth y el encabezado `X-Goog-User-Project`. El proyecto solicita el alcance `https://www.googleapis.com/auth/cloud-platform` documentado por esas APIs. La biblioteca de autenticación Rust de Google obtiene, almacena en caché y renueva los tokens. `GOOGLE_MAPS_TIMEOUT` también limita la inicialización de credenciales y la obtención del token. El flujo de cuenta de servicio no necesita pegar manualmente un token ni una pantalla de consentimiento OAuth. Consulte [la guía de OAuth de Maps para aplicaciones de servidor](https://developers.google.com/maps/api-security-best-practices#use-oauth-for-server-side-apps).

Cree una cuenta de servicio dedicada y concédale permiso para consumir servicios en el proyecto usado para la facturación y la cuota de Maps:

```sh
export PROJECT_ID='your-project-id'
export SA_EMAIL="mcp-gmap@${PROJECT_ID}.iam.gserviceaccount.com"
gcloud iam service-accounts create mcp-gmap --project="$PROJECT_ID" \
  --display-name='MCP Google Maps'
gcloud projects add-iam-policy-binding "$PROJECT_ID" \
  --member="serviceAccount:${SA_EMAIL}" \
  --role='roles/serviceusage.serviceUsageConsumer'
```

El rol Service Usage Consumer incluye `serviceusage.services.use` para cuota y facturación. No permite habilitar servicios ni crear claves; esas tareas requieren un administrador con autorización independiente. Habilite las APIs Maps en el proyecto de cuota con la [guía de Google Cloud](./google-cloud). Consulte [los permisos de Service Usage](https://docs.cloud.google.com/service-usage/docs/access-control).

Elija una de las fuentes de credenciales siguientes. La misma configuración funciona con CLI, stdio y HTTP, respetando la precedencia archivo → entorno → opción.

### A. Clave JSON montada de cuenta de servicio {#a-mounted-service-account-json-key}

Si su organización permite claves JSON, abra **IAM & Admin → Service Accounts**, seleccione la cuenta y después **Keys → Add key → Create new key → JSON**. Guarde el archivo fuera del repositorio. Como alternativa:

```sh
mkdir -p "$HOME/.config/mcp-gmap-rs"
gcloud iam service-accounts keys create \
  "$HOME/.config/mcp-gmap-rs/service-account.json" \
  --iam-account="$SA_EMAIL" --project="$PROJECT_ID"
chmod 600 "$HOME/.config/mcp-gmap-rs/service-account.json"
export GOOGLE_APPLICATION_CREDENTIALS="$HOME/.config/mcp-gmap-rs/service-account.json"
export GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
export GOOGLE_MAPS_AUTH_MODE=oauth
```

Para un archivo montado, el `.env` equivalente es:

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
# Optional, for Geocoding v3 / Elevation / Time Zone / Static Maps / Pollen / Roads:
# GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

Establezca la ruta de credenciales solo para un archivo JSON legible; no use una cadena vacía ni el contenido del JSON. Las rutas relativas se resuelven desde el directorio de trabajo. Los valores del archivo se usan directamente, sin modificar el entorno del proceso. El cargador también acepta configuraciones ADC JSON `external_account`, `impersonated_service_account` y `authorized_user`. Reinicie el proceso al cambiar un archivo de credenciales.

Las claves de cuentas de servicio son credenciales de larga duración. Prefiera las opciones sin clave cuando estén disponibles; mantenga las claves JSON fuera de Git y de las capas de imagen, y rote o elimine las que no use. Google documenta [la creación de claves y las políticas de organización](https://docs.cloud.google.com/iam/docs/keys-create-delete).

### B. Application Default Credentials sin clave JSON {#b-application-default-credentials-without-a-json-key}

Deje `GOOGLE_APPLICATION_CREDENTIALS` sin definir tanto en el entorno del proceso como en el archivo dotenv seleccionado o el Secret/ConfigMap de Kubernetes. Un `unset` de shell no elimina un valor de la capa de archivo; quite la entrada en lugar de establecer una ruta vacía. Después, configure:

```dotenv
GOOGLE_MAPS_AUTH_MODE=oauth
GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id
```

La biblioteca oficial usa [Application Default Credentials (ADC)](https://docs.cloud.google.com/docs/authentication/application-default-credentials): credenciales ADC locales si existen y, en caso contrario, la cuenta de servicio o de carga de trabajo asociada a través del servidor de metadatos. El descubrimiento de credenciales es diferido: `tools`, `schema` y la ayuda funcionan sin obtener un token. Una llamada API OAuth necesita un proyecto de cuota y credenciales válidas.

Para desarrollar localmente mediante suplantación de cuenta de servicio, habilite IAM Credentials API, conceda Token Creator a su cuenta de desarrollador sobre la cuenta de servicio y cree credenciales ADC locales:

```sh
gcloud services enable iamcredentials.googleapis.com --project="$PROJECT_ID"
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
  --project="$PROJECT_ID" --member='user:you@example.com' \
  --role='roles/iam.serviceAccountTokenCreator'
unset GOOGLE_APPLICATION_CREDENTIALS
gcloud auth application-default login --impersonate-service-account="$SA_EMAIL"
export GOOGLE_MAPS_AUTH_MODE=oauth
export GOOGLE_CLOUD_QUOTA_PROJECT="$PROJECT_ID"
```

Así se usa un token de cuenta de servicio de corta duración para Maps sin descargar su clave privada. Consulte [la suplantación de cuentas de servicio](https://docs.cloud.google.com/iam/docs/service-account-impersonation).

### C. Identidad de cargas de trabajo en Kubernetes y GKE {#c-kubernetes-and-gke-workload-identity}

En Kubernetes genérico, monte el JSON de credenciales como archivo Secret y configure las tres variables OAuth de la opción A. El archivo debe ser legible por el UID 10001 del contenedor. Móntelo en modo de solo lectura y mantenga `MCP_HTTP_TOKEN` en su propio Secret de ejecución. La [guía de contenedores](./deployment) explica el Deployment base.

En GKE, prefiera [Workload Identity Federation for GKE](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/workload-identity), con una ServiceAccount de Kubernetes vinculada a la cuenta de servicio IAM. Habilítela primero en el clúster y el grupo de nodos; Autopilot ya la habilita. Después:

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

Estos comandos suponen que el clúster y la cuenta IAM están en `PROJECT_ID` y el Deployment base en `default`. Adapte los proyectos y namespaces a su instalación. No defina `GOOGLE_APPLICATION_CREDENTIALS` para este flujo de metadatos. El pod obtiene tokens a través del servidor de metadatos de GKE. Fuera de GKE puede usarse una configuración `external_account` junto con el token de identidad proyectado del proveedor mediante Workload Identity Federation de Google.

Para operar solo con OAuth, elimine la clave API de Google del Secret de ejecución y habilite únicamente herramientas y entradas cuyas llamadas permanezcan en APIs compatibles con OAuth. Conserve la clave para los endpoints de la tabla que solo aceptan claves o para los flujos compuestos que los necesiten. Los errores OAuth en APIs compatibles fallan explícitamente; nunca cambian silenciosamente a una clave.

## Verificar y resolver problemas

```sh
mcp-gmap-rs --auth-mode oauth search-places --query 'coffee in Paris'
mcp-gmap-rs --auth-mode oauth weather --latitude 48.8584 --longitude 2.2945
```

Son llamadas reales y potencialmente facturables. Compruebe el modo efectivo y el proyecto de cuota al probar. OAuth factura mediante `GOOGLE_CLOUD_QUOTA_PROJECT`, mientras que las llamadas con clave usan el proyecto de la clave; pueden ser proyectos distintos.

Use `mcp-gmap-rs doctor` para comprobar la configuración sin conexión, o `doctor --live` para sondas API explícitamente facturables. La [guía de producción](./production) describe el informe y el filtrado de herramientas.

Con las variables OAuth exportadas, ejecute las seis pruebas de aceptación en vivo opcionales:

```sh
cargo test --locked --test live live_oauth_ -- --ignored --test-threads=1
```

Estas pruebas eliminan explícitamente la clave API de cada proceso hijo, por lo que una prueba aprobada ejercita OAuth. Cubren Places, Routes, Weather, Air Quality, Address Validation y Places Aggregate, y requieren esas APIs habilitadas. No cargan `.env`: exporte la configuración OAuth o ejecute en una carga de trabajo configurada. Las pruebas normales y CI usan fixtures locales de tokens/APIs, sin cuenta de servicio ni contacto con Google.

| Síntoma | Comprobación |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | La operación seleccionada alcanza un endpoint exclusivo de clave, o está seleccionado el modo de clave API. |
| Fallo de autenticación OAuth | Revise ruta y tipo de credenciales, validez de la clave, vínculo de identidad o suplantación, IAM Credentials API si se usa, conexión al endpoint de token y precisión del reloj. Los errores de la aplicación omiten el contenido de credenciales y tokens. |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | Establezca el ID o número del proyecto consumidor de Maps para OAuth. |
| Google `403` / API deshabilitada | Habilite el servicio exacto en el proyecto consumidor; compruebe facturación y permiso Service Usage. |
| Clave API rechazada por IP | Haga coincidir la restricción con la IP pública de salida del servidor o clúster. |
| Tiempo de espera del token | Permita el acceso a endpoints de token/STS/IAM de Google o al servidor de metadatos de la carga; revise `GOOGLE_MAPS_TIMEOUT`. |
| HTTP `401` de este servidor MCP | Envíe su `MCP_HTTP_TOKEN`; los tokens de cuenta de servicio de Google autentican una conexión saliente distinta. |
