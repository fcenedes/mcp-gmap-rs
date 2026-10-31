# Agenten-Skill

Der portable Skill `mcp-gmap` erklärt einem Agenten Werkzeugauswahl, sichere MCP-/CLI-Aufrufe, Teilfehlerprüfung und den Erhalt von Google-Quellenangaben. Er ergänzt das separat konfigurierte Programm oder die MCP-Verbindung.

## Aus Quellcode oder Release installieren

Das Quellpaket und jedes Release-Archiv enthalten den [Skill-Ordner](https://github.com/fcenedes/mcp-gmap-rs/tree/main/skills/mcp-gmap). Installieren Sie ihn aus einem Checkout oder entpackten Archiv für Codex:

```sh
mkdir -p "${CODEX_HOME:-$HOME/.codex}/skills"
cp -R skills/mcp-gmap "${CODEX_HOME:-$HOME/.codex}/skills/"
```

Rufen Sie ihn als `$mcp-gmap` auf, zum Beispiel:

> Verwende $mcp-gmap, um Cafés in der Nähe des Gare de Lyon zu finden, Fußwege zu vergleichen und Quelllinks zu erhalten.

Für andere Agenten legen Sie denselben Ordner in deren unterstütztes Skill-Verzeichnis oder laden `SKILL.md` mit seinen verlinkten Referenzen. Der Anweisungstext ist agentenunabhängig; `agents/openai.yaml` liefert optionale Codex-Oberflächenmetadaten. Installieren Sie ein funktionierendes Programm im PATH des Agenten oder konfigurieren Sie dessen [MCP-Verbindung](./mcp).

## Was der Skill ergänzt

Der Skill beginnt mit der Werkzeugerkennung und setzt daher nicht voraus, dass ältere Releases den neuesten Katalog besitzen. Er bevorzugt exakte Koordinaten und zurückgegebene Orts-IDs, hält Suchbereiche klein und verwendet spezialisierte Werkzeuge für Adressvalidierung, aggregierte Zählungen oder Routenbegleitsuche, statt solche Operationen aus ungeeigneten Ergebnissen abzuleiten.

Zugangsdaten behandelt er als Prozess-/Deployment-Konfiguration, niemals als Werkzeugargumente. Er trennt eingehende MCP-Autorisierung von ausgehendem Google-Zugriff und beginnt mit Offline-Diagnose. Live-Diagnoseproben können kostenpflichtig sein. Zusammengesetzte Ergebnisse sind auch bei erfolgreichem Gesamtaufruf auf verschachtelte Fehler zu prüfen.

Die [Werkzeugauswahlreferenz](https://github.com/fcenedes/mcp-gmap-rs/blob/main/skills/mcp-gmap/references/tool-selection.md) beschreibt die Entscheidungsregeln. Die [CLI-Referenz](https://github.com/fcenedes/mcp-gmap-rs/blob/main/skills/mcp-gmap/references/cli.md) behandelt strukturierte Aufrufe und Zugriffsdiagnose. Vollständige Abläufe stehen in den [Rezepten](./recipes), alle Argumente in der [Werkzeugreferenz](../reference/tools).
