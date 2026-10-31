# Desarrollo y releases

La aplicación es un proyecto Rust autónomo. CLI, MCP stdio y Streamable HTTP comparten las implementaciones de herramientas. El sitio documental es una herramienta de autoría y compilación independiente; el usuario final solo necesita el ejecutable compilado.

## Compilar y comprobar

Clone [el repositorio](https://github.com/fcenedes/mcp-gmap-rs) y use su archivo de bloqueo:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
cargo test --doc --all-features --locked
cargo build --release --locked
```

Rust 1.88 es el mínimo declarado; los binarios de release usan Rust 1.94.1. Las pruebas predeterminadas usan fixtures locales Google/token y no requieren credenciales ni conexión a Google. CI también comprueba Linux/macOS, Rust mínimo, rustdoc estable/nightly, empaquetado fuente, escenarios Docker/MCP/TLS y manifiestos Kubernetes.

Después de verificar en local, `cargo clean` libera los resultados de compilación; las compilaciones siguientes los reconstruyen.

## Mantener completa la referencia

La guía canónica combina los esquemas reales del ejecutable con valores predeterminados, resultados y notas de comportamiento revisados:

```sh
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs
python3 .github/scripts/tool-reference.py --binary /path/to/mcp-gmap-rs --check --verify-examples
```

La comprobación elimina credenciales, desactiva dotenv y valida todos los ejemplos hasta el límite de clave ausente sin contactar con Google. Actualice las notas JSON revisadas cuando cambien parámetros o comportamiento.

Las pruebas en vivo son opcionales y potencialmente facturables. Con una clave de prueba configurada explícitamente, ejecute `cargo test --locked --test live live_maps_ -- --ignored --test-threads=1`. Las seis pruebas OAuth usan el filtro `live_oauth_` y eliminan la clave en los procesos hijos. Nunca guarde credenciales ni active pruebas en vivo en CI ordinaria.

## Publicar una versión

Actualice conjuntamente versión Cargo, archivo de bloqueo, changelog, ejemplos de instalación y notas de release. Haga push y espere a CI; después cree una etiqueta anotada `vX.Y.Z` coincidente. El workflow Release comprueba esa etiqueta exacta, compila tres destinos nativos, verifica archivos y sumas, y publica el borrador preparado.

El workflow independiente **Publish crate** requiere ese release GitHub publicado. Ejecútelo para la misma etiqueta. Guarde su `CARGO_REGISTRY_TOKEN` de alcance limitado en el entorno GitHub `crates-io`, nunca en este repositorio. docs.rs compila automáticamente después de publicar en crates.io; un release GitHub por sí solo no actualiza la documentación Rust alojada. Verifique la compilación alojada real tras cargar el paquete.

Consulte [los procedimientos de release](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/releases.md), [el registro de validación](https://github.com/fcenedes/mcp-gmap-rs/blob/main/docs/validation.md) y [el punto de entrada público Rust](https://docs.rs/mcp-gmap-rs/latest/mcp_gmap_rs/fn.run.html).
