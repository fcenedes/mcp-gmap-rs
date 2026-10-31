# Integración MCP

Conecte un host MCP mediante stdio a un proceso hijo local, o mediante Streamable HTTP a un servicio de larga duración. Ambos exponen el mismo catálogo habilitado y los mismos esquemas.

## Stdio local

Una configuración genérica de cliente MCP es:

```json
{
  "mcpServers": {
    "google-maps": {
      "command": "/absolute/path/to/mcp-gmap-rs",
      "args": ["--env-file", "/secure/mcp-gmap/runtime.env", "serve", "stdio"]
    }
  }
}
```

Use el formato de configuración del host y una ruta absoluta al ejecutable. El archivo dotenv seleccionado debe existir. También puede inyectar variables de entorno desde el host. Es una configuración de servidor MCP autónomo; no hace falta un plugin de ejecución específico de un asistente.

```sh
mcp-gmap-rs serve stdio
```

La entrada y la salida estándar transportan únicamente JSON-RPC de MCP delimitado por saltos de línea. Los diagnósticos van a stderr. Ejecutar sin argumentos inicia el mismo transporte. Un host stdio basado en contenedores debe adjuntar stdin sin asignar un TTY.

## Streamable HTTP

```sh
mcp-gmap-rs serve http --listen 127.0.0.1:3000
```

Configure la URL Streamable HTTP del cliente MCP como **`http://127.0.0.1:3000/mcp`**. Si el host admite entradas MCP remotas, configure el endpoint y el encabezado de autorización:

```json
{
  "url": "https://maps.example.com/mcp",
  "headers": {
    "Authorization": "Bearer your-mcp-http-token"
  }
}
```

La estructura que envuelve esta entrada depende del host MCP. El servidor admite negociación de protocolo y las operaciones de sesión POST, GET/SSE y DELETE correspondientes mediante el SDK MCP de Rust. Use un cliente MCP: un POST JSON-RPC aislado no constituye una sesión completa.

## Direcciones de escucha y acceso {#bind-addresses-and-access}

Los listeners fuera de loopback requieren `MCP_HTTP_TOKEN`. Los clientes envían ese token como credencial bearer; es independiente de Google OAuth y de las claves API. Añada explícitamente el valor Host del endpoint y los orígenes reales del navegador:

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3000 \
  --allowed-host maps.example.com \
  --allowed-origin https://app.example.com:443
```

De forma predeterminada, los encabezados Origin presentes se rechazan si no están permitidos. El cuerpo de la solicitud se limita a 1 MiB. `/health` comprueba que el proceso está vivo sin llamar a Google. Para el acceso público, configure [TLS/mTLS nativo o un gateway](./production). Los proxies deben conservar la autorización y los encabezados de protocolo/sesión MCP, y permitir streaming sin buffering. Las sesiones y la identidad de Google son locales a cada proceso; consulte [despliegue](./deployment) antes de añadir réplicas.
