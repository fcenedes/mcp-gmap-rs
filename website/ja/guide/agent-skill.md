# エージェントスキル {#agent-skill}

移植可能な `mcp-gmap` スキルは、ツール選択、MCP・CLI の安全な呼び出し、部分失敗の確認、Google の帰属情報の保持をエージェントに教えます。別途設定する実行ファイルや MCP 接続を補助します。

## ソースまたはリリースからのインストール {#install-from-source-or-a-release}

ソースパッケージと各リリースアーカイブに[スキルフォルダー](https://github.com/fcenedes/mcp-gmap-rs/tree/main/skills/mcp-gmap)を含みます。チェックアウトまたは展開先から Codex 用にインストールできます。

```sh
mkdir -p "${CODEX_HOME:-$HOME/.codex}/skills"
cp -R skills/mcp-gmap "${CODEX_HOME:-$HOME/.codex}/skills/"
```

たとえば `$mcp-gmap` として呼び出します。

> $mcp-gmap を使って Gare de Lyon 周辺のカフェを探し、徒歩経路を比較して、ソースリンクを保持してください。

他のエージェントでは同じフォルダーを対応スキルディレクトリへ配置するか、`SKILL.md` とリンク先を読み込みます。指示本文はエージェントに依存せず、`agents/openai.yaml` は任意の Codex UI メタデータです。エージェントの PATH に動く実行ファイルを置くか、[MCP 接続](./mcp)を設定してください。

## スキルが追加するもの {#what-the-skill-adds}

スキルはツール検出から始めるため、古いインストール済みバージョンが最新カタログを持つと仮定しません。正確な座標と返された場所 ID を優先し、検索範囲を小さく保ち、住所検証・集計・沿道検索には無関係な結果から推測するのでなく専用ツールを選びます。

認証情報はツール引数ではなくプロセス・デプロイ設定として扱います。受信 MCP 認可と送信 Google 認証を区別し、まずオフライン診断を使います。ライブ診断プローブには料金が発生する場合があります。トップレベル呼び出しが成功しても、複合出力内の入れ子エラーを確認する必要があります。

[ツール選択リファレンス](https://github.com/fcenedes/mcp-gmap-rs/blob/main/skills/mcp-gmap/references/tool-selection.md)は判断規則、[CLI リファレンス](https://github.com/fcenedes/mcp-gmap-rs/blob/main/skills/mcp-gmap/references/cli.md)は構造化呼び出しと接続診断を解説します。全体の手順は本マニュアルの[活用例](./recipes)、全引数は[ツールリファレンス](../reference/tools)を参照してください。
