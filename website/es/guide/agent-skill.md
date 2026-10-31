# Skill para agentes

El skill portátil `mcp-gmap` enseña al agente a elegir herramientas, llamar con seguridad a MCP o CLI, inspeccionar fallos parciales y conservar la atribución Google. Complementa el ejecutable o la conexión MCP que se configura por separado.

## Instalar desde el código fuente o un release

El paquete fuente y cada archivo de release incluyen [la carpeta del skill](https://github.com/fcenedes/mcp-gmap-rs/tree/main/skills/mcp-gmap). Desde un repositorio o archivo extraído, instálelo para Codex:

```sh
mkdir -p "${CODEX_HOME:-$HOME/.codex}/skills"
cp -R skills/mcp-gmap "${CODEX_HOME:-$HOME/.codex}/skills/"
```

Invóquelo como `$mcp-gmap`, por ejemplo:

> Usa $mcp-gmap para encontrar cafés cerca de Gare de Lyon, comparar rutas a pie y conservar los enlaces de origen.

Con otros agentes, coloque la misma carpeta en su directorio de skills compatible, o cargue `SKILL.md` y sus referencias enlazadas. El cuerpo de instrucciones es independiente del agente; `agents/openai.yaml` proporciona metadatos opcionales para la interfaz Codex. Instale un ejecutable funcional en el PATH del agente o configure su [conexión MCP](./mcp).

## Qué aporta el skill

El skill empieza descubriendo herramientas, de modo que no supone que una versión antigua incluya el catálogo más reciente. Prefiere coordenadas exactas e IDs devueltos, mantiene los ámbitos de búsqueda pequeños y elige herramientas específicas para validar direcciones, contar agregados o buscar a lo largo de rutas, sin aproximarlas con resultados ajenos.

Trata las credenciales como configuración del proceso o despliegue, nunca como argumentos. Distingue autorización MCP entrante de acceso Google saliente y empieza con diagnósticos sin conexión. Las sondas en vivo pueden generar cargos. Deben comprobarse los errores anidados de resultados compuestos incluso cuando la llamada global es correcta.

La [referencia de selección de herramientas](https://github.com/fcenedes/mcp-gmap-rs/blob/main/skills/mcp-gmap/references/tool-selection.md) explica las reglas. La [referencia CLI](https://github.com/fcenedes/mcp-gmap-rs/blob/main/skills/mcp-gmap/references/cli.md) cubre invocación estructurada y diagnósticos. Consulte las [recetas](./recipes) para flujos completos y la [referencia de herramientas](../reference/tools) para todos los argumentos.
