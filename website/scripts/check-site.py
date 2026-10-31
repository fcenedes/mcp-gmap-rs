"""Verify the built multilingual manual and GitHub Pages URL contract."""

import argparse
from collections import defaultdict
from html.parser import HTMLParser
import json
from pathlib import Path
import sys
from urllib.parse import unquote, urljoin, urlsplit


WEBSITE = Path(__file__).resolve().parents[1]
REPOSITORY = WEBSITE.parent
BASE = "/mcp-gmap-rs/"
ORIGIN = "https://fcenedes.github.io"
LANGUAGES = json.loads((WEBSITE / "i18n/locales.json").read_text(encoding="utf-8"))
METADATA = {key: json.loads((WEBSITE / f"i18n/{key}.json").read_text(encoding="utf-8")) for key in LANGUAGES}
LOCALES = {key: "" if key == "en" else key + "/" for key in LANGUAGES}
GUIDES = (
    "getting-started", "google-cloud", "authentication", "configuration", "cli",
    "mcp", "deployment", "production", "recipes", "troubleshooting",
    "development", "agent-skill",
)
DOCUMENTS = ("", *(f"guide/{name}" for name in GUIDES),
             "reference/tools", "reference/configuration")


class Page(HTMLParser):
    def __init__(self, source):
        super().__init__(convert_charrefs=True)
        self.lang = ""
        self.direction = ""
        self.ids = set()
        self.links = []
        self.canonical = []
        self.alternates = defaultdict(list)
        self.title = []
        self.h1 = []
        self._title = False
        self._h1 = False
        self._tool = None
        self._pre = None
        self.tool_text = defaultdict(list)
        self.tool_examples = defaultdict(list)
        self.feed(source)

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if tag == "html":
            self.lang = attrs.get("lang", "")
            self.direction = attrs.get("dir", "")
        if attrs.get("id"):
            self.ids.add(attrs["id"])
        if tag == "title":
            self._title = True
        if tag == "h1":
            self._h1 = True
        if tag == "h2":
            heading = attrs.get("id", "")
            self._tool = heading if heading.startswith("maps_") else None
        if tag == "pre" and self._tool:
            self._pre = []
        if tag == "link":
            relation = attrs.get("rel", "").split()
            if "canonical" in relation:
                self.canonical.append(attrs.get("href", ""))
            if "alternate" in relation and "hreflang" in attrs:
                self.alternates[attrs["hreflang"]].append(attrs.get("href", ""))
        for name in ("href", "src", "poster"):
            if attrs.get(name):
                self.links.append((tag, name, attrs[name]))
        if attrs.get("srcset"):
            # Generated image candidates are ordinary URLs, not data URIs.
            if not attrs["srcset"].startswith("data:"):
                for candidate in attrs["srcset"].split(","):
                    url = candidate.strip().split()
                    if url:
                        self.links.append((tag, "srcset", url[0]))

    def handle_endtag(self, tag):
        if tag == "title":
            self._title = False
        if tag == "h1":
            self._h1 = False
        if tag == "pre" and self._pre is not None:
            self.tool_examples[self._tool].append("".join(self._pre))
            self._pre = None

    def handle_data(self, value):
        if self._title:
            self.title.append(value)
        if self._h1:
            self.h1.append(value)
        if self._tool:
            self.tool_text[self._tool].append(value)
        if self._pre is not None:
            self._pre.append(value)


def route_for_file(file, dist):
    relative = file.relative_to(dist).as_posix()
    return relative.removesuffix("index.html") if relative.endswith("index.html") else relative.removesuffix(".html")


def expected_html(dist, route):
    if not route or route.endswith("/"):
        return dist / route / "index.html"
    return dist / (route + ".html")


def resolve_target(dist, pathname):
    relative = unquote(pathname.removeprefix(BASE))
    candidate = (dist / relative).resolve()
    if not candidate.is_relative_to(dist):
        return None
    choices = [candidate]
    if not candidate.suffix:
        choices += [candidate.with_suffix(".html"), candidate / "index.html"]
    elif pathname.endswith("/"):
        choices.append(candidate / "index.html")
    return next((item for item in choices if item.is_file()), None)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dist", type=Path, default=WEBSITE / ".vitepress" / "dist")
    args = parser.parse_args()
    dist = args.dist.resolve()
    errors = []
    if not dist.is_dir():
        raise SystemExit(f"Built site is missing: {dist}; run the website build first")
    notes = json.loads((REPOSITORY / ".github/scripts/tool-reference.json").read_text())
    parameter_count = sum(len(tool["parameters"]) for tool in notes.values())
    if len(notes) != 42 or parameter_count != 325:
        errors.append("The tool catalog changed: update the manual coverage contract")

    pages = {file: Page(file.read_text(encoding="utf-8")) for file in dist.rglob("*.html")}
    for lang, prefix in LOCALES.items():
        source_guide = WEBSITE / prefix / "guide"
        actual = {file.stem for file in source_guide.glob("*.md")}
        if actual != set(GUIDES):
            errors.append(f"{lang}: guide source parity differs: {sorted(actual ^ set(GUIDES))}")
        for route in DOCUMENTS:
            localized = prefix + route
            file = expected_html(dist, localized)
            if file not in pages:
                errors.append(f"Missing localized page: {localized or '/'}")
                continue
            page = pages[file]
            label = file.relative_to(dist).as_posix()
            if page.lang != METADATA[lang]["lang"]:
                errors.append(f"{label}: html lang must be {METADATA[lang]['lang']}, found {page.lang!r}")
            if page.direction != METADATA[lang]["dir"]:
                errors.append(f"{label}: html dir must be {METADATA[lang]['dir']}, found {page.direction!r}")
            if not "".join(page.title).strip():
                errors.append(f"{label}: page title is empty")
            if "{#" in "".join(page.title):
                errors.append(f"{label}: page title exposes a Markdown anchor")
            if not "".join(page.h1).strip():
                errors.append(f"{label}: visible h1 is missing")
            canonical = ORIGIN + BASE + localized
            if page.canonical != [canonical]:
                errors.append(f"{label}: canonical must be {canonical}")
            for alternate, alternate_prefix in LOCALES.items():
                expected = ORIGIN + BASE + alternate_prefix + route
                if page.alternates[METADATA[alternate]["lang"]] != [expected]:
                    errors.append(f"{label}: {alternate} alternate must be {expected}")
            if route == "reference/tools":
                for tool_name, tool in notes.items():
                    if tool_name not in page.ids:
                        errors.append(f"{label}: missing tool anchor #{tool_name}")
                    content = "".join(page.tool_text[tool_name])
                    for parameter in tool["parameters"]:
                        if parameter not in content:
                            errors.append(f"{label}: {tool_name}.{parameter} is undocumented")
                    examples = page.tool_examples[tool_name]
                    objects = []
                    for example in examples:
                        try:
                            objects.append(json.loads(example))
                        except json.JSONDecodeError:
                            pass
                    if tool["example"] not in objects:
                        errors.append(f"{label}: {tool_name} MCP example is missing or changed")
                    if not any(f"mcp-gmap-rs call {tool_name} --args " in example for example in examples):
                        errors.append(f"{label}: {tool_name} CLI example is missing")

    checked_links = 0
    for file, page in pages.items():
        source = ORIGIN + BASE + route_for_file(file, dist)
        label = file.relative_to(dist).as_posix()
        for tag, attribute, href in page.links:
            original = urlsplit(href)
            if not href or href.startswith("?") or original.scheme in {"mailto", "tel", "data", "javascript", "blob"}:
                continue
            absolute = urlsplit(urljoin(source, href))
            if absolute.scheme not in {"http", "https"} or absolute.netloc != urlsplit(ORIGIN).netloc:
                continue
            # Search URLs are generated at runtime, not static destinations.
            if absolute.query:
                continue
            if not absolute.path.startswith(BASE):
                errors.append(f"{label}: {attribute} escapes Pages base: {href}")
                continue
            target = resolve_target(dist, absolute.path)
            if target is None:
                errors.append(f"{label}: broken local {tag}[{attribute}]: {href}")
                continue
            checked_links += 1
            fragment = unquote(absolute.fragment)
            if fragment and target.suffix == ".html" and fragment not in pages[target].ids:
                errors.append(f"{label}: missing #{fragment} in {target.relative_to(dist)}")

    if errors:
        unique = list(dict.fromkeys(errors))
        print(f"Manual verification failed ({len(unique)} issues):", file=sys.stderr)
        for error in unique:
            print(f"- {error}", file=sys.stderr)
        raise SystemExit(1)
    print(f"Verified {len(DOCUMENTS) * len(LOCALES)} localized pages, {len(LOCALES)} languages, {len(notes)} tools/{parameter_count} parameter paths per language, and {checked_links} internal links/assets")


if __name__ == "__main__":
    main()
