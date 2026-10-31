# Agent skill

The portable `mcp-gmap` skill teaches an agent how to choose tools, call MCP or CLI safely, inspect partial failures and retain Google attribution. It supplements the executable or MCP connection you configure separately.

## Install from source or a release

The source package and every release archive include [the skill folder](https://github.com/fcenedes/mcp-gmap-rs/tree/main/skills/mcp-gmap). From a checkout or extracted archive, install it for Codex:

```sh
mkdir -p "${CODEX_HOME:-$HOME/.codex}/skills"
cp -R skills/mcp-gmap "${CODEX_HOME:-$HOME/.codex}/skills/"
```

Invoke it as `$mcp-gmap`, for example:

> Use $mcp-gmap to find cafes near Gare de Lyon, compare walking routes and preserve source links.

For other agents, place the same folder in their supported skill directory, or load `SKILL.md` and its linked references. The instruction body is agent-independent; `agents/openai.yaml` provides optional Codex UI metadata. Install a working executable on the agent’s PATH or configure its [MCP connection](./mcp).

## What the skill adds

The skill starts with tool discovery, so it does not assume that an older installed release exposes the newest catalog. It prefers exact coordinates and returned place IDs, keeps search scopes small, and chooses dedicated tools for address validation, aggregate counts or along-route search instead of approximating those operations from unrelated results.

It treats credentials as process/deployment configuration, never tool arguments. It distinguishes inbound MCP authorization from outbound Google access and uses offline diagnostics first. Live diagnostic probes are potentially billable. Composite output must be checked for nested errors, even when the top-level call succeeds.

The [tool-selection reference](https://github.com/fcenedes/mcp-gmap-rs/blob/main/skills/mcp-gmap/references/tool-selection.md) explains the decision rules. The [CLI reference](https://github.com/fcenedes/mcp-gmap-rs/blob/main/skills/mcp-gmap/references/cli.md) covers structured invocation and access diagnostics. See this manual’s [recipes](./recipes) for complete workflows and [tool reference](../reference/tools) for every argument.
