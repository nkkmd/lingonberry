# relay / storage separation

**Status: active** | **Last updated: 2026-06-19**

## 目的

この文書は、`relay` と `storage node` を別プロセスとして運用する時の前提をまとめます。  
Phase 2 の Issue 2.5 では、同梱起動を前提にせず、必要なときに片方だけ立て替えられることを確認します。

## 1. 基本方針

- `relay` と `storage node` は独立した binary として起動する
- 片方の停止や置き換えが、もう片方のプロセス定義を壊さない
- runtime の設定はそれぞれの binary で解決する
- 保存先の共有は、明示的に同じ `stateDir` / `dataDir` を選んだ場合だけに限る

## 2. 推奨トポロジ

### 2.1 単独運用

最初の確認は、次のように別々に起動します。

- `relay` は `lingonberry-relay`
- `storage node` は `lingonberry-storage`

この段階では、プロセスを同じホストで動かしても、同梱起動にはしません。

### 2.2 開発時の並走

開発時に両方を並走させる場合は、次を守ります。

- `stateDir` を分ける
- `dataDir` を分ける
- 診断出力はそれぞれの binary で確認する

同じディレクトリを共有する構成は、明示的に意図したときだけにします。  
少なくとも Issue 2.5 では、同梱起動を前提にしないことを確認対象にします。

## 3. 起動確認

### relay

```bash
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

### storage node

```bash
cargo run -p lingonberry-storage -- capabilities
cargo run -p lingonberry-storage -- config
cargo run -p lingonberry-storage -- run
```

## 4. 置き換え時の観点

- `relay` を止めても `storage node` の設定確認はできる
- `storage node` を止めても `relay` の起動確認はできる
- 片方の binary を更新しても、もう片方の起動引数は変えない
- 別 binary に分かれていることを、runbook でも区別する

## 5. Issue 2.5 の確認項目

- `relay` と `storage node` を別プロセスで起動できる
- 同梱起動を前提にした runbook が残っていない
- 片方の再デプロイが、もう片方の運用定義を壊さない

## 6. 確認結果

次のコマンドで別 binary 起動を確認しました。

- `cargo run -p lingonberry-relay -- capabilities`
- `cargo run -p lingonberry-storage -- capabilities`
- `cargo run -p lingonberry-storage -- run`

## 参照

- [運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md)
- [運用準備バックログ](../roadmap/OPERATIONAL_READINESS_BACKLOG.md)
- [運用前提メモ](./OPERATIONAL_PREMISES_MEMO.md)
- [storage node runtime](./STORAGE_NODE_RUNTIME.md)
