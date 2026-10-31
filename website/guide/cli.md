# Command-line interface

Use the CLI for one-shot queries, shell automation and structured subprocess calls. Every tool shares its input schema and implementation with MCP.

## Discover the installed interface

```sh
mcp-gmap-rs --version
mcp-gmap-rs --help
mcp-gmap-rs tools
mcp-gmap-rs schema search-nearby
mcp-gmap-rs search-nearby --help
```

`tools` reports the enabled tools and their schemas. Check your installed version rather than assuming that every capability in the newest manual is present. Running without a subcommand starts the stdio server, so always supply a command for one-shot automation.

## Flags or shared JSON

Tool commands use kebab-case flags; object and array values take JSON. Boolean flags accept `true` or `false`, and a bare boolean flag means `true`.

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now
```

The generic `call` interface accepts either a logical MCP tool name or CLI command name and uses the exact MCP argument object:

```sh
mcp-gmap-rs call maps_geocode --args '{"address":"Paris, France"}'
printf '%s' '{"address":"Paris, France"}' | mcp-gmap-rs call geocode --args -
```

When launching a subprocess programmatically, pass serialized JSON as a distinct argument. Avoid interpolating untrusted text into shell command strings. Top-level parameters have flags; nested fields stay inside JSON objects.

## Output and failure handling

Ordinary tool results are JSON on stdout. Errors go to stderr and produce a nonzero exit status. `doctor` also writes its JSON report to stdout when checks fail. Batch, matrix and composite results can contain per-item errors even when the command exits successfully: inspect those fields before treating the whole operation as complete.

```sh
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --output place.json
mcp-gmap-rs static-map --center Paris --zoom 12 --output paris.png
```

`--output` writes JSON for ordinary tools and raw PNG for a static map. Without it, CLI static-map output contains base64 data. Times generally use RFC 3339 with an explicit timezone; the Time Zone tool’s optional `timestamp` is Unix **milliseconds**. See [all tools and examples](../reference/tools).
