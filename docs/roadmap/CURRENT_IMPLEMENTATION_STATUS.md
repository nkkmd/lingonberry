# 現在の実装状況

**Status: paused / resumable** | **Last updated: 2026-07-12**

この文書は、Lingonberry の実装作業を中断・再開するときの引き継ぎ用正本です。

長期計画は `IMPLEMENTATION_ROADMAP.md` と `OPERATIONAL_READINESS_ROADMAP.md`、issue 分解は各 backlog が扱います。この文書では、次回の作業再開に必要な現在地だけを優先します。

- どこまで実装済みか
- 直近で何を追加したか
- runtime state に何が保存されるか
- どの CLI / HTTP API が利用できるか
- 何を安全性の固定条件とするか
- 未解決事項と次の推奨作業は何か
- 再開直後に何を確認すべきか

---

## 1. 休止時点の要約

2026-07-12 時点で、ingress validation から quarantine の定期再評価、状態監視、運用 annotation まで実装済みです。

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
        ↓
quarantine-annotations.jsonl
```

現在の quarantine 運用面は次の状態です。

| 項目 | 状態 |
|---|---|
| persistent quarantine store | 実装済み |
| single-record revalidation / promotion | 実装済み |
| batch revalidation / dry-run | 実装済み |
| status CLI / HTTP API | 実装済み |
| Prometheus metrics | 実装済み |
| systemd timer / cron fallback | 文書・template 実装済み |
| append-only operator annotations | 実装済み |
| manual dismissal | 未実装 |
| permanently rejected lifecycle | 未実装 |
| admin authentication / authorization | 未実装 |
| retention / compaction / rotation | 未実装 |
| distributed locking | 未実装 |

---

## 2. 完了済みの主要実装

### 2.1 Canonicalization v1

- `lb.canonical.json.v1` を仕様化
- Rust / JavaScript の共通 fixture と conformance test を追加
- object key ordering、array preservation、escaping、whitespace 規則を固定

関連 PR：#2

### 2.2 Identity Key v2 と version-aware validation

- SHA-256 ベースの `lb.identity.key.v2`
- v1 互換性
- rule version ごとの identity claim validation
- unsupported rule と mismatch を区別

関連 PR：#3、#4

### 2.3 Integrated Validation Facade

- protocol schema validation と identity validation を統合
- `valid / invalid / unsupported / not-present` を区別
- CLI、HTTP publish、archive import が同じ validation path を使用

関連 PR：#5、#6

### 2.4 Configurable Acceptance Policy

環境変数：

```bash
LINGONBERRY_REQUIRE_IDENTITY_CLAIM
LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY
```

`LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY`：

```text
reject  unsupported identity rule を拒否
defer   canonical storage に入れず quarantine へ保存
```

関連文書：`docs/operations/ACCEPTANCE_POLICY.md`

関連 PR：#7

### 2.5 Persistent Quarantine Store

`defer`された publish request は append-only JSONL に保存されます。

```text
<state-dir>/quarantine.jsonl
```

代表 schema：

```json
{
  "id": "lb:q:...",
  "receivedAt": "...Z",
  "reasonCode": "LB_IDENTITY_DEFERRED",
  "reasons": ["..."],
  "requestJson": "{...}"
}
```

未検証 object は canonical catalog へ混入しません。

関連 PR：#8

### 2.6 Revalidation and Promotion

CLI：

```bash
cargo run -p lingonberry-relay -- quarantine-promote <quarantine-id>
cargo run -p lingonberry-relay -- quarantine-resolutions
```

HTTP：

```text
POST /v1/quarantine/<quarantine-id>/promote
GET  /v1/quarantine-resolutions
```

判定結果：

```text
promoted
already-promoted
still-deferred
rejected
```

promotion 成功時のみ、次の ledger に永続 resolution を追記します。

```text
<state-dir>/quarantine-resolutions.jsonl
```

元の quarantine record は削除しません。

関連 PR：#9

### 2.7 Batch Revalidation and Promotion

CLI：

```bash
cargo run -p lingonberry-relay -- quarantine-promote-batch [limit] [--dry-run]
```

HTTP：

```text
POST /v1/quarantine/promote-batch
```

既定値：

```text
limit: 100
maximum: 1000
```

`--dry-run` / `dryRun: true` は validation と policy evaluation のみを行い、canonical storage と resolution ledger を変更しません。

関連 PR：#10

### 2.8 Quarantine Status API

永続 ledger から現在状態を再構成します。

CLI：

```bash
cargo run -p lingonberry-relay -- quarantine-status
```

HTTP：

```text
GET /v1/quarantine-status
```

response fields：

```text
total
pending
promoted
oldestPendingAt
latestReceivedAt
latestPromotedAt
reasonCodeCounts
```

重要な意味論：

- `promoted` は resolution ledger に永続化された lifecycle state
- `pending` は有効な promotion resolution が存在しない record
- duplicate resolution は 1 件として集計
- quarantine record に対応しない unknown resolution は除外
- `deferred` と `rejected` は現在の再評価結果であり、累積 lifecycle count ではない
- corrupt JSONL / I/O error は 0 件として偽装せず明示的に失敗

関連文書：`docs/roadmap/QUARANTINE_STATUS_API.md`

関連 PR：#13

### 2.9 Quarantine Observability Metrics

CLI：

```bash
cargo run -p lingonberry-relay -- quarantine-metrics
```

HTTP：

```text
GET /metrics
```

Prometheus metrics：

```text
lingonberry_quarantine_records{state="total"}
lingonberry_quarantine_records{state="pending"}
lingonberry_quarantine_records{state="promoted"}
lingonberry_quarantine_oldest_pending_age_seconds
lingonberry_quarantine_reason_code_records{reason_code="..."}
```

高カーディナリティな quarantine ID、canonical ID、request ID、自由文 reason / note は label に使用しません。

関連文書：`docs/operations/QUARANTINE_OBSERVABILITY_METRICS.md`

関連 PR：#15

### 2.10 Quarantine Scheduler Integration

定期実行の正本は CLI です。

```bash
/usr/local/bin/lingonberry-relay quarantine-promote-batch 100
```

systemd template：

```text
deploy/systemd/lingonberry-quarantine-promote.service
deploy/systemd/lingonberry-quarantine-promote.timer
```

既定値：

```text
interval: 15 minutes
batch limit: 100
randomized delay: up to 60 seconds
Persistent=true
```

同一 host 上の重複実行は `flock` で抑止します。distributed lock は保証しません。

関連文書：`docs/operations/QUARANTINE_SCHEDULER.md`

関連 PR：#17

### 2.11 Quarantine Operator Annotations

運用上の確認事項を、元 record を変更せず append-only event として記録します。

永続ファイル：

```text
<state-dir>/quarantine-annotations.jsonl
```

CLI：

```bash
cargo run -p lingonberry-relay -- quarantine-annotate <quarantine-id> <operator> <note>
cargo run -p lingonberry-relay -- quarantine-annotations [quarantine-id]
```

HTTP：

```text
POST /v1/quarantine/<quarantine-id>/annotations
GET  /v1/quarantine/<quarantine-id>/annotations
```

annotation schema：

```json
{
  "id": "lb:qa:...",
  "quarantineId": "lb:q:...",
  "annotatedAt": "...Z",
  "operator": "operator-name",
  "note": "reviewed source material"
}
```

固定条件：

- annotation は lifecycle state ではない
- annotation は promotion eligibility を変更しない
- annotation の更新・削除 API はない
- 訂正は新しい annotation を追記する
- unknown quarantine ID、空 operator、空 note は拒否

関連文書：`docs/operations/QUARANTINE_ANNOTATIONS.md`

関連 PR：#19

---

## 3. 現在の主要ファイル

### 3.1 最初に読む文書

```text
docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md
docs/roadmap/README.md
docs/roadmap/IMPLEMENTATION_BACKLOG.md
docs/roadmap/QUARANTINE_STATUS_API.md
docs/operations/ACCEPTANCE_POLICY.md
docs/operations/QUARANTINE_OBSERVABILITY_METRICS.md
docs/operations/QUARANTINE_SCHEDULER.md
docs/operations/QUARANTINE_ANNOTATIONS.md
docs/operations/OBSERVABILITY.md
```

### 3.2 実装コード

```text
packages/validation/src/policy.rs
packages/core/src/quarantine.rs
packages/core/src/quarantine_status.rs
packages/core/src/quarantine_annotations.rs
packages/core/src/lib.rs
packages/core/src/lib_entry.rs
packages/relay/src/main.rs
packages/relay/src/main_entry.rs
```

| ファイル | 主な責務 |
|---|---|
| `policy.rs` | ingress acceptance decision |
| `quarantine.rs` | quarantine / promotion resolution ledger |
| `quarantine_status.rs` | status 集計と Prometheus text snapshot |
| `quarantine_annotations.rs` | append-only operator annotation ledger |
| `lib.rs` | promotion / batch revalidation |
| `lib_entry.rs` | Core extension module の接続 |
| `main.rs` |既存 CLI / HTTP 実装 |
| `main_entry.rs` | status、metrics、annotations の追加接続 |

### 3.3 運用 template

```text
deploy/systemd/lingonberry-quarantine-promote.service
deploy/systemd/lingonberry-quarantine-promote.timer
```

---

## 4. Runtime state の quarantine 関連ファイル

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
```

### `quarantine.jsonl`

- append-only
- defer された元 publish request
- canonical catalog とは分離
- 監査用原本

### `quarantine-resolutions.jsonl`

- append-only
- 現在は promotion 成功だけを永続化
- `quarantineId` と `canonicalId` を対応付け
- promotion idempotency の基礎

### `quarantine-annotations.jsonl`

- append-only
- operator の自由文監査 event
- lifecycle state ではない
- promotion / scheduler の対象判定には使用しない

すべての ledger で、corruption と I/O error を黙って無視しません。

---

## 5. 現在利用できる quarantine CLI

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
```

注：実行 binary 名は Cargo package 上では `lingonberry-relay` です。インストール済み binary を直接実行する運用文書では `/usr/local/bin/lingonberry-relay` を使用します。

---

## 6. 現在利用できる quarantine HTTP surface

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

これらは運用管理 API を含みます。authentication / authorization が未実装のため、一般公開しない構成を優先します。

---

## 7. 再開時の確認手順

### 7.1 main 更新

```bash
git switch main
git pull --ff-only
```

### 7.2 repository 状態確認

```bash
git status
```

未追跡・未commitの変更がないことを確認します。

### 7.3 Rust test

```bash
cargo test --workspace
```

### 7.4 JavaScript test

CI と同じ入口を優先します。現在の workflow を確認した上で、少なくとも次を実行します。

```bash
node --test \
  packages/codecs/test/canonicalization.test.mjs \
  packages/identity/test/identity-key-v2.test.mjs \
  packages/identity/test/identity-claim-validator.test.mjs \
  packages/validation/test/validation.test.mjs
```

### 7.5 Read-only / dry-run確認

```bash
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- quarantine-status
cargo run -p lingonberry-relay -- quarantine-metrics
cargo run -p lingonberry-relay -- quarantine-annotations
cargo run -p lingonberry-relay -- quarantine-promote-batch 100 --dry-run
```

### 7.6 HTTP確認

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

別terminal：

```bash
curl -sS http://127.0.0.1:8787/v1/ready
curl -sS http://127.0.0.1:8787/v1/quarantine-status
curl -sS http://127.0.0.1:8787/metrics
```

### 7.7 scheduler確認

```bash
systemd-analyze verify deploy/systemd/lingonberry-quarantine-promote.service
systemd-analyze verify deploy/systemd/lingonberry-quarantine-promote.timer
```

実環境へ配置済みの場合：

```bash
systemctl status lingonberry-quarantine-promote.timer
systemctl list-timers lingonberry-quarantine-promote.timer
journalctl -u lingonberry-quarantine-promote.service -n 100
```

---

## 8. 絶対に崩さない安全性ルール

1. validation を通過していない object を canonical storage に保存しない
2. unsupported identity rule を通常の mismatch と混同しない
3. `defer`された object は quarantine にのみ保存する
4. promotion 前に現在の validator と acceptance policy で再評価する
5. rejected / still-deferred record を canonical storage に移さない
6. promotion の再実行は idempotent に扱う
7. 元の quarantine record を監査証跡として保持する
8. dry-run では永続データを変更しない
9. canonical storage と quarantine storage を分離する
10. corruption と I/O error を黙って無視しない
11. annotation を lifecycle state として解釈しない
12. annotation の文言で scheduler 対象を制御しない
13. manual dismissal を追加する場合も物理削除しない
14. status / metrics は ledger を変更しない
15.管理 HTTP endpoint を認証なしで一般公開しない

---

## 9. 永続状態と一時的判定の区別

現在の persistent lifecycle state：

```text
pending
promoted
```

現在の transient revalidation decision：

```text
accept
still-deferred
rejected
```

運用 metadata：

```text
operator annotation
```

`rejected`は現時点では恒久状態として保存されません。将来 `permanently-rejected` や `dismissed` を追加する場合は、annotation text ではなく専用の append-only lifecycle event ledger を設計します。

---

## 10. 未解決の設計事項

### 10.1 Manual Dismissal

次の推奨実装候補です。

要件：

- append-only lifecycle event
- 元recordを削除しない
- operator / reason / timestampを保持
- idempotencyまたは重複eventの意味を固定
- dismissed recordをbatch対象から除外
- status / metricsへ`dismissed`を追加するか明示
- promotion済みrecordをdismissできるか決定
- dismissal取消を許すなら新eventとして表現

annotationの特定文言をdismissalとして解釈してはいけません。

### 10.2 Permanently Rejected Lifecycle

現在の`rejected`は再評価時のdecisionです。恒久状態にする条件、operator介入、policy変更後の再開可否を設計する必要があります。

### 10.3 Concurrent Promotion / Concurrent Ledger Append

- 複数processによる同一recordのpromotion
- resolution ledgerへの同時append
- annotation ledgerへの同時append
- schedulerと手動操作の競合

同一host schedulerは`flock`で抑止しますが、Core全体のdistributed lockingではありません。

### 10.4 HTTP Administration Security

未実装：

- authentication
- authorization
- network exposure policy
- audit logging
- rate limiting
- CSRF相当の管理操作保護

### 10.5 JSONL Growth

対象：

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
```

今後の候補：

- index
- archive export
- backup / restore
- rotation
- compaction
- retention
- corruption recovery
- schema migration

append-only監査証跡を維持したまま設計する必要があります。

### 10.6 Current Implementation Document Drift

この文書とbacklogが実装より古くならないよう、今後はquarantine関連PRの完了条件に次を含めることを推奨します。

```text
CURRENT_IMPLEMENTATION_STATUS.md を更新する、または更新不要の理由をPR本文に記載する
```

---

## 11. 関連PRと実装順序

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
| #13 | Quarantine Status API |
| #15 | Quarantine observability metrics |
| #17 | Quarantine scheduler integration |
| #19 | Append-only operator annotations |

quarantine設計の意図を追う場合は、#7 → #8 → #9 → #10 → #13 → #15 → #17 → #19 の順に確認します。

---

## 12. 次回再開時の推奨開始点

### 第一候補

```text
Append-only Manual Dismissal Lifecycle
```

最初にIssueへ書くべき最小スコープ：

```text
- dismissal event schema
- append-only dismissal ledger
- CLI append/list
- batch promotionからdismissedを除外
- status JSONへのdismissed count
- metricsへのdismissed gauge
- unknown ID / duplicate / promoted recordの扱い
- tests
- operations document
```

実装前に必ず決める項目：

1. promotion済みrecordのdismissalを拒否するか
2. 同じrecordへの複数dismissalをidempotentにするか
3. dismissal取消を初期スコープに含めるか
4. dismissal reasonを自由文だけにするか、bounded reason codeを持たせるか
5. HTTP管理endpointを同時に追加するか、CLI先行にするか

推奨初期判断：

```text
- pending recordのみdismiss可能
- 1 record 1 active dismissal
- undoは非スコープ
- bounded reasonCode + operator note
- CLIとCoreを先行し、HTTPはadmin auth設計と分離してもよい
```

### 第二候補

```text
Quarantine Admin Authentication / Network Isolation
```

公開relayとしての安全性を優先する場合はこちらを先に進めます。

### 第三候補

```text
Quarantine Backup / Export and JSONL Growth Strategy
```

長期運用・データ保全を優先する場合はこちらです。

---

## 13. 休止状態

休止時点では、PR #19 まで main へ squash merge 済みです。

最後に確認されたmain commit：

```text
4ae8b313a2918800d6d61937e44bedcd532347c9
```

次回はこの文書、`IMPLEMENTATION_BACKLOG.md`、関連operations文書を読み、mainの最新commitと差異がないことを確認してから新しいIssueを作成します。
