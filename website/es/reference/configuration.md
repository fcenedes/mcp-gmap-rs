# Referencia de configuración

Estos 21 ajustes se aplican según corresponda a CLI, MCP stdio y Streamable HTTP. La precedencia es **predeterminados → dotenv opcional → entorno → opciones**. Consulte [configuración](../guide/configuration) para selección de archivo y sobrescrituras.

Las opciones globales van antes de una herramienta o de `serve`; las exclusivas HTTP van después de `serve http`. Un guion en la columna de opciones significa que no hay una opción CLI correspondiente: inyecte ese valor mediante el entorno o el dotenv seleccionado.

## Acceso a Google y ejecución de herramientas

| Variable de entorno | Predeterminado | Opción CLI | Significado y valores aceptados |
| --- | --- | --- | --- |
| `GOOGLE_MAPS_API_KEY` | Sin definir | — | Clave API Google restringida. Obligatoria en modo `api-key` y para endpoints exclusivos de clave de esta implementación, incluso con OAuth seleccionado. Nunca es un argumento de herramienta. |
| `GOOGLE_MAPS_AUTH_MODE` | `api-key` | `--auth-mode` | `api-key` u `oauth`. OAuth admite Places (New), Routes, Weather, Air Quality, Address Validation y Places Aggregate. |
| `GOOGLE_APPLICATION_CREDENTIALS` | Descubrimiento ADC | — | Ruta a un JSON legible de cuenta de servicio o ADC compatible. Las rutas relativas parten del directorio de trabajo. Deje sin definir para ADC local o descubrimiento de metadatos. |
| `GOOGLE_CLOUD_QUOTA_PROJECT` | Sin definir | — | ID o número obligatorio del proyecto consumidor de facturación/cuota para OAuth. La cuenta de servicio necesita `serviceusage.services.use` en ese proyecto. |
| `GOOGLE_MAPS_LANGUAGE` | `en` | `--language` | Código de idioma de respuestas Google con soporte lingüístico. Independiente del selector de idioma de este manual. |
| `GOOGLE_MAPS_TIMEOUT` | `30` | `--timeout` | Tiempo de espera por solicitud en segundos, entero 1–600. Incluye cola, reintentos, backoff y lectura; la obtención OAuth tiene su propio límite con el mismo valor. |
| `GOOGLE_MAPS_ENABLED_TOOLS` | Todas (`*`) | `--enabled-tools` | Nombres `maps_*` separados por comas o `*`. Los nombres desconocidos impiden el inicio. Las herramientas desactivadas no se descubren ni pueden llamarse. |
| `GOOGLE_MAPS_MAX_REQUESTS` | `200` | `--max-requests` | Presupuesto de intentos HTTP Google por llamada, entero 1–10000. Compartido por ramas compuestas; los reintentos consumen intentos. |
| `GOOGLE_MAPS_MAX_RETRIES` | `2` | `--max-retries` | Intentos transitorios adicionales por solicitud Google, entero 0–5. Cero significa un único intento inicial. Los reintentos respetan plazo y presupuesto. |
| `GOOGLE_MAPS_TOOL_TIMEOUT` | `120` | `--tool-timeout` | Plazo total de herramienta en segundos, entero 1–3600. Incluye credenciales, cola, llamadas API y reintentos. |

Pueden ejecutarse cinco solicitudes HTTP Google a la vez por proceso. El presupuesto limita solicitudes, no dinero; una matriz puede facturar varios elementos. Algunos compuestos devuelven errores por elemento cuando una rama agota su presupuesto o plazo. Consulte [límites de producción](../guide/production#limits-retries-and-deadlines).

## Listener HTTP y acceso

| Variable de entorno | Predeterminado | Opción CLI | Significado y valores aceptados |
| --- | --- | --- | --- |
| `MCP_HTTP_TOKEN` | Sin definir | — | Token bearer compartido de clientes MCP. Obligatorio fuera de loopback. Independiente de credenciales Google y necesario también con mTLS nativo. |
| `MCP_HTTP_LISTEN` | `127.0.0.1:3000` | `--listen` | Dirección IP y puerto del listener. La imagen Docker fija el valor de entorno `0.0.0.0:3000`. |
| `MCP_HTTP_ALLOWED_HOSTS` | Hosts loopback | `--allowed-host` | Valores HTTP Host aceptados separados por comas. Repita la opción para varios valores; reemplaza la lista completa de archivo/entorno. Incluya el puerto pertinente. |
| `MCP_HTTP_ALLOWED_ORIGINS` | Ninguno | `--allowed-origin` | Orígenes de navegador permitidos separados por comas. Los encabezados Origin presentes se rechazan salvo permiso. Especifique puertos; las opciones repetidas reemplazan la lista. |
| `MCP_HTTP_RATE_LIMIT` | `120` | `--rate-limit` | Solicitudes autenticadas a `/mcp` por minuto y proceso, entero 0–1000000. Cero desactiva. Capacidad de ráfaga igual a la tasa; recarga continua. |
| `MCP_METRICS_ENABLED` | `false` | `--metrics [true\|false]` | Habilitar Prometheus en `/metrics`. `--metrics` sin valor significa true. Usa el mismo bearer si está configurado; también se aplica mTLS nativo. |
| `MCP_TLS_CERT` | Sin definir | `--tls-cert` | Certificado final PEM del servidor más cadena intermedia. Debe acompañarse de una clave privada correspondiente. |
| `MCP_TLS_KEY` | Sin definir | `--tls-key` | Ruta a la clave privada PEM del servidor. Debe ser legible por la identidad de ejecución y mantenerse fuera del código e imágenes. |
| `MCP_TLS_CLIENT_CA` | Sin definir | `--tls-client-ca` | Paquete CA PEM de clientes que exige certificados en todas las rutas. No sustituye al token bearer. |

El depósito de tokens HTTP es global al proceso, no por usuario/IP. Las solicitudes no autorizadas no lo consumen; salud y métricas están exentas. Los rechazos devuelven 429 con `Retry-After`. El cuerpo se limita a 12 MiB. Consulte [acceso MCP](../guide/mcp#bind-addresses-and-access) y [operación TLS](../guide/production#native-tls-and-mutual-tls).

## Eventos de diagnóstico

| Variable de entorno | Predeterminado | Opción CLI | Significado y valores aceptados |
| --- | --- | --- | --- |
| `RUST_LOG` | `warn` | — | Filtro de registro diagnóstico. `info` habilita eventos de finalización; no se registran argumentos ni cuerpos de respuestas Google. |
| `MCP_LOG_FORMAT` | `text` | `--log-format` | `text` o `json`. Los diagnósticos siempre van a stderr; stdout queda para JSON CLI o tráfico MCP. |

## Selección de archivo y control de salida

La selección de archivo precede a las capas de la aplicación. `MCP_ENV_FILE` elige una ruta dotenv; la opción global `--env-file PATH` prevalece. Un archivo seleccionado explícitamente debe existir. `--no-env-file` desactiva toda carga dotenv y es incompatible con `--env-file`. Sin selección, solo se lee el `.env` opcional del directorio de trabajo.

Un valor de entorno explícitamente vacío sigue sobrescribiendo el archivo. Puede borrar un token opcional, una ruta TLS o una lista permitida, pero no valida un ajuste numérico vacío. Los valores cargados no modifican el entorno del proceso.

`doctor --live` activa sondas potencialmente facturables; `doctor` normal trabaja sin conexión. La opción de herramienta `--output PATH` guarda JSON o PNG binario para `static-map`. Son controles del comando, no variables de entorno adicionales.

Los cambios de ajustes y archivos de credenciales requieren reinicio. En Linux/macOS, SIGHUP recarga conjuntamente archivos nativos de certificado/clave/CA de clientes; si falla, conserva la configuración TLS anterior. Consulte [configuración](../guide/configuration) y [operación en producción](../guide/production).
