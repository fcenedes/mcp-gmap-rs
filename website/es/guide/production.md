# Operación en producción

Esta guía cubre v0.3.x. Todas las herramientas Google comparten implementación entre CLI, MCP stdio y Streamable HTTP.

## TLS nativo y TLS mutuo {#native-tls-and-mutual-tls}

El listener HTTP puede servir HTTPS directamente mediante rustls y TLS 1.2/1.3:

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3443 \
  --tls-cert /etc/mcp-gmap/tls/tls.crt \
  --tls-key /etc/mcp-gmap/tls/tls.key \
  --allowed-host maps.example.com:3443
```

Proporcione `MCP_HTTP_TOKEN` por separado mediante el entorno o un Secret. Los clientes se conectan a `https://maps.example.com:3443/mcp`, verifican el certificado del servidor y envían `Authorization: Bearer <token>`. Los archivos contienen el certificado final PEM seguido de su cadena intermedia; la clave privada debe corresponder. El ejecutable no emite certificados ni implementa ACME. Use su PKI, cert-manager o un gateway TLS para emitirlos y renovarlos.

Añada `--tls-client-ca /etc/mcp-gmap/tls/client-ca.pem` para exigir un certificado de cliente firmado por esa CA. El mTLS nativo se aplica a todas las rutas, incluidas `/health` y `/metrics`, y no sustituye al token bearer HTTP. La capa TLS acepta cualquier identidad válida bajo las raíces de confianza configuradas; no hay permisos de herramientas por certificado ni comprobaciones de listas de revocación.

En Linux/macOS, sustituya los archivos de forma atómica y envíe `SIGHUP` para recargar juntos certificado, clave y CA opcional de clientes. Las conexiones existentes conservan su estado TLS; los nuevos handshakes usan los archivos nuevos. Si falla la carga, se mantiene la configuración anterior y se registra un error sin imprimir material de clave. Los archivos Secret montados de Kubernetes pueden recargarse así tras actualizarse su proyección. Las demás opciones siguen requiriendo un reinicio.

Para el UID/GID 10001 del contenedor, monte las claves en modo de solo lectura y con permisos que permitan leerlas; use un grupo apropiado y `fsGroup` para Secrets proyectados. Con TLS nativo, las sondas HTTP de Kubernetes deben usar `scheme: HTTPS`. Las sondas HTTP de kubelet no pueden presentar un certificado mTLS de cliente: use una sonda TCP en ese despliegue, entendiendo que comprueba el listener, no TLS ni Google.

## Gateway TLS de Kubernetes

El [manifiesto Gateway](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/gateway.yaml) proporciona un Gateway HTTPS y HTTPRoute para el [Deployment base](https://github.com/fcenedes/mcp-gmap-rs/blob/main/deploy/kubernetes.yaml). Instale primero los CRDs de Gateway API y un controlador conforme. Sustituya `your-gateway-class` y `maps.example.com`, y cree un Secret `kubernetes.io/tls` llamado `mcp-gmap-tls` en el mismo namespace. El ejemplo utiliza campos Gateway/HTTPRoute v1 estables y tiempos de espera HTTPRoute admitidos desde Gateway API v1.2. Compruebe el soporte de su controlador.

```sh
kubectl create secret tls mcp-gmap-tls --cert=/secure/fullchain.pem --key=/secure/key.pem
kubectl set env deployment/mcp-gmap-rs \
  MCP_HTTP_ALLOWED_HOSTS='maps.example.com,mcp-gmap-rs:3000,localhost:3000,127.0.0.1:3000'
kubectl apply -f deploy/gateway.yaml
kubectl get gateway,httproute
```

Guarde también la lista de hosts correspondiente en la ConfigMap o en la fuente del Deployment. Añada los orígenes reales del navegador con `MCP_HTTP_ALLOWED_ORIGINS` si hace falta. La ruta expone solo `/mcp`; salud y métricas permanecen en el Service privado. La autorización y los encabezados de sesión/protocolo MCP deben pasar sin cambios. Desactive buffering y caché de respuestas, y configure tiempos de streaming e inactividad en el controlador. La ruta desactiva las fechas límite de solicitud y de solicitud al backend; configure límites de conexión e inactividad apropiados para sus clientes.

Esta configuración termina TLS en el gateway y usa HTTP hacia el backend ClusterIP. Si también debe cifrarse el salto gateway-pod, use TLS nativo y una BackendTLSPolicy admitida por el controlador, o una malla. Restrinja el acceso directo a pods/Services mediante políticas de red del clúster. TLS no proporciona un inicio de sesión MCP OAuth: el ejecutable usa un token bearer compartido y una identidad Google por proceso.

Consulte [la guía TLS de Gateway](https://gateway-api.sigs.k8s.io/guides/user-guides/tls/) y [la guía de tiempos de espera HTTPRoute](https://gateway-api.sigs.k8s.io/guides/http-timeouts/).

## Límites, reintentos y plazos {#limits-retries-and-deadlines}

| Variable | Predeterminado | Opción | Significado |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | Intentos HTTP Google por llamada de herramienta, 1–10000; incluye reintentos |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | Intentos adicionales por solicitud Google, 0–5 |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | Plazo total de herramienta en segundos, 1–3600 |
| `MCP_HTTP_RATE_LIMIT` | `120` | HTTP `--rate-limit` | Solicitudes HTTP autenticadas a `/mcp` por minuto y proceso; 0 desactiva |
| `MCP_METRICS_ENABLED` | `false` | HTTP `--metrics [true\|false]` | Habilitar Prometheus protegido en `/metrics` |
| `MCP_LOG_FORMAT` | `text` | `--log-format text\|json` | Formato de diagnósticos en stderr |
| `MCP_TLS_CERT` | sin definir | HTTP `--tls-cert` | Archivo PEM de cadena de certificados del servidor |
| `MCP_TLS_KEY` | sin definir | HTTP `--tls-key` | Archivo PEM de clave privada correspondiente |
| `MCP_TLS_CLIENT_CA` | sin definir | HTTP `--tls-client-ca` | Paquete CA PEM; exige certificados de cliente |

Estas opciones siguen valores predeterminados → dotenv opcional → entorno → opciones. Certificado y clave TLS deben proporcionarse juntos. Las rutas TLS vacías en el entorno borran sus valores de archivo. `GOOGLE_MAPS_TIMEOUT` limita cada solicitud HTTP Google, incluida la espera del semáforo, reintentos, backoff y lectura de respuesta; la obtención OAuth tiene un límite propio con el mismo valor. El plazo total de la herramienta incluye ambos.

Cada llamada recibe un presupuesto nuevo compartido entre sus ramas concurrentes. Pueden ejecutarse cinco solicitudes Google por proceso a la vez. Al agotarse el presupuesto, los intentos siguientes fallan localmente. Las herramientas compuestas pueden devolver una respuesta global correcta con errores de presupuesto por elemento: revíselos. Se limita el número de solicitudes, no el gasto monetario; distintas solicitudes y campos tienen precios diferentes y una matriz puede facturar varios elementos.

Las solicitudes Google de solo lectura reintentan HTTP 429, 500, 502, 503 y 504, además de fallos de red. Respetan `Retry-After` en segundos o como fecha HTTP; si falta, usan espera exponencial con variación aleatoria. Permanecen dentro del plazo de solicitud y del presupuesto de llamada. Otros errores HTTP, incluidos 400/401/403, no se reintentan. Ningún fallo de autenticación cambia silenciosamente al modo de clave API. Configure cero reintentos para un único intento inicial. Los estados de error JSON internos de APIs antiguas de Google se informan sin reintentos.

El limitador HTTP es un depósito de tokens con capacidad de ráfaga igual a la tasa por minuto y recarga continua. Cubre inicialización, POST, GET y DELETE en `/mcp` para todas las sesiones. Las solicitudes no autorizadas no consumen tokens. Los rechazos devuelven HTTP 429 con `Retry-After`; salud y lectura de métricas están exentas. El límite es global a un proceso, no por usuario/IP, y no confía en encabezados IP reenviados. Use autenticación de gateway y límites por cliente en despliegues compartidos.

## Métricas y eventos de diagnóstico

```sh
mcp-gmap-rs --log-format json serve http --metrics --rate-limit 120
```

`GET /metrics` utiliza el mismo token bearer que `/mcp` si está configurado. En un listener loopback sin autenticación, tampoco requiere autenticación. Está deshabilitado por defecto y se le aplica mTLS nativo si está activo. Expóngalo solo a recolectores de confianza. El endpoint de texto Prometheus exporta:

- `mcp_gmap_tool_calls_total{tool,outcome}` para llamadas terminadas;
- `mcp_gmap_tool_duration_seconds{tool,outcome}` como histograma;
- `mcp_gmap_google_requests_total{api,status}` para intentos HTTP Google, incluidos reintentos;
- `mcp_gmap_http_rate_limited_total` para solicitudes HTTP rechazadas.

Las etiquetas contienen nombres integrados de herramientas, hosts API, códigos HTTP y resultados fijos. Nunca contienen direcciones, coordenadas, credenciales, tokens ni IDs de sesión. Una llamada compuesta con errores por elemento puede contar como éxito global; examine el resultado y las métricas de estado del proveedor. Las métricas se reinician al reiniciar el proceso.

Configure `RUST_LOG=info` para ver eventos de finalización con nombre, éxito y duración. No se incluyen argumentos ni cuerpos de respuesta Google. La dirección de inicio del listener se imprime aparte en stderr. Stdout queda reservado para JSON CLI o tráfico MCP.

## Doctor

```sh
mcp-gmap-rs doctor
# Explicit opt-in: makes potentially billable requests.
mcp-gmap-rs doctor --live
# Check only the API dependencies of selected tools:
mcp-gmap-rs --enabled-tools maps_pollen,maps_area_insights doctor --live
```

Sin `--live`, doctor comprueba la configuración local de credenciales, las opciones HTTP y la correspondencia de archivos y claves TLS sin solicitudes de red. No demuestra validez de claves, IAM, habilitación de APIs, confianza/nombre/caducidad de certificados ni acceso ADC. El descubrimiento ADC se marca `not_tested` hasta una llamada en vivo. Doctor nunca imprime valores de credenciales ni su JSON. El informe JSON va a stdout; un fallo produce código de salida 1. Analice `checks[].status`: `pass`, `fail` o `not_tested`.

El modo en vivo hace una llamada representativa por cada API Google distinta usada por las herramientas habilitadas, con los límites habituales de reintento, plazo y presupuesto. Evita compuestos costosos. Routes usa coordenadas exactas; Roads usa búsqueda de carretera más cercana. Las sondas utilizan lugares públicos de París, no direcciones del usuario. El éxito significa que Google aceptó esa sonda, no que estén disponibles todas las operaciones o regiones.

## Cierre y cambios de configuración

Ctrl-C y SIGTERM cierran el servidor HTTP y sus sesiones MCP activas. Kubernetes concede 30 segundos en el ejemplo. Otras opciones, incluidos cambios de Secret inyectados por entorno, requieren reinicio; solo los archivos nativos de certificado/clave/CA de clientes admiten recarga SIGHUP. Las sesiones y métricas están en memoria y se pierden al reiniciar. Use una réplica o afinidad de sesión; los clientes deben reconectar al salir el proceso propietario.

Consulte [despliegue](./deployment) para permisos, sondas y montajes de credenciales, y la [referencia de configuración](../reference/configuration) para cada ajuste y opción.
