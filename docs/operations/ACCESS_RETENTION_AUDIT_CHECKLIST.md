# Access and Retention Audit Checklist

**Status: draft** | **Last updated: 2026-06-20**

## 目的

この文書は、`access / retention policy` の変更や監査確認をするときの、最小の確認手順をまとめます。  
policy、carrier contract、runbook の 3 点を突き合わせるための実行用チェックリストです。

## 使いどころ

- access scope や retention hint の既定値を変えるとき
- export / import の scrub 方針を見直すとき
- backup / restore / retirement の保持対象を見直すとき
- authn/authz の追加で secret の注入経路を決めるとき

## チェックリスト

### 1. 運用方針

- [ ] [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md) の原則を確認した
- [ ] `public / curated / private` が protocol semantic ではないことを確認した
- [ ] `scrub` が protocol core の責務ではないことを確認した

### 2. Carrier 既定値

- [ ] [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md) の `supportedAccessScopes` を確認した
- [ ] [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md) の `supportedRetentionHints` を確認した
- [ ] [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md) の語彙と policy が一致していることを確認した
- [ ] [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md) の import 可否を確認した

### 3. Export / Import

- [ ] export 時の scrub 方針が manifest か別 policy 参照で説明できる
- [ ] import 可否が policy と capability で説明できる
- [ ] archive から replay できる条件を確認した
- [ ] backup / restore / retirement と export / import の責務を分けた

### 4. 保持対象

- [ ] raw log の保持方針を確認した
- [ ] canonical catalog の保持方針を確認した
- [ ] replay metadata の保持方針を確認した
- [ ] archive manifest の保持方針を確認した
- [ ] `tempDir` 配下の一時物を retention 対象に含めないことを確認した

### 5. 参照順

1. [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
2. [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
3. [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)
4. [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
5. [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
6. [Secret Management](./SECRET_MANAGEMENT.md)

## 関連

- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
