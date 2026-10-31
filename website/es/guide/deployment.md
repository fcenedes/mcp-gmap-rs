# Despliegue

Para crear claves API, habilitar servicios y usar OAuth de cuentas de servicio, siga [la guía de Google Cloud](./google-cloud). La misma configuración por capas funciona en contenedores. OAuth puede usar un archivo JSON montado o la identidad ADC de la carga de trabajo; las credenciales se proporcionan durante la ejecución.

Descargue o clone [el repositorio](https://github.com/fcenedes/mcp-gmap-rs) y compile desde su raíz:

```sh
docker build -t mcp-gmap-rs:local .
```

El Dockerfile multietapa compila con Rust 1.94.1 y el archivo de bloqueo. La imagen de ejecución contiene el ejecutable, certificados CA de Debian y licencias; usa UID/GID 10001 y admite un sistema de archivos raíz de solo lectura. La lista permitida del contexto de compilación excluye `.env` locales, datos Git y resultados de compilación. No hacen falta credenciales para compilar. BuildKit almacena las descargas Cargo y los resultados para compilaciones posteriores.

El mismo Dockerfile compila de forma nativa en Linux amd64 y arm64. Para compilar ambas y publicarlas en su registro con un builder Buildx configurado:

```sh
docker buildx build --platform linux/amd64,linux/arm64 \
  -t registry.example.com/your-team/mcp-gmap-rs:YOUR_VERSION --push .
```

No hay publicación automática de contenedores en un registro. Sustituya nombres y versiones por la imagen que haya compilado o publicado.

## Ejecutar HTTP, CLI o stdio

La imagen ejecuta por defecto `serve http` en `0.0.0.0:3000`. Proporcione un token bearer en ejecución, por ejemplo con el formato `.env` mostrado en `.env.example`:

```sh
docker run --rm --read-only --cap-drop ALL --security-opt no-new-privileges \
  --env-file .env -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

Compruebe que `.env` contiene `MCP_HTTP_TOKEN`, obligatorio porque el contenedor escucha fuera de loopback. Las llamadas Google usan `GOOGLE_MAPS_API_KEY` o la configuración OAuth y sus montajes de credenciales, según las herramientas habilitadas. Los clientes usan `http://127.0.0.1:3000/mcp` y `Authorization: Bearer <token>`. `GET /health` no necesita token ni contacta con Google.

La opción `--env-file` de Docker inyecta variables de entorno. Para usar en cambio el cargador de archivos de la aplicación, monte un archivo legible y selecciónelo:

```sh
docker run --rm --read-only \
  --mount type=bind,src="$(pwd)/.env",dst=/etc/mcp-gmap/runtime.env,readonly \
  -e GOOGLE_MAPS_LANGUAGE=fr -p 127.0.0.1:3000:3000 \
  mcp-gmap-rs:local --env-file /etc/mcp-gmap/runtime.env serve http
```

El entorno prevalece sobre el archivo montado y las opciones CLI prevalecen sobre ambos. `MCP_HTTP_LISTEN=0.0.0.0:3000` de la imagen también es un valor de entorno; cámbielo con `-e MCP_HTTP_LISTEN=...` o `--listen` si es necesario. Para usar exclusivamente variables de entorno, pase `--no-env-file serve http`.

Sobrescriba el comando de la imagen para CLI o stdio:

```sh
docker run --rm mcp-gmap-rs:local tools
docker run --rm --env-file .env mcp-gmap-rs:local geocode --address Paris
docker run --rm -i --env-file .env mcp-gmap-rs:local serve stdio
```

Use `-i` para stdio, sin asignar un TTY. Si escribe archivos desde la CLI, monte un directorio escribible por UID 10001 y use `--output` dentro de él.

## Kubernetes

El [manifiesto base de Kubernetes](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml) proporciona valores no secretos mediante un ConfigMap y credenciales mediante un Secret existente llamado `mcp-gmap-rs`. Cree ese Secret desde un archivo local con `GOOGLE_MAPS_API_KEY` y `MCP_HTTP_TOKEN`, sin guardar el archivo en Git:

```sh
kubectl create secret generic mcp-gmap-rs --from-env-file=/secure/credentials.env
```

Sustituya `mcp-gmap-rs:local` en el Deployment por su imagen del registro, o cargue la imagen local en el clúster de desarrollo. Aplique después el ejemplo:

```sh
kubectl apply -f deploy/kubernetes.yaml
kubectl rollout status deployment/mcp-gmap-rs
kubectl port-forward service/mcp-gmap-rs 3000:3000
```

El ejemplo usa el namespace `default`. Adapte los nombres DNS de servicios al usar otro namespace o un ingress. El Service ClusterIP es privado al clúster; use un ingress o proxy TLS para clientes remotos y reenvíe un Host permitido en `MCP_HTTP_ALLOWED_HOSTS`, la autorización y los encabezados MCP/SSE. Añada orígenes de navegador explícitos, con puertos, cuando haga falta.

El Deployment usa `--no-env-file` porque toda la configuración se inyecta con `envFrom`. Como alternativa, monte un Secret que contenga un archivo dotenv, quite esa opción y establezca `args: ["--env-file", "/etc/mcp-gmap/runtime.env", "serve", "http"]`. El archivo proyectado debe ser legible por UID 10001; las variables de Secrets o ConfigMaps siguen teniendo prioridad. Consulte las guías de Kubernetes sobre [inyección de entorno](https://kubernetes.io/docs/tasks/inject-data-application/define-environment-variable-container/) y [volúmenes Secret](https://kubernetes.io/docs/concepts/configuration/secret/).

La configuración se lee al iniciar. Reinicie los pods después de cambiar esos valores de Secret/ConfigMap; los archivos TLS nativos también admiten recarga SIGHUP. Las sondas usan `/health` para la disponibilidad HTTP: un pod sano no demuestra que una API de Google esté habilitada o autorizada. Las llamadas Google requieren HTTPS saliente y DNS funcional. Las restricciones de claves API deben permitir la IP pública de salida del clúster; OAuth necesita acceso a sus endpoints de token e identidad.

Mantenga una réplica con el almacén de sesiones MCP en memoria predeterminado. Varias réplicas necesitan afinidad de sesión para alcanzar el pod propietario; los reinicios progresivos descartan sesiones y los clientes deben reconectar. Las solicitudes y límites de recursos del ejemplo son valores iniciales que debe ajustar a su carga.

SIGTERM inicia el cierre ordenado de MCP/HTTP; Kubernetes concede 30 segundos en el ejemplo. La imagen elimina capacidades y no permite escalada de privilegios.

## HTTPS, métricas y límites en producción

La [guía de producción](./production) cubre TLS/mTLS nativo, recarga de certificados, el Gateway HTTPS opcional `deploy/gateway.yaml`, métricas, presupuestos de solicitudes, límites HTTP, registros JSON y comprobaciones `doctor` sin conexión o en vivo. Pase las mismas opciones TLS tras el nombre de imagen o inyecte `MCP_TLS_CERT`, `MCP_TLS_KEY` y, opcionalmente, `MCP_TLS_CLIENT_CA` con montajes PEM de solo lectura. Mantenga las claves reales fuera de la imagen. El ConfigMap predeterminado fija límites y registro JSON; las métricas siguen desactivadas hasta habilitarlas explícitamente. El Service HTTP sin cifrar está pensado para el salto privado entre gateway y pod.

## Credenciales OAuth en un contenedor

Para un JSON de cuenta de servicio, móntelo en modo de solo lectura y configure la ruta interna del contenedor, no la del host:

```sh
docker run --rm --read-only --env-file /secure/oauth.env \
  --mount type=bind,src=/secure/service-account.json,dst=/credentials/service-account.json,readonly \
  -p 127.0.0.1:3000:3000 mcp-gmap-rs:local
```

El archivo de entorno OAuth define `GOOGLE_MAPS_AUTH_MODE=oauth`, `GOOGLE_APPLICATION_CREDENTIALS=/credentials/service-account.json`, `GOOGLE_CLOUD_QUOTA_PROJECT=your-project-id` y `MCP_HTTP_TOKEN`. UID 10001 debe poder leer el archivo montado. Conserve una clave API solo para herramientas cuyos endpoints la requieran. En GKE, use [las instrucciones de identidad de cargas de trabajo](./authentication#c-kubernetes-and-gke-workload-identity) en lugar de montar una clave de larga duración.
