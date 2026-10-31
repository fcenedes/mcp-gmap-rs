# Resolución de problemas

Identifique primero la conexión que falla: configuración local, conexión del cliente al servidor MCP o solicitud saliente a Google. Corrija la capa correspondiente y repita la operación afectada.

## Empezar sin conexión

```sh
mcp-gmap-rs --version
mcp-gmap-rs tools
mcp-gmap-rs doctor
```

`doctor` imprime JSON incluso si alguna comprobación falla. Examine el código de salida y `checks[].status`: `pass`, `fail` o `not_tested`. Las comprobaciones sin conexión no demuestran IAM, habilitación de APIs, facturación, confianza/caducidad de certificados ni acceso ADC. `/health` solo demuestra que el proceso está vivo.

Si un dotenv local mal formado bloquea el descubrimiento, `--no-env-file tools` lo omite. Esto puede cambiar las herramientas seleccionadas; no lo confunda con una verificación de la configuración de archivo prevista.

## Interpretar fallos frecuentes

| Síntoma | Siguiente comprobación |
| --- | --- |
| `GOOGLE_MAPS_API_KEY is required` | Está seleccionado el modo de clave API o la operación usa un endpoint exclusivo de clave. |
| `GOOGLE_CLOUD_QUOTA_PROJECT is required` | Defina el ID o número del proyecto consumidor para OAuth. |
| Google 403 / API deshabilitada | Habilite la API exacta en el proyecto consumidor de las credenciales; revise facturación y restricciones de clave. |
| `serviceusage.services.use` denegado | Conceda `roles/serviceusage.serviceUsageConsumer` a la cuenta de servicio en el proyecto de cuota; espere la propagación IAM y reintente. |
| Clave API rechazada por IP | Permita la IP pública de salida del servidor o clúster, no la IP privada del pod. |
| MCP HTTP 401 | Envíe el token bearer del servidor; el token Google autentica una conexión distinta. |
| Host/Origin rechazado | Haga coincidir exactamente Host y origen del navegador, incluidos los puertos pertinentes. |
| HTTP 429 de MCP | Se agotó el limitador global; respete `Retry-After` y revise la concurrencia del cliente. |
| Tiempo de espera OAuth | Compruebe conexión a token/STS/IAM o metadatos, reloj y tiempo de espera por solicitud. |
| Fallo de conexión TLS | Revise cadena de certificados, hostname, confianza y certificado mTLS opcional por separado de la autorización HTTP. |
| Salida 0 con datos incompletos | Revise errores por elemento de matrices/compuestos, no solo el código de salida. |

## Verificar acceso en vivo deliberadamente

Las sondas en vivo pueden generar cargos. Limítelas a las APIs que necesite:

```sh
mcp-gmap-rs --enabled-tools maps_weather doctor --live
```

Una sonda correcta no garantiza todas las regiones u operaciones. Después de cambiar Google Cloud, repita la llamada que falló en lugar de deducir éxito desde la consola. Consulte [autenticación](./authentication) para IAM y credenciales, [configuración](./configuration) para precedencia y [operación en producción](./production) para presupuestos, plazos y métricas. Nunca imprima JSON de credenciales ni vuelque el entorno completo en un informe de incidencia.
