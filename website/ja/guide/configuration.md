# 設定 {#configuration}

CLI と両方の MCP トランスポートは同じ起動設定を共有します。ローカルではファイル、コンテナでは環境変数、プロセス単位で明示的に上書きする場合はオプションを使用します。

## 優先順位 {#precedence}

値は次の順番で適用され、後の層が優先されます。

**組み込みの既定値 → 任意の dotenv ファイル → プロセス環境変数 → CLI オプション**

既定のファイルは現在の作業ディレクトリの `.env` です。親ディレクトリは検索しません。既定のファイルが存在しなくても起動できますが、読み取り不能や構文エラーがある場合は内容を表示せず起動に失敗します。

```sh
mcp-gmap-rs --env-file /etc/mcp-gmap/runtime.env serve http
mcp-gmap-rs --no-env-file tools
```

`MCP_ENV_FILE` でもファイルを指定できますが、`--env-file` が優先されます。明示的に選択したファイルは存在する必要があります。`--no-env-file` は `MCP_ENV_FILE` を含むファイル読み込みを無効にし、`--env-file` とは併用できません。dotenv の解析はプロセス環境を変更しません。

## マウントした設定の上書き {#override-a-mounted-configuration}

```dotenv
GOOGLE_MAPS_LANGUAGE=en
GOOGLE_MAPS_TIMEOUT=30
GOOGLE_MAPS_ENABLED_TOOLS=maps_search_places,maps_place_details
```

このファイルをマウントした状態で、環境変数により言語を、オプションによりタイムアウトを変更できます。

```sh
GOOGLE_MAPS_LANGUAGE=fr mcp-gmap-rs \
  --env-file /etc/mcp-gmap/runtime.env --timeout 15 tools
```

`--allowed-host` と `--allowed-origin` を繰り返し指定すると、対応するファイル・環境変数のリスト全体を置き換えます。明示的な空の環境変数もファイルを上書きします。任意のトークンやリストは消去できますが、空のタイムアウトは無効です。

## 公開するツールの制御 {#control-the-tool-surface}

`GOOGLE_MAPS_ENABLED_TOOLS` または `--enabled-tools` に、`maps_weather,maps_air_quality` のような論理名をカンマ区切りで指定します。`*` は全ツールを表します。不明な名前は起動エラーになり、無効化されたツールは検出されず呼び出せません。ツール検出とスキーマ参照には認証情報が不要ですが、API 操作には適切な Google 認証情報が必要です。

アプリケーション設定は起動時に読み込みます。環境変数、認証 JSON、Kubernetes Secret/ConfigMap の変更後は再起動してください。ネイティブ TLS の証明書・秘密鍵・クライアント CA ファイルは例外で、[SIGHUP による再読み込み](./production#native-tls-and-mutual-tls)に対応します。

全 21 設定の既定値、オプション、範囲は[設定リファレンス](../reference/configuration)を参照してください。サイトの言語選択は、Google の応答言語を制御する `GOOGLE_MAPS_LANGUAGE` を変更しません。
