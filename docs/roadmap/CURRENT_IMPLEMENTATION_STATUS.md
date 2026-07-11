# 現在の実装状況

**Status: active** | **Last updated: 2026-07-11**

この文書は、Lingonberry の実装作業を中断・再開するときの引き継ぎ用正本です。

実装ロードマップや運用準備ロードマップが中長期の計画を扱うのに対し、この文書は次を短く確認するために使います。

- 現在どこまで実装済みか
- 直近で追加された機能は何か
- 再開時に読むべき文書とコードはどれか
- 次に着手する作業候補は何か
- 再開前に実行すべき確認コマンドは何か

---

## 1. 現在の到達点

2026-07-11 時点で、Lingonberry の ingress validation と quarantine 運用フローは、次の範囲まで実装済みです。

```text
publish / archive import
        ↓
integrated validation
        ↓
acceptance policy
   ├─ Accept  → canonical storage
   ├─ Reject  → error
   └─ Defer   → quarantine storage
                     ↓
              revalidation
                     ↓
            promotion / remain pending
```

現在は、単体 publish の検証だけでなく、unsupported identity rule を安全に保留し、後から再検証して canonical storage へ昇格できる状態です。

さらに、未解決 record を件数制限付きで一括再評価する機能と、書き込みを行わない dry-run も実装されています。

---

## 2. 完了済みの主要機能

### 2.1 Canonicalization v1

- `lb.canonical.json.v1` を仕様化
- Rust / JavaScript 共通 fixture を追加
- cross-language conformance test を追加
- object key ordering、array preservation、escaping、whitespace 規則を固定

関連文書：

```text
docs/protocols/CANONICALIZATION.md
```

関連 PR：

- PR #2 `Define canonicalization v1 and add conformance tests`

### 2.2 Identity Key v2

- `lb.identity.key.v2` を追加
- SHA-256 ベースの identity key を実装
- Rust / JavaScript の参照実装を追加
- v1 互換性を維持
- rule version ごとの identity claim validation を追加

関連コード：

```text
packages/identity/
```

関連 PR：

- PR #3 `Add SHA-256 identity key v2`
- PR #4 `Validate identity claims by rule version`

### 2.3 Integrated Validation Facade

- protocol schema validation と identity validation を統合
- `valid / invalid / unsupported / not-present` を区別
- Rust / JavaScript の facade API を追加
- CLI、HTTP publish、archive import から同じ validation path を使用

関連コード：

```text
packages/validation/
```

関連 PR：

- PR #5 `Add integrated validation facade`
- PR #6 `Enforce integrated validation at ingress`

### 2.4 Configurable Acceptance Policy

環境変数による ingress policy を実装済みです。

```bash
LINGONBERRY_REQUIRE_IDENTITY_CLAIM
LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY
```

`LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY` の値：

```text
reject
  unsupported identity rule を拒否する

defer
  canonical storage には入れず quarantine へ保存する
```

関連文書：

```text
docs/operations/ACCEPTANCE_POLICY.md
```

関連コード：

```text
packages/validation/src/policy.rs
```

関連 PR：

- PR #7 `Add configurable ingress acceptance policy`

### 2.5 Persistent Quarantine Store

`defer`された publish request は、次の append-only JSONL に保存されます。

```text
<state-dir>/quarantine.jsonl
```

quarantine record は、少なくとも次を保持します。

```json
{
  "id": "lb:q:...",
  "receivedAt": "...",
  "reasonCode": "LB_IDENTITY_DEFERRED",
  "reasons": ["..."],
  "requestJson": "{...}"
}
```

canonical catalog とは分離されており、未検証 object が canonical storage に混入しない設計です。

関連コード：

```text
packages/core/src/quarantine.rs
```

関連 PR：

- PR #8 `Add persistent quarantine store`

### 2.6 Revalidation and Promotion

quarantine record を、現在の validator と acceptance policy で再検証できます。

CLI：

```bash
lingonberry quarantine-promote <quarantine-id>
lingonberry quarantine-resolutions
```

HTTP：

```text
POST /v1/quarantine/<quarantine-id>/promote
GET /v1/quarantine-resolutions
```

結果は次のいずれかです。

```text
promoted
already-promoted
deferred
rejected
```

昇格履歴は次に保存されます。

```text
<state-dir>/quarantine-resolutions.jsonl
```

元の quarantine record は削除せず、監査証跡として保持します。

関連 PR：

- PR #9 `Add quarantine revalidation and promotion`

### 2.7 Batch Revalidation and Promotion

未解決 quarantine record を、件数制限付きで一括処理できます。

CLI：

```bash
lingonberry quarantine-promote-batch [limit] [--dry-run]
```

HTTP：

```text
POST /v1/quarantine/promote-batch
```

デフォルト：

```text
limit: 100
maximum: 1000
```

`--dry-run`または`dryRun: true`の場合は、validation と policy evaluation のみを実行し、次には書き込みません。

```text
canonical storage
quarantine-resolutions.jsonl
```

既に resolution が存在する record は自動的に除外されます。

関連 PR：

- PR #10 `Add quarantine batch revalidation and promotion`

---

## 3. 現在の主要ファイル

再開時は、次の順で読むと実装の流れを追いやすくなります。

### 3.1 仕様と運用

```text
README.md
docs/architecture/README.md
docs/architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md
docs/roadmap/README.md
docs/roadmap/IMPLEMENTATION_ROADMAP.md
docs/roadmap/IMPLEMENTATION_BACKLOG.md
docs/operations/README.md
docs/operations/ACCEPTANCE_POLICY.md
```

### 3.2 実装

```text
packages/validation/src/policy.rs
packages/core/src/quarantine.rs
packages/core/src/lib.rs
packages/relay/src/main.rs
```

役割：

| ファイル | 主な責務 |
|---|---|
| `packages/validation/src/policy.rs` | ingress acceptance decision |
| `packages/core/src/quarantine.rs` | quarantine / resolution ledger の永続化 |
| `packages/core/src/lib.rs` | 単体・一括再検証、promotion、archive integration |
| `packages/relay/src/main.rs` | CLI と HTTP API |

---

## 4. 現在のデータファイル

runtime state directory には、少なくとも次の quarantine 関連ファイルが存在します。

```text
quarantine.jsonl
quarantine-resolutions.jsonl
```

### `quarantine.jsonl`

- append-only
- defer された元 publish request を保存
- canonical catalog には含まれない
- 監査用の原本として保持

### `quarantine-resolutions.jsonl`

- append-only
- promotion 成功結果を保存
- `quarantineId`と`canonicalId`を対応付ける
- 二重 promotion を防止するための idempotency ledger として使用

---

## 5. 再開時の確認手順

### 5.1 main を更新

```bash
git switch main
git pull
```

### 5.2 Rust test

```bash
cargo test --workspace
```

### 5.3 JavaScript test

```bash
node --test \
  packages/codecs/test/canonicalization.test.mjs \
  packages/identity/test/identity-key-v2.test.mjs \
  packages/identity/test/identity-claim-validator.test.mjs \
  packages/validation/test/validation.test.mjs
```

### 5.4 基本動作確認

```bash
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- quarantine-list
cargo run -p lingonberry-relay -- quarantine-resolutions
cargo run -p lingonberry-relay -- quarantine-promote-batch --dry-run
```

HTTP server を確認する場合：

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

代表的な endpoint：

```text
GET  /v1/ready
GET  /v1/capabilities
GET  /v1/quarantine
GET  /v1/quarantine/<quarantine-id>
POST /v1/quarantine/<quarantine-id>/promote
POST /v1/quarantine/promote-batch
GET  /v1/quarantine-resolutions
```

---

## 6. 現在の安全性ルール

次のルールを崩さないことが重要です。

1. validation を通過していない object を canonical storage に保存しない
2. unsupported identity rule を通常の mismatch と混同しない
3. `defer`された object は quarantine にのみ保存する
4. promotion 前に必ず現在の validator と policy で再評価する
5. rejected / deferred record は canonical storage に移さない
6. promotion の再実行は idempotent に扱う
7. 元の quarantine record は監査証跡として保持する
8. dry-run では永続データを変更しない
9. canonical storage と quarantine storage を明確に分離する
10. file corruption や I/O error を黙って無視しない

---

## 7. 次の作業候補

休止時点での次の自然な作業は、quarantine の状態管理と運用監視です。

### 7.1 Quarantine Status API

次の集計を返す CLI / HTTP API を追加する候補です。

```text
pending count
promoted count
deferred count
rejected count
oldest pending record
latest received timestamp
latest promotion timestamp
reasonCode breakdown
```

候補 CLI：

```bash
lingonberry quarantine-status
```

候補 HTTP：

```text
GET /v1/quarantine-status
```

### 7.2 Observability Integration

`docs/operations/OBSERVABILITY.md`と整合させながら、次を検討します。

- quarantine backlog size
- promotion success count
- promotion rejection count
- deferred record age
- batch run duration
- corrupt record count
- quarantine I/O error count

### 7.3 Scheduler Integration

一括再評価を定期実行する方法を運用文書へ追加します。

候補：

```text
systemd timer
cron
container scheduler
external HTTP scheduler
```

最初は CLI の定期実行を正本とし、HTTP 経由の scheduler は認証・公開範囲を整理した後に扱う方が安全です。

### 7.4 Quarantine Record Lifecycle

将来的な検討事項：

- permanently rejected record の分類
- operator annotation
- manual dismissal
- export / backup
- compaction
- retention policy
- migration / schema versioning

削除機能を追加する場合も、監査証跡と append-only 原則を壊さない設計が必要です。

---

## 8. 未解決の設計上の注意点

### 8.1 Status と永続状態

現状では、`promoted`のみ resolution ledger に永続記録されます。

`deferred`と`rejected`は再評価時の判定結果であり、恒久的な状態としては保存されません。そのため、status API を設計するときは次を区別する必要があります。

```text
last observed decision
persistent lifecycle state
current validation result
```

### 8.2 Concurrent Promotion

複数 process が同じ quarantine record を同時に昇格しようとする場合の排他制御は、今後の確認事項です。

canonical storage の duplicate handling により結果は一定程度保護されますが、resolution ledger への同時 append や二重 resolution の扱いを明示的にテストする必要があります。

### 8.3 HTTP Administration Surface

quarantine の参照・promotion endpoint は運用管理 API に相当します。

公開 relay で使用する場合は、次を整理する必要があります。

- authentication
- authorization
- network exposure
- audit logging
- rate limiting

Caddy などの reverse proxy を使う場合も、管理 endpoint を一般公開しない構成を優先します。

### 8.4 JSONL Growth

`quarantine.jsonl`と`quarantine-resolutions.jsonl`は append-only のため、長期運用ではファイルが増加します。

今後、次を検討します。

- index
- compaction
- rotation
- archive export
- backup / restore
- corruption recovery

---

## 9. 関連 PR

直近の実装履歴：

| PR | 内容 |
|---|---|
| #2 | Canonicalization v1 |
| #3 | Identity Key v2 |
| #4 | Version-aware identity validation |
| #5 | Integrated validation facade |
| #6 | Ingress validation enforcement |
| #7 | Configurable acceptance policy |
| #8 | Persistent quarantine store |
| #9 | Revalidation and promotion |
| #10 | Batch revalidation and promotion |

再開時に実装意図まで確認する必要がある場合は、PR #7 から #10 を順に読むと quarantine 関連の設計判断を追いやすくなります。

---

## 10. 再開時の推奨開始点

再開時は、まず次を実行します。

```text
1. この文書を読む
2. docs/operations/ACCEPTANCE_POLICY.md を読む
3. main を更新する
4. 全 test を実行する
5. quarantine-promote-batch --dry-run を実行する
6. docs/roadmap/IMPLEMENTATION_BACKLOG.md を更新する
7. quarantine status / observability の設計から再開する
```

次に着手する実装候補の第一案：

```text
Quarantine Status API and Metrics
```

想定する最小スコープ：

```text
CLI:  quarantine-status
HTTP: GET /v1/quarantine-status

fields:
  total
  pending
  promoted
  oldestPendingAt
  latestReceivedAt
  latestPromotedAt
  reasonCodeCounts
```

この最小スコープを実装した後、observability、scheduler、retention、admin authentication の順に進めると整理しやすいです。
