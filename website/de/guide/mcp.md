# MCP-Integration

Verbinden Sie einen MCP-Host über stdio mit einem lokalen Kindprozess oder über Streamable HTTP mit einem dauerhaft laufenden Dienst. Beide Varianten stellen denselben aktivierten Werkzeugkatalog mit denselben Schemas bereit.

## Lokales stdio

Eine allgemeine MCP-Clientkonfiguration sieht so aus:

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

Verwenden Sie das Konfigurationsformat Ihres Hosts und einen absoluten Programmpfad. Die gewählte Dotenv-Datei muss existieren. Alternativ kann der Host Umgebungsvariablen bereitstellen. Dies ist eine eigenständige MCP-Serverkonfiguration; ein assistentenspezifisches Laufzeit-Plugin ist nicht erforderlich.

```sh
mcp-gmap-rs serve stdio
```

Standardeingabe und Standardausgabe transportieren ausschließlich zeilengetrennte MCP-JSON-RPC-Nachrichten. Diagnosen gehen an stderr. Ohne Argumente startet derselbe Transport. Ein containerbasierter stdio-Host muss stdin ohne TTY anbinden.

## Streamable HTTP

```sh
mcp-gmap-rs serve http --listen 127.0.0.1:3000
```

Setzen Sie die Streamable-HTTP-Adresse Ihres MCP-Clients auf **`http://127.0.0.1:3000/mcp`**. Unterstützt Ihr Host entfernte MCP-Einträge, konfigurieren Sie Endpunkt und Autorisierungsheader:

```json
{
  "url": "https://maps.example.com/mcp",
  "headers": {
    "Authorization": "Bearer your-mcp-http-token"
  }
}
```

Die genaue äußere Struktur dieses Eintrags hängt vom MCP-Host ab. Der Server unterstützt Protokollaushandlung und die zugehörigen POST-, GET/SSE- und DELETE-Sitzungsoperationen über das Rust-MCP-SDK. Verwenden Sie einen MCP-Client; ein einzelner isolierter JSON-RPC-POST ist keine vollständige Sitzung.

## Bindeadressen und Zugriff {#bind-addresses-and-access}

Listener außerhalb von Loopback benötigen `MCP_HTTP_TOKEN`. Clients senden dieses Token als Bearer-Zugangsdaten; es ist von Google OAuth und API-Schlüsseln getrennt. Tragen Sie den Host-Wert des Endpunkts und tatsächlich verwendete Browser-Origins ausdrücklich ein:

```sh
mcp-gmap-rs serve http --listen 0.0.0.0:3000 \
  --allowed-host maps.example.com \
  --allowed-origin https://app.example.com:443
```

Vorhandene Origin-Header werden standardmäßig abgewiesen, sofern sie nicht zugelassen wurden. Anfragekörper sind auf 12 MiB begrenzt. `/health` prüft die Prozesslebendigkeit ohne Google-Aufruf. Für öffentlichen Zugriff konfigurieren Sie [natives TLS/mTLS oder ein Gateway](./production). Proxys müssen Autorisierung sowie MCP-Protokoll- und Sitzungsheader erhalten und ungepuffertes Streaming erlauben. Sitzungen und Google-Identität sind prozesslokal; lesen Sie [Bereitstellung](./deployment), bevor Sie Replikate hinzufügen.
