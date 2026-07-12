# Quarantine Lifecycle Backlog

**Status: active** | **Last updated: 2026-07-12**

この文書は、quarantine運用の次段階を再開しやすいissue単位で整理します。

現在地の正本は [CURRENT_IMPLEMENTATION_STATUS.md](./CURRENT_IMPLEMENTATION_STATUS.md) です。実装前に必ず両方を確認してください。

---

## 完了済み

| 項目 | PR / Issue | 状態 |
|---|---:|---|
| persistent quarantine store | #8 | 完了 |
| revalidation / promotion | #9 | 完了 |
| batch promotion / dry-run | #10 | 完了 |
| status API | #13 | 完了 |
| Prometheus metrics | #15 | 完了 |
| scheduler integration | #17 | 完了 |
| operator annotations | #19 | 完了 |
| append-only manual dismissal lifecycle | #22 / #23 | 完了 |
| admin authentication and network isolation | #24 | 実装・PR化 |

---

## QL-1: Append-only Manual Dismissal Lifecycle

**状態: completed**

- pending recordのみdismiss可能
- 1 record 1 active dismissalとしてidempotent
- undo / reopenは非スコープ
- bounded reasonCode + operator note
- Core + CLI
- HTTP mutation APIは非スコープ

関連文書：`docs/operations/QUARANTINE_DISMISSALS.md`

---

## QL-2: Admin Authentication and Network Isolation

**状態: implemented**

### 固定した判断

```text
public listener: quarantine admin routeを404で遮断
admin listener: 127.0.0.1:8788を既定
認証: LINGONBERRY_ADMIN_TOKEN + Bearer token
authorization: 初期版は単一admin role
失敗監査: admin-auth-audit.jsonlへappend-only
```

### 実装済み完了条件

- `serve-http`からquarantine管理routeを分離
- `serve-admin-http`を追加
- token未設定・空文字でadmin listener起動を拒否
- missing / invalid tokenを同一401 responseとして処理
- 認証失敗をbounded metadataだけでappend-only audit
- bearer token、request body、annotation note、quarantine payloadをauditへ保存しない
- loopback-only systemd templateを追加
- 管理endpointの公開境界を文書化

### 非スコープ

- TLS termination
- multi-role RBAC
- distributed rate limiting
- browser session / CSRF protection
- remote-by-default binding
- dismissal HTTP mutation endpoint

関連文書：`docs/operations/QUARANTINE_ADMIN_HTTP.md`

---

## QL-3: Persistent Rejection Decisions

**優先度: highest**

### 目的

現在のtransientな`rejected`判定と、operatorまたはpolicyによる恒久的なlifecycle stateを分離します。

### 検討項目

- `permanently-rejected`を自動判定できるか
- policy変更後の再評価を許すか
- operator承認を必須にするか
- dismissalとの違い
- status / metrics上の分類
- event取消の表現

### 注意

現在の`rejected`をそのまま永続状態として数えてはいけません。

---

## QL-4: Backup / Export / Restore

**優先度: medium**

### 対象

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
admin-auth-audit.jsonl
将来のrejection ledger
```

### 完了条件

- 一貫したsnapshot境界が定義される
- restore時の重複と順序を扱える
- 原本hashまたはmanifestを検討する
- restore検証手順がある
- 権限と秘密情報の扱いを文書化する

---

## QL-5: JSONL Index / Rotation / Compaction

**優先度: medium-low**

### 検討順

1. read-only index
2. archive export
3. rotation
4. verified compaction
5. retention policy

### 固定条件

- compaction前の原本を検証可能にする
- lifecycle eventの意味を失わない
- unknown / corrupt lineを黙って除外しない
- statusとmetricsの値がcompaction前後で一致する

---

## QL-6: Concurrent Ledger Operations

**優先度: medium**

### 対象

- 同一recordの同時promotion
- resolution ledgerへの同時append
- annotation ledgerへの同時append
- dismissal ledgerへの同時append
- admin auth auditへの同時append
- schedulerと手動操作の競合
- promotionとdismissalの競合

### 完了条件

- 同一hostとmulti-hostを区別する
- atomic appendの前提を文書化する
- duplicate eventの意味を固定する
- concurrency testを追加する
- distributed lockを実装しない場合は明記する

---

## 再開時のIssue作成テンプレート

```markdown
## Goal

## Persistent state changes

## CLI / HTTP changes

## Lifecycle semantics

## Idempotency and concurrency

## Error handling

## Tests

## Documentation updates

## Non-goals
```

各quarantine関連PRでは、次を完了条件に含めます。

```text
CURRENT_IMPLEMENTATION_STATUS.mdを更新する、または更新不要の理由をPR本文へ記載する
```
