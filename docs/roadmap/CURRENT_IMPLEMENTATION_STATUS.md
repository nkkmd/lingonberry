# 現在の実装状況

**Status: active** | **Last updated: 2026-07-12**

この文書は、Lingonberry の実装作業を中断・再開するときの引き継ぎ用正本です。

長期計画は `IMPLEMENTATION_ROADMAP.md` と `OPERATIONAL_READINESS_ROADMAP.md`、issue 分解は各 backlog が扱います。この文書では、再開に必要な現在地、永続状態、安全性条件、運用入口を優先します。

---

## 1. 現在地

2026-07-12 時点で、ingress validation、acceptance policy、persistent quarantine、promotion、batch revalidation、status、metrics、scheduler、operator annotations、append-only manual dismissal lifecycle まで実装済みです。

```text
publish / archive import
        ↓
integrated validation
        ↓
acceptance policy
   ├─ Accept  → canonical storage
   ├─ Reject  → error
   └─ Defer   → quarantine.jsonl
                     ↓
              status / metrics
                     ↓
       manual or scheduled revalidation
                     ↓
     promoted / still deferred / rejected
                     ↓
        quarantine-resolutions.jsonl

operator review
   ├─ annotation → quarantine-annotations.jsonl
   └─ dismissal  → quarantine-dismissals.jsonl
                         ↓
              normal batch promotionから除外
```

| 項目 | 状態 |
|---|---|
| persistent quarantine store | 実装済み |
| single-record revalidation / promotion | 実装済み |
| batch revalidation / dry-run | 実装済み |
| status CLI / HTTP API | 実装済み |
| Prometheus metrics | 実装済み |
| systemd timer / cron fallback | 文書・template 実装済み |
| append-only operator annotations | 実装済み |
| append-only manual dismissal | 実装済み |
| permanently rejected lifecycle | 未実装 |
| admin authentication / authorization | 未実装 |
| retention / compaction / rotation | 未実装 |
| distributed locking | 未実装 |

---

## 2. Quarantine lifecycle

### 2.1 Persistent states

```text
pending
promoted
dismissed
```

- `pending`: promotion resolutionもdismissal eventもないrecord
- `promoted`: 有効なpromotion resolutionがあるrecord
- `dismissed`: 有効なappend-only dismissal eventがあるrecord

### 2.2 Transient revalidation decisions

```text
accept
still-deferred
rejected
```

`rejected`は恒久状態ではありません。`permanently-rejected` lifecycleは未実装です。

### 2.3 Operator metadata

```text
operator annotation
```

annotationはlifecycle stateではなく、promotion eligibilityを変更しません。

---

## 3. Manual dismissalの固定仕様

QL-1では次を固定しました。

```text
対象: pending recordのみ
重複: 1 record 1 active dismissalとしてidempotent
undo / reopen: 非スコープ
理由: bounded reasonCode + required operator note
入口: Core + CLI
HTTP mutation API: admin auth設計まで非スコープ
```

永続event schema：

```json
{
  "id": "lb:qd:...",
  "quarantineId": "lb:q:...",
  "dismissedAt": "...Z",
  "operator": "operator-name",
  "reasonCode": "LB_OPERATOR_DISMISSED",
  "note": "duplicate external submission"
}
```

固定条件：

- unknown quarantine IDを拒否する
- promoted recordのdismissalを拒否する
- duplicate dismissal requestは既存eventを返す
- duplicate dismissal eventがledgerに存在する場合はcorruptionとして失敗する
- 元のquarantine recordとannotationを変更・削除しない
- dismissed recordを通常のquarantine listとbatch promotion scanから除外する
- statusの`pending`はpromotedとdismissedを除外する
- corruption / I/O errorを黙って無視しない

関連文書：`docs/operations/QUARANTINE_DISMISSALS.md`

関連Issue：#22

---

## 4. Runtime stateのquarantine関連ファイル

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
```

### `quarantine.jsonl`

- append-only
- deferされた元publish request
- canonical catalogとは分離
- 監査用原本

### `quarantine-resolutions.jsonl`

- append-only
- promotion成功を永続化
- `quarantineId`と`canonicalId`を対応付け
- promotion idempotencyの基礎

### `quarantine-annotations.jsonl`

- append-only
- operatorの自由文監査event
- lifecycle stateではない
- promotion / scheduler対象判定には使用しない

### `quarantine-dismissals.jsonl`

- append-only lifecycle event
- pending recordを通常のpromotion対象から外す
- 1 quarantine recordにつき1 active dismissal
- undo / reopenは未実装

すべてのledgerで、corruptionとI/O errorを黙って無視しません。

---

## 5. 現在利用できるquarantine CLI

```bash
cargo run -p lingonberry-relay -- quarantine-list
cargo run -p lingonberry-relay -- quarantine-get <quarantine-id>
cargo run -p lingonberry-relay -- quarantine-promote <quarantine-id>
cargo run -p lingonberry-relay -- quarantine-promote-batch 100 --dry-run
cargo run -p lingonberry-relay -- quarantine-resolutions
cargo run -p lingonberry-relay -- quarantine-status
cargo run -p lingonberry-relay -- quarantine-metrics
cargo run -p lingonberry-relay -- quarantine-annotate <quarantine-id> <operator> <note>
cargo run -p lingonberry-relay -- quarantine-annotations [quarantine-id]
cargo run -p lingonberry-relay -- quarantine-dismiss <quarantine-id> <operator> <note>
cargo run -p lingonberry-relay -- quarantine-dismissals [quarantine-id]
```

`quarantine-dismiss`のbounded reason codeはCLI側で`LB_OPERATOR_DISMISSED`に固定します。

---

## 6. 現在利用できるquarantine HTTP surface

```text
GET  /v1/quarantine
GET  /v1/quarantine/<quarantine-id>
POST /v1/quarantine/<quarantine-id>/promote
POST /v1/quarantine/promote-batch
GET  /v1/quarantine-resolutions
GET  /v1/quarantine-status
GET  /metrics
POST /v1/quarantine/<quarantine-id>/annotations
GET  /v1/quarantine/<quarantine-id>/annotations
```

manual dismissalのHTTP mutation endpointは追加していません。authentication / authorizationが未実装のため、管理endpointを一般公開しない構成を優先します。

---

## 7. Statusとmetrics

`quarantine-status`と`GET /v1/quarantine-status`の主要field：

```text
total
pending
promoted
dismissed
oldestPendingAt
latestReceivedAt
latestPromotedAt
latestDismissedAt
reasonCodeCounts
```

Prometheus metrics：

```text
lingonberry_quarantine_records{state="total"}
lingonberry_quarantine_records{state="pending"}
lingonberry_quarantine_records{state="promoted"}
lingonberry_quarantine_records{state="dismissed"}
lingonberry_quarantine_oldest_pending_age_seconds
lingonberry_quarantine_reason_code_records{reason_code="..."}
```

高カーディナリティなquarantine ID、canonical ID、operator、自由文noteはlabelに使用しません。

---

## 8. 現在の主要ファイル

```text
docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md
docs/roadmap/QUARANTINE_LIFECYCLE_BACKLOG.md
docs/operations/QUARANTINE_DISMISSALS.md
packages/validation/src/policy.rs
packages/core/src/quarantine.rs
packages/core/src/quarantine_status.rs
packages/core/src/quarantine_annotations.rs
packages/core/src/quarantine_dismissals.rs
packages/core/src/lib.rs
packages/core/src/lib_entry.rs
packages/relay/src/main.rs
packages/relay/src/main_entry.rs
```

---

## 9. 再開時の確認手順

```bash
git switch main
git pull --ff-only
git status
cargo test --workspace
```

JavaScript testはCIと同じ入口を優先します。

Read-only / dry-run確認：

```bash
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- quarantine-status
cargo run -p lingonberry-relay -- quarantine-metrics
cargo run -p lingonberry-relay -- quarantine-annotations
cargo run -p lingonberry-relay -- quarantine-dismissals
cargo run -p lingonberry-relay -- quarantine-promote-batch 100 --dry-run
```

---

## 10. 絶対に崩さない安全性ルール

1. validationを通過していないobjectをcanonical storageに保存しない
2. unsupported identity ruleを通常のmismatchと混同しない
3. `defer`されたobjectはquarantineにのみ保存する
4. promotion前に現在のvalidatorとacceptance policyで再評価する
5. rejected / still-deferred recordをcanonical storageへ移さない
6. promotionの再実行はidempotentに扱う
7. 元のquarantine recordを監査証跡として保持する
8. dry-runでは永続データを変更しない
9. canonical storageとquarantine storageを分離する
10. corruptionとI/O errorを黙って無視しない
11. annotationをlifecycle stateとして解釈しない
12. annotationの文言でscheduler対象を制御しない
13. dismissalは物理削除ではなく専用append-only eventで表現する
14. dismissed recordを通常のbatch promotion対象へ戻さない
15. status / metricsはledgerを変更しない
16. 管理HTTP endpointを認証なしで一般公開しない

---

## 11. 未解決事項と次の推奨作業

### 第一候補

```text
QL-2: Admin Authentication and Network Isolation
```

quarantine参照・promotion・annotation・将来のdismissal HTTP APIを公開relayの一般surfaceから分離します。

### 第二候補

```text
QL-3: Persistent Rejection Decisions
```

transientな`rejected`と恒久的なlifecycle stateを分離します。

### 第三候補

```text
QL-4 / QL-5: Backup, export, restore, index, rotation, compaction
```

append-only監査証跡を維持しながら長期運用を可能にします。

### 継続課題

- concurrent promotion / ledger append
- multi-host distributed locking
- HTTP authentication / authorization
- JSONL growth management
- backup / restore boundary
- schema migration

---

## 12. 実装順序

```text
#7 acceptance policy
→ #8 persistent quarantine
→ #9 promotion
→ #10 batch promotion
→ #13 status
→ #15 metrics
→ #17 scheduler
→ #19 annotations
→ #22 manual dismissal
```

この文書とbacklogをquarantine関連PRの完了条件として更新します。
