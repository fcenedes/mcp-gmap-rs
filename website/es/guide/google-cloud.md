# Configuración de Google Cloud

Este ejecutable autentica las **llamadas salientes a Google** con una clave API o credenciales OAuth 2.0. `MCP_HTTP_TOKEN` autentica de forma independiente a los clientes del servidor MCP HTTP; elegir Google OAuth no lo sustituye.

## Habilitar servicios y facturación

En la [consola de Google Cloud](https://console.cloud.google.com/), seleccione o cree un proyecto y vincule una cuenta de facturación activa en **Billing**. Habilite las APIs necesarias en **APIs & Services → Library**. Habilitar una API y permitirla en las restricciones de una clave API son pasos distintos. Las llamadas pueden generar cargos.

La tabla siguiente describe los endpoints utilizados por esta versión del proyecto:

| API de Google | Servicio que habilitar | Autenticación usada aquí |
| --- | --- | --- |
| [Places API (New)](https://developers.google.com/maps/documentation/places/web-service/get-api-key) | `places.googleapis.com` | Clave API u OAuth |
| [Routes API](https://developers.google.com/maps/documentation/routes/get-api-key) | `routes.googleapis.com` | Clave API u OAuth |
| [Weather API](https://developers.google.com/maps/documentation/weather/get-api-key) | `weather.googleapis.com` | Clave API u OAuth |
| [Air Quality API](https://developers.google.com/maps/documentation/air-quality/get-api-key) | `airquality.googleapis.com` | Clave API u OAuth |
| [Address Validation API](https://developers.google.com/maps/documentation/address-validation/get-api-key) | `addressvalidation.googleapis.com` | Clave API u OAuth |
| [Places Aggregate API](https://developers.google.com/maps/documentation/places-aggregate/get-api-key) | `areainsights.googleapis.com` | Clave API u OAuth |
| [Pollen API](https://developers.google.com/maps/documentation/pollen/get-api-key) | `pollen.googleapis.com` | Clave API |
| [Roads API](https://developers.google.com/maps/documentation/roads/get-api-key) | `roads.googleapis.com` | Clave API |
| [Geocoding API v3](https://developers.google.com/maps/documentation/geocoding/guides-v3/get-api-key) | `geocoding-backend.googleapis.com` | Clave API |
| [Elevation API](https://developers.google.com/maps/documentation/elevation/get-api-key) | `elevation-backend.googleapis.com` | Clave API |
| [Time Zone API](https://developers.google.com/maps/documentation/timezone/get-api-key) | `timezone-backend.googleapis.com` | Clave API |
| [Maps Static API](https://developers.google.com/maps/documentation/maps-static/get-api-key) | `static-maps-backend.googleapis.com` | Clave API |

Para habilitar los doce servicios con una sesión administrativa de `gcloud`:

```sh
export PROJECT_ID='your-project-id'
gcloud services enable --project="$PROJECT_ID" \
  places.googleapis.com routes.googleapis.com \
  weather.googleapis.com airquality.googleapis.com \
  geocoding-backend.googleapis.com elevation-backend.googleapis.com \
  timezone-backend.googleapis.com static-maps-backend.googleapis.com \
  addressvalidation.googleapis.com areainsights.googleapis.com \
  pollen.googleapis.com roads.googleapis.com
```

Las herramientas `directions` y `distance-matrix` de la CLI usan **Routes API**; las APIs antiguas Directions y Distance Matrix no son necesarias. Este proyecto sigue usando Geocoding v3; [la compatibilidad OAuth de Geocoding v4 de Google](https://developers.google.com/maps/documentation/geocoding/geocoding-v4-overview) no cambia la autenticación de estas llamadas v3. Las herramientas compuestas pueden llamar a varias APIs. La resolución de lugares por nombre o sus alternativas puede necesitar Geocoding aunque la operación principal use Places o Routes. Use coordenadas para evitar esa dependencia, o conserve una clave restringida para los endpoints que solo admiten claves.

## Crear una clave API restringida

1. Abra [Google Maps Platform → Keys & Credentials](https://console.cloud.google.com/google/maps-apis/credentials) en el proyecto seleccionado y elija **Create credentials → API key**.
2. Seleccione solo las APIs de la tabla que vaya a usar esta instancia. Para una clave existente, edite las restricciones API y guárdelas.
3. Configure la restricción de aplicación como **IP addresses** para esta aplicación de servidor. Permita las direcciones IP públicas de salida que verá Google. Las IP privadas o de pods Kubernetes normalmente no son la dirección pública de origen; una Cloud NAT fija u otra dirección de salida hace predecible esta restricción.
4. Guarde la clave en un `.env` ignorado, mediante inyección de entorno o en un Secret de Kubernetes. Consulte [las recomendaciones de Google sobre restricciones de claves](https://developers.google.com/maps/api-security-best-practices#restrict-your-api-keys).

Ejemplo de `.env`:

```dotenv
GOOGLE_MAPS_AUTH_MODE=api-key
GOOGLE_MAPS_API_KEY=your-restricted-api-key
```

`api-key` es el modo predeterminado. Proporcionar solo una ruta de cuenta de servicio no cambia el modo de autenticación. El descubrimiento no contacta con Google; use una llamada real para verificar la facturación, la activación de APIs y las restricciones:

```sh
mcp-gmap-rs geocode --address 'Paris, France'
```

## Pasos siguientes

Para JSON de cuenta de servicio, ADC, suplantación local e identidad de GKE, siga [Autenticación](./authentication). Después, elija [CLI](./cli) o [MCP](./mcp). La [referencia de herramientas](../reference/tools) indica los requisitos de credenciales de cada herramienta.
