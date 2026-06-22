# 0.1.0 公開前チェックリスト

**Status: draft** | **Last updated: 2026-06-22**

## 目的

この文書は、Lingonberry を `0.1.0` として OSS 公開・配布する前に、最低限確認する項目をまとめるための実務用チェックリストです。

対象は source release を前提にした公開リポジトリです。  
README、仕様文書、schema、fixtures、実装のあいだで、公開してよい範囲が揃っているかを確認します。

このチェックリストは、「公開してよいか」を判断するための最終確認です。  
未完了項目がある場合は、その理由を release note または README に明示します。

## 公開範囲の確認

- [ ] `0.1.0` で公開する範囲を 1 文で説明できる
- [ ] `core protocol` と `application profile` の境界を説明できる
- [ ] `relay` と `storage node` の責務分離を説明できる
- [ ] `wire` と `canonical` が別プロトコルではなく別表現であると説明できる
- [ ] `Toitoi` は application profile であり、core ではないと明示できる
- [ ] Phase 1 の JavaScript 実装が検証用ブートストラップであることを説明できる
- [ ] Phase 2 以降の Rust + SQLite 本命実装との関係を説明できる
- [ ] 未完了の roadmap / backlog を draft / active として扱う方針がある

## 法務とライセンス

- [x] `LICENSE` を追加している
- [x] README に license の種別を明記している
- [ ] 主要な外部由来素材があれば、再配布条件を確認している
- [ ] 追加した文書やコードに、公開できない第三者情報が混ざっていない
- [ ] 依存物や引用物の出典を、必要に応じて明示している

### 現時点の判断

- `Apache-2.0` を採用する
- `NOTICE` は現時点では追加しない
- 今後、第三者由来の notice を同梱する場合だけ `NOTICE` を再検討する

## リポジトリ衛生

- [ ] 秘密情報、API key、認証情報、個人情報を含めていない
- [ ] ローカル実行時の生成物が `.gitignore` で適切に除外されている
- [ ] 公開不要な一時ファイルやキャッシュが残っていない
- [ ] `git status --short` が空、または差分が意図通りで説明できる
- [ ] ファイル名、配置、リンク切れが公開前に整理されている

## 仕様整合

- [ ] [GLOSSARY](../concepts/GLOSSARY.md) と実装・README の用語が一致している
- [ ] [CONCEPT_MODEL](../concepts/CONCEPT_MODEL.md) と README の説明が一致している
- [ ] [CARRIER](../concepts/CARRIER.md) と carrier 実装の説明が一致している
- [ ] [Toitoi Application Profile](../profiles/TOITOI_APPLICATION_PROFILE.md) と core の境界が一致している
- [ ] [PROTOCOL_NATIVE_WIRE_FORMAT](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md) と schema が矛盾していない
- [ ] [schemas/README](../../schemas/README.md) と各 schema ファイルの説明が一致している
- [ ] README、operations README、roadmap README の案内先が一致している
- [ ] `Status` と `Last updated` が、更新した文書でそろっている
- [ ] `（完了済み）` の表記が必要な完了済み見出しに付いている

## 実装確認

- [ ] `cargo run -p lingonberry-relay -- capabilities` を実行できる
- [ ] `cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787` を実行できる
- [ ] `cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json` を実行できる
- [ ] `cargo run -p lingonberry-relay -- export-archive /tmp/lingonberry-archive` を実行できる
- [ ] `cargo run -p lingonberry-relay -- import-archive /tmp/lingonberry-archive` を実行できる
- [ ] `cargo run -p lingonberry-storage -- capabilities` を実行できる
- [ ] `cargo run -p lingonberry-storage -- config` を実行できる
- [ ] `cargo run -p lingonberry-storage -- ready` を実行できる
- [ ] `cargo run -p lingonberry-storage -- run` を実行できる
- [ ] `fixtures/` の最小入力が validate できる
- [ ] 不正な fixture が reject される
- [ ] relay と storage node の責務が README の記述どおりに分かれている
- [ ] 公開向けの README 例が実際の CLI と一致している

## 配布物

- [ ] 公開対象のディレクトリ構成を 1 つの README から辿れる
- [ ] `README.md` から最初の確認先が分かる
- [ ] `docs/architecture/`、`docs/concepts/`、`docs/protocols/`、`docs/operations/`、`docs/roadmap/`、`docs/profiles/`、`schemas/`、`fixtures/` の役割が説明できる
- [ ] release note で `0.1.0` の範囲を説明できる
- [ ] 必要な場合は source archive の作成手順を説明できる
- [ ] `0.1.0` のタグ名と公開名が一致している

## 運用・安全

- [ ] public relay の trust model が文書化されている
- [ ] public / private の扱いが core から外れている
- [ ] access / retention の扱いが文書化されている
- [ ] storage path、state dir、backup の考え方が文書化されている
- [ ] readiness の確認方法が `ready` コマンドまたは endpoint で説明できる
- [ ] 0.1.0 でサポートしないものを明示できる
- [ ] 失敗時に参照すべき文書が 1 本化されている

## 公開直前の最終確認

- [ ] `git status --short` が空、または公開してよい差分だけになっている
- [ ] ルート README が初見の読者にとって入口として機能する
- [ ] 0.1.0 の release note に、未完了項目と今後のロードマップを分けて書いている
- [ ] 公開後に直したい事項が backlog に落ちている

## 参照

- [README](../../README.md)
- [概念](../concepts/README.md)
- [Protocols](../protocols/README.md)
- [Schemas](../../schemas/README.md)
- [Operations](../operations/README.md)
- [ロードマップ](./README.md)
- [Toitoi Application Profile](../profiles/TOITOI_APPLICATION_PROFILE.md)
