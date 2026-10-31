# Interfaz de línea de comandos

Use la CLI para consultas puntuales, automatización de shell y llamadas estructuradas a subprocesos. Cada herramienta comparte su esquema de entrada e implementación con MCP.

## Descubrir la interfaz instalada

```sh
mcp-gmap-rs --version
mcp-gmap-rs --help
mcp-gmap-rs tools
mcp-gmap-rs schema search-nearby
mcp-gmap-rs search-nearby --help
```

`tools` muestra las herramientas habilitadas y sus esquemas. Compruebe la versión instalada en lugar de suponer que incluye todas las funciones del manual más reciente. Ejecutar sin subcomando inicia el servidor stdio; indique siempre un comando para automatizaciones de una sola ejecución.

## Opciones o JSON compartido

Los comandos usan opciones en formato kebab-case; los objetos y arrays se pasan como JSON. Las opciones booleanas aceptan `true` o `false`, y una opción sin valor significa `true`.

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now
```

La interfaz genérica `call` acepta un nombre lógico de herramienta MCP o un nombre de comando CLI y usa exactamente el objeto de argumentos MCP:

```sh
mcp-gmap-rs call maps_geocode --args '{"address":"Paris, France"}'
printf '%s' '{"address":"Paris, France"}' | mcp-gmap-rs call geocode --args -
```

Al iniciar un subproceso desde código, pase el JSON serializado como un argumento independiente. No inserte texto no confiable en cadenas de comandos de shell. Los parámetros de primer nivel tienen opciones; los campos anidados permanecen dentro de objetos JSON.

## Salida y gestión de errores

Los resultados normales se imprimen como JSON en stdout. Los errores van a stderr y producen un código de salida distinto de cero. `doctor` también escribe su informe JSON en stdout cuando falla alguna comprobación. Los resultados por lotes, matriciales y compuestos pueden contener errores por elemento aunque el comando termine correctamente: revíselos antes de dar la operación por completa.

```sh
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --output place.json
mcp-gmap-rs static-map --center Paris --zoom 12 --output paris.png
```

`--output` escribe JSON para las herramientas normales y PNG binario para un mapa estático. Sin esa opción, la salida CLI del mapa contiene datos Base64. Las horas suelen usar RFC 3339 con zona horaria explícita; el `timestamp` opcional de la herramienta Time Zone usa **milisegundos** Unix. Consulte [todas las herramientas y ejemplos](../reference/tools).
