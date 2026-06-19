# storage node runtime

**Status: draft** | **Last updated: 2026-06-19**

## 目的

この文書は、`storage node` の起動時設定、保存先レイアウト、診断出力をまとめる正本メモです。  
Phase 2 の `storage node` 独立バイナリ化に向けて、設定面の入口を先に固定します。

## 1. 起動面

- binary 名は `lingonberry-storage`
- 既定の実行形は `capabilities`、`config`、`run`、`append`、`retrieve`、`replay`、`list`
- `config` は解決済み設定を表示する
- `run` はサービス状態と解決済み設定を表示する

## 2. 環境変数の責務

- `LINGONBERRY_STATE_DIR` は、`relay` と `storage node` で共通して使う実行ルートの指定です
- `LINGONBERRY_STORAGE_CONFIG` は、`storage node` の設定ファイル位置を上書きするための storage 固有変数です
- 設定値そのものは、できるだけ config file に寄せます
- secret はこの段階では設定ファイルに載せず、[Secret Management](./SECRET_MANAGEMENT.md) に分離します

## 3. 設定の優先順

設定は次の順で解決します。

1. 明示した `LINGONBERRY_STORAGE_CONFIG`
2. 既定の設定ファイル `"$LINGONBERRY_STATE_DIR/storage-config.json"`
3. 環境変数 `LINGONBERRY_STATE_DIR`
4. 既定値 `".lingonberry"`

この文書でいう「優先順」は、まず設定ファイルの場所を決め、その後でファイル中の項目を既定値へ重ねる、という 2 段構えの解決を指します。  
`stateDir` を設定ファイルで上書きした場合、`dataDir`、`backupDir`、`tempDir` が未指定なら、その新しい `stateDir` を基準に既定値を再計算します。

## 4. 設定ファイル形式

設定ファイルは JSON object とします。  
現時点で認識するキーは次の通りです。

- `stateDir`
- `dataDir`
- `backupDir`
- `tempDir`

未指定のキーは、解決済みの `stateDir` を基準に既定値へ戻します。

### 4.0 例

```json
{
  "stateDir": "/var/lib/lingonberry/storage",
  "backupDir": "/var/backups/lingonberry/storage"
}
```

### 4.1 `stateDir`

`storage node` の論理ルートです。  
設定ファイルや運用メモを置く基準点として扱います。

### 4.2 `dataDir`

保存実体を置くディレクトリです。  
既定値は解決済みの `stateDir` と同じにして後方互換を保ちます。  
`relay-wire-log.jsonl` と `canonical-catalog.sqlite3` はこの配下に置きます。

### 4.3 `backupDir`

バックアップや退役時の退避先です。  
既定値は `"$stateDir/backup"` です。

### 4.4 `tempDir`

一時ファイルや再構成途中の作業領域です。  
既定値は `"$stateDir/tmp"` です。

## 5. 実際の出力

`lingonberry-storage config` と `lingonberry-storage run` は、次を返します。

- `configPath`
- `stateDir`
- `dataDir`
- `backupDir`
- `tempDir`
- `rawLogPath`
- `catalogPath`

## 6. 運用上の含意

- `storage node` は `relay` と独立したプロセスとして起動する
- 保存実体は `dataDir` 側に寄せる
- `backupDir` と `tempDir` は `dataDir` とは役割を分ける
- `rawLogPath` は `dataDir/relay-wire-log.jsonl`
- `catalogPath` は `dataDir/canonical-catalog.sqlite3`
- `relay` 側の transport 詳細を `storage node` に持ち込まない

## 参照

- [運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md)
- [運用準備バックログ](../roadmap/OPERATIONAL_READINESS_BACKLOG.md)
- [運用前提メモ](./OPERATIONAL_PREMISES_MEMO.md)
- [storage node README](../../packages/storage/README.md)
