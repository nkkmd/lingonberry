# Quarantine Scheduler

**Status: implemented** | **Last updated: 2026-07-12**

## 1. 目的

未解決の quarantine record を定期的に再評価し、現在の validator と acceptance policy を通過したものだけを canonical storage へ昇格させます。

定期実行の正本は、既存 CLI の次のコマンドです。

```bash
lingonberry-relay quarantine-promote-batch 100
```

HTTP 管理 endpoint を外部 scheduler から呼び出す構成は、authentication、authorization、network exposure の設計が完了するまで推奨しません。

## 2. 実行モデル

### 2.1 推奨構成

```text
systemd timer
    ↓
oneshot service
    ↓
quarantine-promote-batch
    ↓
current validation + acceptance policy
    ├─ promoted
    ├─ deferred
    └─ rejected
```

### 2.2 既定値

```text
interval: 15 minutes
batch limit: 100
randomized delay: up to 60 seconds
persistent timer: enabled
```

`Persistent=true` により、停止中に実行時刻を過ぎた場合は、次回起動後に一度実行されます。

## 3. systemd

テンプレート：

```text
deploy/systemd/lingonberry-quarantine-promote.service
deploy/systemd/lingonberry-quarantine-promote.timer
```

配置：

```bash
sudo install -m 0644 deploy/systemd/lingonberry-quarantine-promote.service /etc/systemd/system/
sudo install -m 0644 deploy/systemd/lingonberry-quarantine-promote.timer /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now lingonberry-quarantine-promote.timer
```

状態確認：

```bash
systemctl status lingonberry-quarantine-promote.timer
systemctl list-timers lingonberry-quarantine-promote.timer
journalctl -u lingonberry-quarantine-promote.service
```

手動実行：

```bash
sudo systemctl start lingonberry-quarantine-promote.service
```

停止：

```bash
sudo systemctl disable --now lingonberry-quarantine-promote.timer
```

## 4. dry-run

本番で timer を有効化する前に、同じ state directory と policy を使って dry-run を実行します。

```bash
sudo -u lingonberry \
  env LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay \
  /usr/local/bin/lingonberry-relay quarantine-promote-batch 100 --dry-run
```

確認事項：

- canonical storage が変更されない
- `quarantine-resolutions.jsonl` が変更されない
- `deferred` と `rejected` の件数が想定内
- corrupt JSONL や I/O error がない

## 5. cron fallback

systemd timer を利用できない環境では、cron を fallback として使用できます。

```cron
*/15 * * * * LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay /usr/local/bin/lingonberry-relay quarantine-promote-batch 100 >> /var/log/lingonberry/quarantine-promote.log 2>&1
```

ただし、systemd timer の方が次の点で推奨です。

- service ごとの実行結果を journal で確認できる
- missed run を `Persistent=true` で補える
- user、working directory、環境変数を明示できる
- timeout と resource control を追加しやすい

## 6. 排他制御

現時点では、複数 scheduler や複数 process から同じ batch を並行実行しないことを運用前提とします。

次を避けます。

- systemd timer と cron の同時有効化
- 複数 host から同一 state directory を対象に実行
- timer 実行中の手動 batch 実行

unit には `flock` を使用し、同一 host 上の重複実行を抑止します。ただし、共有 filesystem 上の分散 lock を保証するものではありません。

## 7. Failure handling

service が非ゼロ終了した場合、systemd は unit を failed として記録します。

確認順：

1. `systemctl status lingonberry-quarantine-promote.service`
2. `journalctl -u lingonberry-quarantine-promote.service -n 100`
3. `lingonberry-relay quarantine-status`
4. `lingonberry-relay quarantine-metrics`
5. ledger の権限と空き容量
6. corrupt JSONL の有無

失敗時に ledger error を `0 records` として扱いません。

## 8. Observability

定期実行後は、次を確認します。

```bash
lingonberry-relay quarantine-status
lingonberry-relay quarantine-metrics
```

重点項目：

- pending が継続的に減少または安定している
- oldest pending age が継続増加していない
- 特定 reason code が急増していない
- service の失敗が連続していない

## 9. Security

scheduler は relay の管理操作です。

- 専用の `lingonberry` user で実行する
- state directory への必要最小限の権限だけを付与する
- shell 経由の可変入力を渡さない
- HTTP endpoint を一般公開しない
- environment file に secret を保存する場合は mode `0600` とする

## 10. 非スコープ

- distributed locking
- retry queue
- exponential backoff
- HTTP scheduler authentication
- retention / compaction
- operator annotation
- permanently rejected lifecycle

## 11. 関連資料

- [Quarantine Status API](../roadmap/QUARANTINE_STATUS_API.md)
- [Quarantine Observability Metrics](./QUARANTINE_OBSERVABILITY_METRICS.md)
- [Observability](./OBSERVABILITY.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)
