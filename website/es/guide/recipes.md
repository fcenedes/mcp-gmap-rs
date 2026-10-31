# Recetas

Combine llamadas pequeñas y explícitas alrededor de una ubicación o una selección de IDs devueltos. Estos ejemplos realizan solicitudes Google potencialmente facturables. Revise primero [la autenticación](./authentication); los compuestos pueden usar varias APIs y devolver errores parciales.

## Encontrar un café y comparar el trayecto

Busque dentro de un radio real, amplíe solo los IDs seleccionados y compare los desplazamientos. `keyword` de Nearby es un tipo de lugar de Google, no texto libre.

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now --min-rating 4
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --max-photos 0
mcp-gmap-rs distance-matrix --origins '["48.8584,2.2945"]' \
  --destinations '["48.8606,2.3376","48.8530,2.3499"]' --mode walking
```

Use [`search-places`](../reference/tools#maps_search_places) para intención en lenguaje natural, [`compare-places`](../reference/tools#maps_compare_places) para coincidencias detalladas con distancia en coche opcional o [`explore-area`](../reference/tools#maps_explore_area) para explorar por categorías. Una página de búsqueda no es un inventario exhaustivo. Conserve enlaces de origen Google, autores de reseñas y avisos de IA.

## Planificar paradas y encontrar un desvío

```sh
mcp-gmap-rs plan-route \
  --stops '["48.8584,2.2945","48.8606,2.3376","48.8530,2.3499"]' \
  --mode walking --optimize false
mcp-gmap-rs search-along-route --text-query cafes \
  --origin '48.8584,2.2945' --destination '48.8606,2.3376' \
  --mode walking --max-results 3
```

[`directions`](../reference/tools#maps_directions) gestiona un viaje con opciones horarias y de evitación; [`plan-route`](../reference/tools#maps_plan_route) mantiene los extremos al optimizar paradas intermedias. Transit solo acepta dos paradas en total. La búsqueda a lo largo de una ruta sigue una polilínea, sin sustituirla por un círculo en el punto medio. Por defecto usa desplazamiento a pie; seleccione siempre el modo deseado.

## Resolver una vez y consultar condiciones locales

```sh
mcp-gmap-rs geocode --address 'Eiffel Tower, Paris, France'
mcp-gmap-rs weather --latitude 48.8584 --longitude 2.2945 \
  --type forecast_daily --forecast-days 3
mcp-gmap-rs air-quality --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs pollen --latitude 48.8584 --longitude 2.2945 --days 3
mcp-gmap-rs timezone --latitude 48.8584 --longitude 2.2945
mcp-gmap-rs elevation --locations '[{"latitude":48.8584,"longitude":2.2945}]'
```

Reutilice coordenadas en lugar de repetir la resolución. Distinga observaciones, previsiones y recomendaciones del proveedor; la cobertura puede variar. [`reverse-geocode`](../reference/tools#maps_reverse_geocode) convierte coordenadas en dirección; [`batch-geocode`](../reference/tools#maps_batch_geocode) resuelve hasta 50 direcciones con errores individuales. El timestamp opcional de Time Zone usa milisegundos Unix; la elevación se expresa en metros.

## Validar una dirección postal

```sh
mcp-gmap-rs validate-address --address-lines '["5 Avenue Anatole France"]' \
  --locality Paris --postal-code 75007 --region-code FR
```

Revise el veredicto, las correcciones y los componentes sin resolver. La validación postal es distinta de la geocodificación y no garantiza la entrega. USPS CASS es opcional y requiere `regionCode` US o PR en esta implementación.

## Contar lugares sin contar resultados de búsqueda

```sh
mcp-gmap-rs area-insights \
  --area '{"circle":{"latLng":{"latitude":48.8584,"longitude":2.2945},"radius":500}}' \
  --included-types '["restaurant"]' --min-rating 4
```

Los recuentos agregados suelen usar la representación int64 de Google como cadena. Los IDs opcionales solo están disponibles cuando el recuento es como máximo 100; no se solicitan detalles automáticamente. Para análisis de visibilidad solicitado explícitamente, [`local-rank-tracker`](../reference/tools#maps_local_rank_tracker) mide posiciones por palabra clave en una cuadrícula. Empiece con una palabra y una cuadrícula 3×3. Las posiciones no son valoraciones de reseñas y los puntos fallidos deben seguir visibles.

## Ajustar datos GPS y guardar un mapa

Use [`snap-to-roads`](../reference/tools#maps_snap_to_roads) para un recorrido ordenado y [`nearest-roads`](../reference/tools#maps_nearest_roads) para puntos independientes. Conserve `originalIndex`; la interpolación añade puntos que pueden no tenerlo. No se devuelven límites de velocidad.

```sh
mcp-gmap-rs nearest-roads --points '[{"latitude":48.8584,"longitude":2.2945}]'
mcp-gmap-rs static-map --center '48.8584,2.2945' --zoom 14 --output area.png
```

La [referencia de herramientas](../reference/tools) enumera límites exactos, campos obligatorios, valores predeterminados y formatos de resultado de cada operación.
