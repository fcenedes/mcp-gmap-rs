# Primeros pasos

Instale un ejecutable, configure el acceso a Google y use las mismas 23 herramientas desde el terminal o un cliente MCP. Este manual describe v0.3.1.

## Instalación

En [GitHub Releases](https://github.com/fcenedes/mcp-gmap-rs/releases) hay binarios precompilados para Linux x86_64, macOS Apple Silicon y macOS Intel. Descargue el archivo para su equipo, compruébelo con `SHA256SUMS`, extráigalo y añada `mcp-gmap-rs` a su `PATH`. Los binarios Linux requieren glibc 2.35 o posterior. Los binarios macOS no están firmados ni notarizados.

Con Rust 1.88 o posterior, instale desde crates.io:

```sh
cargo install mcp-gmap-rs --version 0.3.1 --locked
mcp-gmap-rs --version
```

Una vez compilado el ejecutable, no necesita un entorno de ejecución de Rust, Python o JavaScript. El equipo necesita certificados de CA, DNS y acceso HTTPS saliente a Google.

## Configurar el acceso

[Cree una clave restringida y habilite los servicios](./google-cloud), o configure [OAuth con una cuenta de servicio](./authentication). Guarde las credenciales en un archivo `.env` ignorado por Git dentro del directorio de trabajo:

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

Las variables de entorno prevalecen sobre el archivo. Mantenga las credenciales fuera del control de versiones y de las capas de imagen. El directorio de trabajo importa: no se buscan archivos `.env` en directorios superiores.

## Descubrir, comprobar y llamar

El descubrimiento y los diagnósticos sin conexión no realizan solicitudes a Google:

```sh
mcp-gmap-rs tools
mcp-gmap-rs schema search-places
mcp-gmap-rs doctor
```

Después, haga su primera llamada potencialmente facturable:

```sh
mcp-gmap-rs search-places --query 'coffee near Gare de Lyon, Paris'
```

El resultado es JSON. Conserve los identificadores de lugar devueltos para consultar detalles sin volver a resolver los mismos nombres. Consulte [el uso de la CLI](./cli) para automatizar o [la integración MCP](./mcp) para conectar un agente. Las [recetas](./recipes) muestran cómo combinar herramientas; la [referencia completa](../reference/tools) documenta cada parámetro, valor predeterminado y resultado.
