# Phase 12 実装完了チェックリスト

**Status: draft** | **Last updated: 2026-06-22**

## 目的

この文書は、Phase 12「追加 carrier への拡張準備」が、文書上の整理だけでなく、実装と運用確認まで含めて完了したと言える条件をまとめます。

Phase 12 の文書面はすでに整えていますが、実装完了は別です。  
このチェックリストは、その差を埋めるための実行用メモです。

## 完了条件

Phase 12 を「実装その他を含めて完了」とみなすには、少なくとも次を満たします。

### 1. Capability の実装

- [ ] 新 carrier が `carrier kind` を名乗れる
- [ ] capability discovery で `protocol version` を返せる
- [ ] `supported object types` を返せる
- [ ] `supported schema versions` を返せる
- [ ] `supported auth modes` を返せる
- [ ] `supported content types` を返せる
- [ ] `validation constraints` と `finalize constraints` を返せる
- [ ] `supported access scopes` を返せる
- [ ] `supported retention hints` を返せる
- [ ] 必要なら `replay support` と `supported archive versions` を返せる

### 2. 共通 validation の実装

- [ ] `validate` が必須 field、schema version、carrier identity を確認する
- [ ] carrier 固有の framing と semantic validation を分離できている
- [ ] `normalize` が決定的である
- [ ] `finalize` が canonical view を壊さない
- [ ] `rawRef` と `provenance` を失わない
- [ ] capability 不足時に `fail closed` で拒否できる

### 3. Profile 差し替えの実装

- [ ] profile が carrier の違いを semantic model に持ち込まない
- [ ] Toitoi などの profile が carrier ごとの既定値を上書きできる
- [ ] profile validation が canonical view に対する追加制約として動く
- [ ] profile 側の語彙と carrier 側の語彙が衝突しない
- [ ] API 返却形が profile 契約として保たれている

### 4. Runbook / 運用確認

- [ ] [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md) から追加 carrier の確認順に辿れる
- [ ] `carrier kind` -> capability -> policy -> profile の順で確認できる
- [ ] 受け入れ可否を capability と policy から判定できる
- [ ] semantic translation ではなく拒否を優先する運用が説明できる
- [ ] 失敗時にどの文書を見るかが 1 本化されている

### 5. 実装検証

- [ ] 新 carrier の publish / retrieve / discover を 1 回通せる
- [ ] 既存 carrier と同じ object を扱って、semantic 差分が出ないことを確認した
- [ ] 互換境界の不足時に拒否されることを確認した
- [ ] profile 依存の差分が carrier 実装へ漏れていないことを確認した
- [ ] 運用手順を実際に 1 回たどった

### 6. ドキュメント整合

- [ ] [Carrier Capability Negotiation](../operations/CARRIER_CAPABILITY_NEGOTIATION.md) と実装が一致している
- [ ] [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md) と実装が一致している
- [ ] [File / Archive Carrier Contract](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md) と実装が一致している
- [ ] [Toitoi Application Profile](../profiles/TOITOI_APPLICATION_PROFILE.md) と実装が一致している
- [ ] [OPERATIONAL_READINESS_ROADMAP](./OPERATIONAL_READINESS_ROADMAP.md) の Phase 12 完了メモと一致している

## 参照

- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md)
- [運用準備バックログ](./OPERATIONAL_READINESS_BACKLOG.md)
- [Carrier Capability Negotiation](../operations/CARRIER_CAPABILITY_NEGOTIATION.md)
- [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md)
- [Toitoi Application Profile](../profiles/TOITOI_APPLICATION_PROFILE.md)
