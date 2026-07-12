# Quarantine Operator Annotations

**Status: implemented** | **Last updated: 2026-07-12**

## 1. 目的

quarantine record に対する運用上の確認事項や判断根拠を、元recordを変更せずappend-onlyの監査イベントとして記録します。

annotationはlifecycle stateではありません。recordをpromoted、dismissed、rejectedなどへ遷移させず、promotion判定にも影響しません。

## 2. 永続ファイル

```text
<state-dir>/quarantine-annotations.jsonl
```

各行は独立したannotation eventです。

```json
{
  "id": "lb:qa:...",
  "quarantineId": "lb:q:...",
  "annotatedAt": "...Z",
  "operator": "operator-name",
  "note": "reviewed source material"
}
```

## 3. CLI

annotation追加：

```bash
lingonberry-relay quarantine-annotate \
  <quarantine-id> \
  <operator> \
  <note>
```

noteに空白を含む場合はshellで引用します。

```bash
lingonberry-relay quarantine-annotate \
  lb:q:123 \
  akihiro \
  "source identity requires follow-up"
```

全annotation一覧：

```bash
lingonberry-relay quarantine-annotations
```

特定recordのannotation一覧：

```bash
lingonberry-relay quarantine-annotations lb:q:123
```

## 4. HTTP

追加：

```text
POST /v1/quarantine/<quarantine-id>/annotations
Content-Type: application/json
```

```json
{
  "operator": "akihiro",
  "note": "source identity requires follow-up"
}
```

一覧：

```text
GET /v1/quarantine/<quarantine-id>/annotations
```

## 5. Validation

次を拒否します。

- 存在しないquarantine ID
- 空のoperator
- 空のnote
- objectではないHTTP request body
- 必須fieldがないrequest
- corrupt annotation JSONL

operatorとnoteは前後の空白を除去して保存します。

## 6. Append-only原則

- 元の`quarantine.jsonl`を書き換えない
- resolution ledgerを書き換えない
- annotationの更新・削除APIを設けない
- 同一recordへ複数annotationを追加できる
-過去のannotationを上書きせず、新しいeventを追記する

誤ったannotationを訂正する場合も、訂正内容を新しいannotationとして追記します。

## 7. Securityと個人情報

annotationは運用管理情報です。

- HTTP管理endpointを一般公開しない
- operatorには安定した運用識別子を使う
- secret、credential、個人情報をnoteへ記録しない
- 自由文noteをmetric labelへ使用しない
- access control実装前は信頼された管理環境だけで使用する

## 8. Lifecycleとの関係

annotationの有無にかかわらず、pending recordはschedulerや手動promotionの対象です。

manual dismissalを将来追加する場合は、annotationとは別のappend-only lifecycle eventとして設計します。annotationの特定文言を機械的なdismissal状態として解釈してはいけません。

## 9. 非スコープ

- manual dismissal
- permanently rejected state
- annotationの更新・削除
- authentication / authorization
- retention / compaction
- distributed locking

## 10. 関連資料

- [Quarantine Status API](../roadmap/QUARANTINE_STATUS_API.md)
- [Quarantine Scheduler](./QUARANTINE_SCHEDULER.md)
- [Quarantine Observability Metrics](./QUARANTINE_OBSERVABILITY_METRICS.md)
