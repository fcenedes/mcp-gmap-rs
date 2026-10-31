# Multilingual manual

This directory contains the VitePress manual and its GitHub Pages presentation.
The English, French and Italian manuals each include setup, authentication,
configuration, CLI/MCP, deployment, production operations, recipes, troubleshooting,
the agent skill, development, every configuration setting and all 23 tools.
Node.js is documentation tooling only; the distributed CLI/MCP executable remains
standalone Rust.

## Work locally

Use Node.js 24, npm and Python 3. From the repository root:

```sh
cd website
npm ci
npm run dev
```

The documentation dependencies are locked in `package-lock.json`. No Google API
key, service account, MCP token or network access to Google APIs is needed to
generate or build the site. Dependency installation still needs npm access.

Before committing:

```sh
npm run generate
npm run format:check
npm run build
npm run preview -- --host 127.0.0.1
```

The preview serves the built site under `/mcp-gmap-rs/`; use the address printed
by VitePress. `npm run build` first checks committed generated references, then
builds VitePress and runs the generated-site validator. A stale reference fails
the build instead of silently rewriting committed documentation.

Restart the preview after rebuilding: its static-file cache can still refer to
the previous build's hashed assets. Use `npm run dev` for continuous editing.

The lockfile pins VitePress 1.6.4 with a patched Vite 6.4.5 override. The production
build and browser preview are verified together; keep that pairing checked when
updating dependencies. `npm audit` currently reports no known vulnerabilities.

## Source of truth

- `../docs/tools.md` supplies the reviewed English tool reference and the schema
  tables produced by the Rust executable's documentation generator.
- `../.github/scripts/tool-reference.json` supplies reviewed English behavior,
  defaults, response descriptions and example argument objects.
- `i18n/fr-tools.json` and `i18n/it-tools.json` supply complete localized prose,
  tool summaries and omission behavior. Their tool names, parameter paths and
  example objects must match the English source exactly.
- `guide/` and `reference/configuration.md` contain the English manual;
  `fr/` and `it/` contain the corresponding translated pages.
- `.vitepress/config.mts` owns navigation, language metadata, search labels,
  canonical/alternate links and the Pages base path. The displayed release
  version comes from `../Cargo.toml`.

The generator also produces localized `public/llms.txt` indexes and
`public/llms-full.txt` exports (under `fr/` and `it/` for those languages).
They expose the same complete manual as plain text for agents.

After a tool schema or behavior changes, first update the Rust-side reference
using the process in `../docs/releases.md`. Then update both translated note
files and regenerate the website references. Keep parameter names, command
names, environment variables, JSON keys, enum literals and example argument
objects unchanged across languages. Translate explanations rather than the
machine interface. Check that new limits and authentication dependencies are
explained in every language.

The generator's coverage checks reject missing/stale tool or parameter entries
and changed examples. The site validator checks the generated pages; VitePress
also reports unresolved internal page links. Review rendered mobile/desktop
layouts, tables, search, copy controls and language navigation before delivery.

## Add a language

1. Create the locale's index, all twelve guide chapters and configuration
   reference under its language prefix, matching the existing page filenames.
2. Add `i18n/LANGUAGE-tools.json`, translating every prose field and summary
   while preserving the tool/parameter keys and example objects exactly.
3. Extend the reference generator and its language labels, then add the locale
   to `.vitepress/config.mts`: navigation, language switcher, search strings,
   descriptions and canonical/alternate links. Localize the landing-page copy
   in the theme as well.
4. Extend the site validator's expected locales, run generation and the full
   build, and inspect cross-language navigation on equivalent manual pages.

Do not ship partially translated references or silently fall back to English
prose for a locale advertised as complete.

## Build output and packaging

VitePress output is in `.vitepress/dist/`; its cache and `node_modules/` are
Git-ignored. The Cargo package's allowlist and Docker build-context allowlist
exclude the website and its output. Website changes do not require local Cargo
builds, and running npm does not recreate Rust's `target/` directory.

## GitHub Pages deployment

`.github/workflows/pages.yml` builds relevant pull requests with read-only
repository permissions. It does not configure Pages, upload a deploy artifact
or request a deployment identity on pull requests.

Relevant pushes to `main` build all languages, upload only the verified static
output, and deploy through the `github-pages` environment using GitHub's official
Pages actions. Manual workflow dispatch also deploys when run from `main`;
dispatching another branch performs validation only. The separate Rust CI and
crate/release workflows are unchanged.

The repository's **Settings → Pages → Build and deployment → Source** must be
set to **GitHub Actions** before deployment. The workflow reads this setup and
does not attempt administrative enablement. Deployments need `pages: write`
and `id-token: write` only in the deploy job; no personal access token or extra
deployment secret is required. All action references are pinned to commit SHAs.

Only one main-branch deployment runs at a time, without canceling an active
publication. PR updates may cancel their own older validation run. The Pages
artifact has one-day retention. If an upload reports Actions artifact-storage
quota exhaustion, inspect account storage/billing and retention; rerunning the
workflow cannot by itself restore unavailable storage. Do not delete unrelated
artifacts as part of site maintenance.

After deployment, verify the public URL, all three language homepages, one
deep manual link per locale, tool-reference anchors and search. A successful
local build alone does not prove that GitHub Pages is serving the new site.
