# Compétence pour les agents

Le dossier portable [skills/mcp-gmap](https://github.com/fcenedes/mcp-gmap-rs/tree/main/skills/mcp-gmap) apprend à un agent à choisir les outils, utiliser MCP ou la CLI, inspecter les erreurs partielles et préserver les attributions. Il est inclus dans le paquet source et les archives de version.

Cette compétence contient des instructions ; elle n’installe pas le binaire et ne crée pas la connexion MCP. Configurez d’abord la [CLI](./cli) ou le [transport MCP](./mcp), puis installez le dossier dans l’emplacement reconnu par votre agent.

## Installation pour Codex

Depuis un checkout ou une archive extraite :

```sh
mkdir -p "${CODEX_HOME:-$HOME/.codex}/skills"
cp -R skills/mcp-gmap "${CODEX_HOME:-$HOME/.codex}/skills/"
```

Exemple d’invocation : « Utilise $mcp-gmap pour trouver trois cafés près de la gare de Lyon, puis compare les trajets à pied. » Le fichier `agents/openai.yaml` ajoute des métadonnées d’interface facultatives ; le corps de `SKILL.md` et ses références restent indépendants de l’agent. Pour un autre hôte, utilisez son répertoire de compétences ou chargez directement ces instructions.

## Conduite attendue

L’agent découvre les capacités de la version installée avant d’appeler un outil, choisit la requête la plus limitée qui répond au besoin et inspecte les schémas plutôt que d’inventer des paramètres. Il garde les clés et les fichiers JSON dans la configuration du processus, préserve les coordonnées exactes, lit les erreurs par élément et garde les auteurs/liens/déclarations d’IA avec le contenu Google.

La [référence des outils](../reference/tools) constitue le manuel détaillé ; `schema TOOL` fournit le contrat de la version réellement installée. Les appels et sondes `--live` peuvent être facturés. Une compétence ne remplace ni les autorisations Google ni les paramètres de sécurité du serveur HTTP.
