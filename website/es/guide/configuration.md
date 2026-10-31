# Configuración

La CLI y ambos transportes MCP comparten la misma configuración de inicio. Use un archivo para trabajar en local, variables de entorno en contenedores y opciones para sobrescrituras explícitas por proceso.

## Precedencia

Los valores se aplican en este orden; las capas posteriores tienen prioridad:

**Valores integrados → archivo dotenv opcional → entorno del proceso → opciones CLI**

El archivo predeterminado es `.env` en el directorio de trabajo actual. No se buscan directorios superiores. Se permite que falte el archivo predeterminado; un archivo ilegible o mal formado impide el inicio sin imprimir su contenido.

```sh
mcp-gmap-rs --env-file /etc/mcp-gmap/runtime.env serve http
mcp-gmap-rs --no-env-file tools
```

`MCP_ENV_FILE` también selecciona un archivo, aunque `--env-file` tiene prioridad. Un archivo seleccionado explícitamente debe existir. `--no-env-file` desactiva la carga, incluido `MCP_ENV_FILE`, y es incompatible con `--env-file`. El análisis dotenv no modifica el entorno del proceso.

## Sobrescribir una configuración montada

```dotenv
GOOGLE_MAPS_LANGUAGE=en
GOOGLE_MAPS_TIMEOUT=30
GOOGLE_MAPS_ENABLED_TOOLS=maps_search_places,maps_place_details
```

Con ese archivo montado, una variable de entorno cambia el idioma y una opción cambia el tiempo de espera:

```sh
GOOGLE_MAPS_LANGUAGE=fr mcp-gmap-rs \
  --env-file /etc/mcp-gmap/runtime.env --timeout 15 tools
```

Las opciones repetidas `--allowed-host` y `--allowed-origin` reemplazan por completo sus listas del archivo o del entorno. Un valor de entorno explícitamente vacío también prevalece sobre el archivo: puede borrar un token opcional o una lista, mientras que un tiempo de espera vacío no es válido.

## Limitar las herramientas disponibles

`GOOGLE_MAPS_ENABLED_TOOLS` o `--enabled-tools` acepta nombres lógicos separados por comas, como `maps_weather,maps_air_quality`, o `*` para todas las herramientas. Un nombre desconocido impide el inicio. Las herramientas desactivadas desaparecen del descubrimiento y no se pueden llamar. El descubrimiento y la consulta del esquema no requieren credenciales, pero cada operación API necesita las credenciales de Google adecuadas.

La configuración de la aplicación se lee al iniciar. Reinicie después de cambiar valores de entorno, JSON de credenciales o la configuración de Secret/ConfigMap de Kubernetes. Los archivos de certificado, clave y CA de clientes para TLS nativo son la excepción: admiten [recarga mediante SIGHUP](./production#native-tls-and-mutual-tls).

Consulte la [referencia completa de configuración](../reference/configuration) para conocer los 21 ajustes, valores predeterminados, opciones y rangos. La selección de idioma del sitio no cambia `GOOGLE_MAPS_LANGUAGE`, que controla el texto de las respuestas de Google.
