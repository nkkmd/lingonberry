# ロードマップ

**Status: v0.5.0 release candidate merged** | **Latest published release: v0.4.0** | **Last updated: 2026-07-19**

このディレクトリには、実装・運用準備・releaseのroadmap、checklist、release note、および作業再開用の現在地文書を置きます。

## 再開時に最初に読む文書

1. [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)
2. [v1.0までのロードマップ](./ROADMAP_TO_V1_0.md)
3. [v0.5.0 Release Roadmap](./RELEASE_0_5_0_ROADMAP.md)
4. [v0.5.0 Release Checklist](./RELEASE_0_5_0_CHECKLIST.md)
5. [v0.5.0 Release Notes](./RELEASE_0_5_0_RELEASE_NOTE.md)
6. [v0.4.0 Release Checklist](./RELEASE_0_4_0_CHECKLIST.md)
7. [v0.4.0 Release Notes](./RELEASE_0_4_0_RELEASE_NOTE.md)
8. [Quarantine Replacement Retention Policy](../operations/QUARANTINE_REPLACEMENT_RETENTION_POLICY.md)
9. [Cleanup Operations Runbook](../operations/QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)
10. [運用文書索引](../operations/README.md)
11. [実装バックログ](./IMPLEMENTATION_BACKLOG.md)

`CURRENT_IMPLEMENTATION_STATUS.md`は中断後に作業を再開するための引き継ぎ用正本です。`ROADMAP_TO_V1_0.md`はrelease-level roadmap、`RELEASE_0_5_0_ROADMAP.md`はv0.5.0の実装・検証記録です。

## v0.5.0の到達点

通常のknowledge object lifecycleを単一ノードでend-to-endに成立させました。

```text
publish
→ validate
→ store
→ retrieve
→ query
→ restart
→ retrieve／query
→ rebuild／consistency verification
→ checkpoint catch-up
```

主な到達点：

- publish／retrieval／basic queryのversioned contract
- CLI／HTTP共通ingestion orchestration
- duplicate／conflictのdeterministic分類
- canonical storageを正本としたindex generation／verification／rebuild
- atomic checkpointとcheckpoint-driven catch-up
- corrupt／unsupported／partial／ambiguous stateのfail-closed処理
- restart／recovery／ambiguityを含む実binary smoke coverage

## v0.4.0から維持する基盤

- deterministic retention evaluationとretained generation floor
- canonical cleanup plan／proofとdigest sidecar
- stale state、path traversal、symlink、unexpected entryのfail-closed rejection
- same-filesystem tomb preparationとsealed inventory
- durable path-level progress、idempotent resume、pre-processing rollback
- irreversible boundary後の`recovery-required`／`partially-deleted`分類
- explicit double opt-in authorization

v0.4.0 cleanupはexact-subject、proof-bound、operator-triggeredです。scheduled／unattended cleanupは導入していません。

## 文書の役割

- [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md): 現在のrelease state、実装済み機能、安全境界、次の作業
- [v1.0までのロードマップ](./ROADMAP_TO_V1_0.md): v0.5.0以降のrelease sequenceとv1.0の境界
- [v0.5.0 Release Roadmap](./RELEASE_0_5_0_ROADMAP.md): 通常object lifecycleの設計・実装・検証記録
- [v0.5.0 Release Checklist](./RELEASE_0_5_0_CHECKLIST.md): release gateとpublication記録
- [v0.5.0 Release Notes](./RELEASE_0_5_0_RELEASE_NOTE.md): 公開範囲、互換性、安全性、既知制約
- [Retention Policy](../operations/QUARANTINE_REPLACEMENT_RETENTION_POLICY.md): cleanup適格性、proof、revalidation、不可逆境界の正本
- [Cleanup Runbook](../operations/QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md): operatorの停止条件、回復分類、証拠保全
- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md): core実装の中長期計画
- [実装バックログ](./IMPLEMENTATION_BACKLOG.md): 全体roadmapのissue分解
- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md): 実運用に向けた中長期計画

## Release history

- v0.5.0: normal object lifecycle、versioned public contracts、index verification／checkpoint／catch-up、restart recovery。
- v0.4.0: deterministic retention cleanup、proof-bound authorization、verified cleanup transaction、path-level recovery。
- v0.3.0: verified replacement transaction、generation publication、recovery、operations hardening。
- v0.2.0: persistent quarantine lifecycle、backup／restore、RBAC。
- v0.1.0: initial protocol／schema／fixtures／carrier contracts。

## 絶対に崩さない境界

- validation未通過objectをcanonical storageへ保存しない
- canonical storageよりindexをsemantic sourceとして優先しない
- duplicateとconflictを同一分類にしない
- conflict時に既存objectを上書きしない
- partial index updateを完全成功として扱わない
- inconsistent verification resultからcheckpointを更新しない
- corruptionとI/O errorを黙って無視しない
- active generationをcleanup対象にしない
- filesystem timestampだけでcleanup eligibilityを判断しない
- wildcard、implicit-all、partial selectionをcleanup applyに使わない
- symbolic linkやunsupported entry typeをfollow／acceptしない
- stale proofやcontradictory stateを自動修復しない
- archive segmentやimmutable evidence ledgerをrewrite／deleteしない
- irreversible processing開始後にrollbackを案内しない
- same-host lockをdistributed lockとして扱わない
