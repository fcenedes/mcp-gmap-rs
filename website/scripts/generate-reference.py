"""Generate three complete tool references from reviewed, schema-derived sources."""

import argparse
import json
from pathlib import Path
import re
import shlex


WEBSITE = Path(__file__).resolve().parents[1]
ROOT = WEBSITE.parent
SOURCE = ROOT / "docs/tools.md"
NOTES = ROOT / ".github/scripts/tool-reference.json"
WORDS = {
    "en": {
        "title": "Tool reference", "intro": "All 23 tools, with every parameter, default, constraint and example. The same interface is available through MCP and the CLI.",
        "calling": "Calling conventions", "cli": "CLI command", "access": "Google access", "parameters": "Parameters", "parameter": "Parameter / CLI flag", "type": "Type and schema limits", "required": "Required", "default": "Default when omitted", "description": "Description", "behavior": "Behavior and restrictions", "result": "Result", "example": "Example", "mcp": "MCP arguments", "equivalent": "Equivalent CLI call", "yes": "Yes", "no": "No", "conditional": "When parent is supplied", "contents": "Tools",
        "conventions": ["The examples are MCP argument objects. For a raw MCP `tools/call`, use the tool name as `name` and the object as `arguments`. Hosts may prefix tool names. The equivalent CLI calls below use `mcp-gmap-rs call TOOL --args JSON`.", "Only top-level parameters have CLI flags. Pass nested objects and arrays as JSON. A bare boolean flag means `true`; `--flag false` sets `false`. Unknown properties are rejected. Inspect your installed release with `tools` or `schema TOOL`.", "Google authentication and language are process configuration, not tool parameters. Exact route coordinates avoid address resolution; named locations may also require an API key for Geocoding. See [authentication](../guide/authentication) and [configuration](../guide/configuration).", "Calls can incur Google charges. Composite and matrix results may contain per-item errors even when the tool succeeds. Preserve Google source links, review/photo authors and AI-summary disclosures. Examples illustrate inputs, not current coverage or results.", "Types, required fields and schema bounds come from the executable's schema-derived reference. Defaults, result descriptions and cross-field restrictions are reviewed against the implementation. Google may impose additional coverage and quota limits."],
    },
    "fr": {
        "title": "Référence des outils", "intro": "Les 23 outils, avec tous leurs paramètres, valeurs par défaut, contraintes et exemples. La même interface est disponible via MCP et le CLI.",
        "calling": "Conventions d’appel", "cli": "Commande CLI", "access": "Accès Google", "parameters": "Paramètres", "parameter": "Paramètre / option CLI", "type": "Type et limites du schéma", "required": "Obligatoire", "default": "Valeur en cas d’omission", "description": "Description", "behavior": "Comportement et restrictions", "result": "Résultat", "example": "Exemple", "mcp": "Arguments MCP", "equivalent": "Appel CLI équivalent", "yes": "Oui", "no": "Non", "conditional": "Si le parent est fourni", "contents": "Outils",
        "conventions": ["Les exemples sont des objets d’arguments MCP. Pour un appel MCP brut `tools/call`, utilisez le nom de l’outil dans `name` et l’objet dans `arguments`. L’hôte peut préfixer les noms. Les appels CLI équivalents ci-dessous utilisent `mcp-gmap-rs call OUTIL --args JSON`.", "Seuls les paramètres de premier niveau ont une option CLI. Passez les objets imbriqués et tableaux en JSON. Une option booléenne seule signifie `true` ; `--flag false` fixe `false`. Les propriétés inconnues sont rejetées. Consultez la version installée avec `tools` ou `schema OUTIL`.", "L’authentification Google et la langue se configurent au niveau du processus, pas de l’outil. Des coordonnées exactes évitent la résolution d’adresse ; les lieux nommés peuvent aussi nécessiter une clé API pour Geocoding. Voir [l’authentification](../guide/authentication) et [la configuration](../guide/configuration).", "Les appels peuvent être facturés par Google. Les résultats composites et matriciels peuvent contenir des erreurs par élément même si l’outil réussit. Conservez les liens Google, les auteurs d’avis/photos et les mentions des résumés IA. Les exemples illustrent des entrées, sans garantir des résultats ou une couverture actuels.", "Les types, champs obligatoires et bornes proviennent de la référence issue des schémas de l’exécutable. Les valeurs par défaut, résultats et restrictions entre champs sont vérifiés dans l’implémentation. Google peut imposer d’autres limites de couverture et de quota."],
    },
    "it": {
        "title": "Riferimento strumenti", "intro": "Tutti i 23 strumenti, con ogni parametro, valore predefinito, vincolo ed esempio. La stessa interfaccia è disponibile tramite MCP e CLI.",
        "calling": "Convenzioni di chiamata", "cli": "Comando CLI", "access": "Accesso Google", "parameters": "Parametri", "parameter": "Parametro / opzione CLI", "type": "Tipo e limiti dello schema", "required": "Obbligatorio", "default": "Valore se omesso", "description": "Descrizione", "behavior": "Comportamento e restrizioni", "result": "Risultato", "example": "Esempio", "mcp": "Argomenti MCP", "equivalent": "Chiamata CLI equivalente", "yes": "Sì", "no": "No", "conditional": "Se il genitore è presente", "contents": "Strumenti",
        "conventions": ["Gli esempi sono oggetti di argomenti MCP. Per una chiamata MCP diretta `tools/call`, usa il nome dello strumento in `name` e l’oggetto in `arguments`. L’host può aggiungere un prefisso ai nomi. Le chiamate CLI equivalenti qui sotto usano `mcp-gmap-rs call STRUMENTO --args JSON`.", "Solo i parametri di primo livello hanno un’opzione CLI. Passa oggetti annidati e array in JSON. Un’opzione booleana da sola significa `true`; `--flag false` imposta `false`. Le proprietà sconosciute vengono rifiutate. Consulta la versione installata con `tools` o `schema STRUMENTO`.", "L’autenticazione Google e la lingua sono configurazioni del processo, non parametri dello strumento. Le coordinate esatte evitano la risoluzione degli indirizzi; i luoghi nominati possono richiedere anche una chiave API per Geocoding. Vedi [autenticazione](../guide/authentication) e [configurazione](../guide/configuration).", "Le chiamate possono comportare costi Google. I risultati compositi e delle matrici possono contenere errori per elemento anche se lo strumento riesce. Mantieni i link Google, gli autori di recensioni/foto e le diciture dei riepiloghi IA. Gli esempi illustrano gli input, senza garantire risultati o copertura attuali.", "Tipi, campi obbligatori e limiti provengono dal riferimento derivato dagli schemi dell’eseguibile. Valori predefiniti, risultati e vincoli tra campi sono verificati nell’implementazione. Google può imporre ulteriori limiti di copertura e quota."],
    },
}


def schemas():
    source = SOURCE.read_text(encoding="utf-8")
    tools = {}
    for match in re.finditer(r"^## (maps_\w+)\n(.*?)(?=^## maps_|\Z)", source, re.M | re.S):
        name, body = match.groups()
        rows = []
        for row in body.splitlines():
            if row.startswith("| `"):
                columns = [cell.strip() for cell in re.split(r"(?<!\\)\|", row)[1:-1]]
                if len(columns) != 5:
                    raise SystemExit(f"Invalid schema-derived table row: {name}")
                path = re.match(r"`([^`]+)`", columns[0])[1]
                rows.append((path, *columns[:3]))
        tools[name] = {"summary": body.strip().split("\n\n", 1)[0], "rows": rows}
    return tools


def translate_type(value, lang):
    if lang == "en":
        return value
    translations = {
        "fr": {"array of string": "tableau de chaînes", "array of object": "tableau d’objets", "array of number": "tableau de nombres", "string": "chaîne", "integer": "entier", "number": "nombre", "boolean": "booléen", "object": "objet", "items": "éléments", "each item at least": "chaque élément contient au moins", "at least": "au moins", "character(s)": "caractère(s)", "values:": "valeurs :", "pattern:": "motif :", "unbounded": "sans borne"},
        "it": {"array of string": "array di stringhe", "array of object": "array di oggetti", "array of number": "array di numeri", "string": "stringa", "integer": "intero", "number": "numero", "boolean": "booleano", "object": "oggetto", "items": "elementi", "each item at least": "ogni elemento contiene almeno", "at least": "almeno", "character(s)": "carattere/i", "values:": "valori:", "pattern:": "modello:", "unbounded": "senza limite"},
    }[lang]
    # Schema enums and regexes in backticks are machine identifiers, never translate them.
    parts = re.split(r"(`[^`]*`)", value)
    for index in range(0, len(parts), 2):
        for old, new in translations.items():
            parts[index] = parts[index].replace(old, new)
    return "".join(parts)


def cell(value):
    return str(value).replace("|", "\\|").replace("\n", " ")


def render(lang, notes, tools):
    t = WORDS[lang]
    lines = ["---", "title: " + t["title"], "description: " + t["intro"], "---", "", "<!-- Generated by scripts/generate-reference.py. Edit the source notes, then regenerate. -->", "", "# " + t["title"], "", t["intro"], "", "## " + t["calling"], ""]
    for paragraph in t["conventions"]:
        lines += [paragraph, ""]
    lines += ["## " + t["contents"], ""]
    lines.extend(f"- [`{name}`](#{name})" for name in tools)
    for name, tool in tools.items():
        note = notes[name]
        command = name.removeprefix("maps_").replace("_", "-")
        lines += ["", f"## {name} {{#{name}}}", "", note.get("summary", tool["summary"]), "", note["purpose"], "", f'**{t["cli"]}:** `{command}`. **{t["access"]}:** {note["authentication"]}', "", "### " + t["parameters"], "", "| " + " | ".join(t[key] for key in ["parameter", "type", "required", "default", "description"]) + " |", "| --- | --- | --- | --- | --- |"]
        for path, label, shape, required in tool["rows"]:
            detail = note["parameters"][path]
            status = t[{"Yes": "yes", "No": "no", "When parent is supplied": "conditional"}[required]]
            # Labels and shapes are already escaped by the upstream reference generator.
            lines.append(f"| {label} | {translate_type(shape, lang)} | {status} | {cell(detail['default'])} | {cell(detail['description'])} |")
        lines += ["", "### " + t["behavior"], ""]
        lines.extend("- " + value for value in note["notes"])
        lines += ["", "### " + t["result"], "", note["result"], "", "### " + t["example"], "", t["mcp"] + ":", "", "```json", json.dumps(note["example"], ensure_ascii=False, indent=2), "```", "", t["equivalent"] + ":", "", "```sh", f"mcp-gmap-rs call {name} --args " + shlex.quote(json.dumps(note["example"], ensure_ascii=False, separators=(",", ":"))), "```", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    tools = schemas()
    english = json.loads(NOTES.read_text(encoding="utf-8"))
    if set(english) != set(tools):
        raise SystemExit("The English reference and reviewed tool notes differ")
    locales = {"en": english}
    for lang in ["fr", "it"]:
        locales[lang] = json.loads((WEBSITE / f"i18n/{lang}-tools.json").read_text(encoding="utf-8"))
    outputs = {}
    for lang, notes in locales.items():
        if set(notes) != set(english):
            raise SystemExit(f"{lang}: incomplete tool translations")
        for name, note in notes.items():
            paths = {row[0] for row in tools[name]["rows"]}
            if set(note["parameters"]) != paths or paths != set(english[name]["parameters"]):
                raise SystemExit(f"{lang}.{name}: incomplete parameter translations")
            if note["example"] != english[name]["example"] or len(note["notes"]) != len(english[name]["notes"]):
                raise SystemExit(f"{lang}.{name}: example/behavior drift")
            for field in ["purpose", "authentication", "result", "notes"] + (["summary"] if lang != "en" else []):
                if not note.get(field):
                    raise SystemExit(f"{lang}.{name}: empty {field}")
            for path, detail in note["parameters"].items():
                if not detail.get("description") or not detail.get("default"):
                    raise SystemExit(f"{lang}.{name}.{path}: incomplete description/default")
        prefix = "" if lang == "en" else lang + "/"
        outputs[WEBSITE / f"{prefix}reference/tools.md"] = render(lang, notes, tools)
    category = {
        "geography": "geocode reverse_geocode elevation timezone static_map batch_geocode",
        "places": "validate_address area_insights search_places search_nearby place_details explore_area compare_places local_rank_tracker",
        "routes": "snap_to_roads nearest_roads directions distance_matrix plan_route search_along_route",
        "environment": "pollen weather air_quality",
    }
    release = re.search(r'^version = "([^"]+)"', (ROOT / "Cargo.toml").read_text(), re.M)[1]
    catalog = {"version": release, "tools": [{"name": name, "category": next(key for key, names in category.items() if name.removeprefix("maps_") in names.split()), "summary": {lang: notes[name].get("summary", tool["summary"]) for lang, notes in locales.items()}} for name, tool in tools.items()]}
    outputs[WEBSITE / "data/catalog.json"] = json.dumps(catalog, ensure_ascii=False, indent=2) + "\n"
    # Readable exports let agents fetch the same complete manual without browser UI.
    for lang in locales:
        prefix = "" if lang == "en" else lang + "/"
        url = "https://fcenedes.github.io/mcp-gmap-rs/" + prefix
        pages = sorted((WEBSITE / prefix / "guide").glob("*.md"))
        pages += [WEBSITE / prefix / "reference/configuration.md", WEBSITE / prefix / "reference/tools.md"]
        index = ["# mcp-gmap-rs", "", WORDS[lang]["intro"], "", "## " + WORDS[lang]["contents"], ""]
        full = ["# mcp-gmap-rs / " + lang, "", url, ""]
        for path in pages:
            content = outputs.get(path, path.read_text(encoding="utf-8") if path.exists() else "")
            content = re.sub(r"\A---\n.*?\n---\n", "", content, count=1, flags=re.S)
            title = re.search(r"^# (.+)$", content, re.M)[1]
            route = path.relative_to(WEBSITE / prefix).as_posix().removesuffix(".md")
            index.append(f"- [{title}]({url}{route})")
            full += ["", f"<!-- {url}{route} -->", "", content]
        full_label, languages_label = {"en": ("Full text", "Languages"), "fr": ("Texte intégral", "Langues"), "it": ("Testo integrale", "Lingue")}[lang]
        index += ["", f"- [{full_label}]({url}llms-full.txt)", "", "## " + languages_label, "", "- [English](https://fcenedes.github.io/mcp-gmap-rs/llms.txt)", "- [Français](https://fcenedes.github.io/mcp-gmap-rs/fr/llms.txt)", "- [Italiano](https://fcenedes.github.io/mcp-gmap-rs/it/llms.txt)", ""]
        outputs[WEBSITE / f"public/{prefix}llms.txt"] = "\n".join(index)
        outputs[WEBSITE / f"public/{prefix}llms-full.txt"] = "\n".join(full)
    stale = []
    for path, content in outputs.items():
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8") != content:
                stale.append(str(path.relative_to(ROOT)))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
    if stale:
        raise SystemExit("Stale references; run npm run generate in website/: " + ", ".join(stale))
    print(f"Verified en/fr/it: {len(tools)} tools, {sum(len(tool['rows']) for tool in tools.values())} parameter paths per language; examples match")


if __name__ == "__main__":
    main()
