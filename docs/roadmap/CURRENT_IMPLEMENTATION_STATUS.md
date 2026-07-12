# 現在の実装状況

**Status: active** | **Last updated: 2026-07-12**

この文書は、Lingonberry の実装作業を中断・再開するときの引き継ぎ用正本です。

長期計画は `IMPLEMENTATION_ROADMAP.md` と `OPERATIONAL_READINESS_ROADMAP.md`、issue 分解は各 backlog が扱います。この文書では、再開に必要な現在地、永続状態、安全性条件、運用入口を優先します。

---

## 1. 現在地

2026-07-12 時点で、ingress validation、acceptance policy、persistent quarantine、promotion、batch revalidation、status、metrics、scheduler、operator annotations、manual dismissal、quarantine admin HTTP isolationまで実装済みです。

| 項目 | 状態 |
|---|---|
| persistent quarantine store | 実装済み |
| single-record revalidation / promotion | 実装済み |
| batch revalidation / dry-run | 実装済み |
| status CLI / admin HTTP API | 実装済み |
| Prometheus metrics | 実装済み |
| scheduler integration | 実装済み |
| append-only operator annotations | 実装済み |
| append-only manual dismissal | 実装済み |
| public/admin HTTP listener separation | 実装済み |
| admin Bearer authentication | 実装済み |
| append-only authentication failure audit | 実装済み |
| permanently rejected lifecycle | 未実装 |
| multi-role authorization | 未実装 |
| retention / compaction / rotation | 未実装 |
| distributed locking | 未実装 |

---

## 2. Quarantine lifecycle

### Persistent states

```text
pending
promoted
dismissed
```

- `pending`: promotion resolutionもdismissal eventもないrecord
- `promoted`: 有効なpromotion resolutionがあるrecord
- `dismissed`: 有効なappend-only dismissal eventがあるrecord

### Transient revalidation decisions

```text
accept
still-deferred
rejected
```

`rejected`は恒久状態ではありません。`permanently-rejected` lifecycleは未実装です。

### Operator metadata

```text
operator annotation
```

annotationはlifecycle stateではなく、promotion eligibilityを変更しません。

---

## 3. HTTP公開境界

### Public listener

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

公開listenerで利用できるroute：

```text
GET  /v1/ready
GET  /v1/capabilities
POST /v1/objects
GET  /v1/objects/<canonical-id>
```

次の管理routeは公開listenerで`404`を返します。

```text
GET  /metrics
GET  /v1/quarantine-status
GET  /v1/quarantine
GET  /v1/quarantine-resolutions
GET/POST /v1/quarantine/*
```

### Admin listener

```bash
export LINGONBERRY_ADMIN_TOKEN='<long-random-secret>'
cargo run -p lingonberry-relay -- serve-admin-http 127.0.0.1:8788
```

固定条件：

- 既定bind addressは`127.0.0.1:8788`
- `LINGONBERRY_ADMIN_TOKEN`が未設定または空なら起動失敗
- `Authorization: Bearer <token>`を要求
- missing tokenとinvalid tokenは同じ`401 Unauthorized`
- 初期版は単一admin role
- admin listenerで非管理routeは`404`

関連文書：`docs/operations/QUARANTINE_ADMIN_HTTP.md`

関連Issue：#24

---

## 4. Runtime state

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
admin-auth-audit.jsonl
```

### `quarantine.jsonl`

- append-only
- deferされた元publish request
- canonical catalogとは分離
- 監査用原本

### `quarantine-resolutions.jsonl`

- append-only
- promotion成功を永続化
- promotion idempotencyの基礎

### `quarantine-annotations.jsonl`

- append-only operator metadata
- lifecycle stateではない

### `quarantine-dismissals.jsonl`

- append-only lifecycle event
- pending recordを通常のpromotion対象から外す
- 1 record 1 active dismissal

### `admin-auth-audit.jsonl`

- append-only authentication failure audit
- timestamp、remote address、method、path、bounded outcome codeのみ保存
- bearer token、request body、annotation note、quarantine payloadは保存しない

corruptionとI/O errorを黙って無視しません。

---

## 5. CLI

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
cargo run -p lingonberry-relay -- serve-http [addr]
cargo run -p lingonberry-relay -- serve-admin-http [addr]
```

CLIはローカル管理運用の正本です。

---

## 6. Admin HTTP surface

Bearer認証後に利用可能：

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

manual dismissalのHTTP mutation endpointは追加していません。

---

## 7. Statusとmetrics

Status fields：

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

高カーディナリティなID、operator、自由文noteはlabelに使用しません。

---

## 8. 主要ファイル

```text
docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md
docs/roadmap/QUARANTINE_LIFECYCLE_BACKLOG.md
docs/operations/QUARANTINE_ADMIN_HTTP.md
docs/operations/QUARANTINE_DISMISSALS.md
packages/core/src/quarantine.rs
packages/core/src/quarantine_status.rs
packages/core/src/quarantine_annotations.rs
packages/core/src/quarantine_dismissals.rs
packages/relay/src/admin_auth.rs
packages/relay/src/main.rs
packages/relay/src/main_entry.rs
deploy/systemd/lingonberry-admin-http.service
```

---

## 9. 再開時の確認

```bash
git switch main
git pull --ff-only
git status
cargo test --workspace
```

Public boundary確認：

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
curl -i http://127.0.0.1:8787/v1/ready
curl -i http://127.0.0.1:8787/v1/quarantine-status
```

2つ目は`404`であること。

Admin boundary確認：

```bash
export LINGONBERRY_ADMIN_TOKEN='<test-token>'
cargo run -p lingonberry-relay -- serve-admin-http 127.0.0.1:8788
curl -i http://127.0.0.1:8788/v1/quarantine-status
curl -i -H "Authorization: Bearer $LINGONBERRY_ADMIN_TOKEN" \
  http://127.0.0.1:8788/v1/quarantine-status
```

1つ目は`401`、2つ目は`200`であること。

systemd template確認：

```bash
systemd-analyze verify deploy/systemd/lingonberry-admin-http.service
```

---

## 10. 絶対に崩さない安全性ルール

1. validationを通過していないobjectをcanonical storageに保存しない
2. unsupported identity ruleを通常のmismatchと混同しない
3. `defer`されたobjectはquarantineにのみ保存する
4. promotion前に現在のvalidatorとacceptance policyで再評価する
5. rejected / still-deferred recordをcanonical storageへ移さない
6. promotionの再実行はidempotentに扱う
7. 元quarantine recordを監査証跡として保持する
8. dry-runでは永続データを変更しない
9. corruptionとI/O errorを黙って無視しない
10. annotationをlifecycle stateとして解釈しない
11. dismissalは専用append-only eventで表現する
12. dismissed recordを通常のbatch promotion対象へ戻さない
13. status / metricsはledgerを変更しない
14. quarantine管理routeを公開listenerへ戻さない
15. admin listenerをtoken未設定で起動しない
16. bearer tokenや自由文noteをauth auditへ記録しない
17. remote-by-defaultのadmin bindを採用しない

---

## 11. 次の推奨作業

### 第一候補

```text
QL-3: Persistent Rejection Decisions
```

transientな`rejected`と恒久的なlifecycle stateを分離します。

### 第二候補

```text
QL-4: Backup / Export / Restore
```

全append-only ledgerの一貫したsnapshot境界を定義します。

### 第三候補

```text
QL-6: Concurrent Ledger Operations
```

promotion、dismissal、annotation、audit appendの競合条件を固定します。

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
→ #23 manual dismissal
→ #24 admin HTTP isolation
```
