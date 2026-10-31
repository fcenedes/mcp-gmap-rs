# コマンドラインインターフェース {#command-line-interface}

CLI は単発の照会、シェル自動化、構造化された子プロセス呼び出しに利用できます。各ツールの入力スキーマと実装は MCP と共通です。

## インストール済みインターフェースの確認 {#discover-the-installed-interface}

```sh
mcp-gmap-rs --version
mcp-gmap-rs --help
mcp-gmap-rs tools
mcp-gmap-rs schema search-nearby
mcp-gmap-rs search-nearby --help
```

`tools` は有効なツールとスキーマを返します。最新版マニュアルの全機能があると仮定せず、インストール済みバージョンを確認してください。サブコマンドなしで起動すると stdio サーバーになるため、単発の自動化では必ずコマンドを指定します。

## オプションと共通 JSON {#flags-or-shared-json}

ツールのオプション名は kebab-case です。オブジェクト・配列の値は JSON で渡します。真偽値オプションは `true` または `false` を受け取り、値なしの指定は `true` を意味します。

```sh
mcp-gmap-rs search-nearby \
  --center '{"value":"48.8584,2.2945","isCoordinates":true}' \
  --keyword cafe --radius 500 --open-now
```

汎用の `call` は MCP の論理ツール名と CLI コマンド名のどちらも受け取り、MCP と同じ引数オブジェクトを使用します。

```sh
mcp-gmap-rs call maps_geocode --args '{"address":"Paris, France"}'
printf '%s' '{"address":"Paris, France"}' | mcp-gmap-rs call geocode --args -
```

プログラムから子プロセスを起動するときは、シリアライズ済み JSON を独立した引数として渡してください。信頼できない文字列をシェルコマンドに埋め込まないでください。トップレベル引数にはオプションがあり、入れ子の項目は JSON オブジェクト内に保持します。

## 出力と失敗の扱い {#output-and-failure-handling}

通常のツール結果は stdout に JSON で出力します。エラーは stderr に出力し、終了ステータスは非ゼロになります。`doctor` は検査失敗時も JSON レポートを stdout に書きます。バッチ、行列、複合処理は終了ステータスが成功でも項目別エラーを含む場合があるため、完了と判断する前に確認してください。

```sh
mcp-gmap-rs place-details --place-id ChIJLU7jZClu5kcR4PcOOO6p3I0 --output place.json
mcp-gmap-rs static-map --center Paris --zoom 12 --output paris.png
```

`--output` は通常のツールで JSON、静的地図では生の PNG を保存します。省略した場合、CLI の静的地図出力は base64 データです。時刻は通常、明示的なタイムゾーンを持つ RFC 3339 を使用します。Time Zone ツールの任意の `timestamp` は Unix **ミリ秒**です。[全ツールと実行例](../reference/tools)を参照してください。
