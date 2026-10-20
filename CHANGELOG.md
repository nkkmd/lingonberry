# Changelog

All notable changes to Lingonberry are documented in this file. Detailed operational contracts and release notes are retained under `docs/`.

English is normative for the unreleased v1.0.0 summary. The Japanese section is an operational translation and does not define an independent contract.

## [Unreleased]

### v1.0.0 preparation summary

Lingonberry v1.0.0 is not released. The fixed pre-version qualification candidate is `8c6b48082205a3af555130eec1f3e7d2ac8811fe`. Executable evidence bound to the previous candidate `f9543019f2c219aea3b085ff90f2da201b268a48` is historical. Documentation, evidence, and tooling changes made after that commit do not redefine the candidate or replace candidate-bound executable evidence.

#### Completed preparation

- Candidate qualification passed on Ubuntu 24.04 x86_64 with release-built storage and relay binaries, retained checksummed evidence, and recorded binary digests.
- Security and compatibility review found no open Critical, High, or release-blocking Medium finding.
- The candidate documentation walkthrough passed all recorded procedures.
- Protocol, transition, effective-view, versioning, operator, storage, migration, recovery, and evidence contracts were reviewed and normalized without changing the fixed runtime candidate.
- Documentation inventory and freeze checks track remaining release-blocking documentation work explicitly.

#### Compatibility boundary

- The production protocol and Knowledge Object schema remain `0.1.0` for the fixed candidate.
- No post-candidate documentation or tooling commit changes the candidate's protocol behavior, durable storage format, public CLI/HTTP behavior, migration semantics, or release-built binaries.
- Canonical storage remains authoritative; indexes and effective views remain derived and verifiable.
- Unknown, unsupported, contradictory, corrupt, stale, or incomplete state continues to fail closed according to the applicable contract.

#### Not yet completed

- The formal continuous 72-hour qualification soak has not been performed.
- Privileged reference-host qualification remains incomplete.
- At the maintainer’s request, real-host execution has been deferred since 2026-10-20. Resume only after an explicit request and rechecking the candidate, binaries, retained artifacts, and host prerequisites.
- Workspace/package version preparation for `1.0.0` has not been performed.
- The reviewed release PR, merged-release-commit validation, annotated tag, GitHub Release, and final publication evidence do not yet exist.

The eventual `1.0.0` release entry must be created only after all mandatory gates pass. It must identify the merged release commit and publication date rather than treating this unreleased summary as release evidence.

### v1.0.0 準備状況（日本語）

Lingonberry v1.0.0は、まだ公開されていません。バージョン更新前の公開候補は`8c6b48082205a3af555130eec1f3e7d2ac8811fe`に固定しています。このコミット以後に文書、検証記録、ツールを変更しても、公開候補や、それに対応する実行結果は置き換わりません。旧候補`f9543019f2c219aea3b085ff90f2da201b268a48`に対応する実行結果は過去の記録です。

#### 完了済みの準備

- Ubuntu 24.04 x86_64上で、リリース用にビルドしたストレージとリレーのバイナリを使い、公開候補の検証に合格しました。検証記録はチェックサム付きで保存し、バイナリのハッシュ値も記録しています。
- セキュリティと互換性の確認では、未解決のCritical、High、リリースを妨げるMediumの指摘はありません。
- 公開候補を使い、文書に記載されたすべての対象手順の動作確認に合格しました。
- プロトコル、遷移、有効ビュー、バージョン管理、運用、ストレージ、移行、復旧、検証記録の各仕様を、固定した公開候補の実行時の動作を変えずに確認・整理しました。
- 文書一覧と文書の確定範囲の検査によって、公開前に解消すべき文書作業を明示的に管理しています。

#### 互換性の範囲

- 固定した公開候補のプロトコルとKnowledge Objectのスキーマは、`0.1.0`のままです。
- 公開候補の決定後に文書やツールを変更しても、候補のプロトコルの動作、永続保存形式、公開CLI・HTTPの動作、移行の意味、リリース用バイナリは変更しません。
- 正規化済みデータを保存するストレージを引き続き正本とします。インデックスと有効ビューは、検証・再構築できる派生状態です。
- 未知、未対応、矛盾、破損、古い状態、不完全な状態は、該当する仕様に従って安全側に処理を拒否します。

#### 未完了の工程

- 正式72時間連続稼働試験は未実施です。
- 管理者権限による基準ホストでの検証は未完了です。
- 上記の実機試験は、管理者の指示により2026-10-20から保留中です。明示的な再開指示を受けた後に、公開候補、バイナリ、保存済みの検証記録、ホストの前提条件を再確認します。
- ワークスペースと各パッケージのバージョンを`1.0.0`に更新する準備は未実施です。
- レビュー済みのリリースPR、統合後のリリースコミットの検証、注釈付きタグ、GitHub Release、最終的な公開結果の記録は未作成です。

正式な`1.0.0`の変更履歴は、すべての必須検証に合格した後に作成します。その際は、統合後のリリースコミットと公開日を明記します。この未公開版の要約だけを、リリース完了の根拠にはできません。

## [0.9.0] - 2026-07-22

### Added

- Public Rust API inventory and v1.0 freeze-candidate classification across all workspace crates.
- Protocol JSON parser limits: 1 MiB maximum input and maximum array/object nesting depth 128.
- Parser boundary regression tests for exact-size, oversized, maximum-depth, excessive-depth, and mixed-nesting inputs.
- Signature-verification workspace security tests covering cleanup, Unix owner-only permissions, create-new collision rejection, and concurrent isolation.
- v0.9.0 security review, finding ledger, remediation contract, release checklist, and release-evidence ledger.
- Five-iteration bounded hardening soak for parser, signature workspace, and replacement crash-matrix regressions.

### Changed

- All Rust workspace packages and `Cargo.lock` are versioned as `0.9.0`.
- Signature verification uses exclusive temporary-directory creation with PID, timestamp, and an atomic counter.
- Signature verification artifacts use create-new semantics and ordinary success/error paths are cleaned by an RAII guard.
- Signature workspace and artifact failures return generic non-sensitive errors rather than host-path or verification-material details.
- Protocol and public API surfaces are treated as v1.0 freeze candidates; breaking changes require explicit compatibility review.

### Compatibility and safety

- Protocol and schema versions remain `0.1.0`; no wire-format breaking change is introduced.
- Storage format, migration journals, backup archives, replacement proofs, cleanup proofs, and authorization ordering are unchanged.
- Oversized or excessively nested JSON input now fails closed at the protocol-library boundary.
- Unknown-newer, corrupt, contradictory, or unsupported durable state continues to fail closed.
- Open Critical, High, and release-blocking Medium security findings are zero.

### Known limitations

- Process crash, `SIGKILL`, kernel termination, or host power loss can prevent RAII cleanup of a signature verification workspace.
- The bounded CI soak does not replace long-running production telemetry, disk-pressure injection, or power-loss testing required before v1.0 stable.
- Multi-node coordination, distributed locking, and replication are not included.

## [0.8.0] - 2026-07-22

### Added

- Formal Linux reference platform: Ubuntu Server 24.04 LTS, x86_64, and systemd.
- Hardened systemd units, environment-file examples, non-root ownership, and filesystem layout guidance.
- Integrated storage operator commands for configuration, health, readiness, status, read-only diagnosis, strict verification, metrics, backup, restore, index lifecycle, and disaster-recovery drills.
- Stable machine-readable diagnostic codes, canonical JSON output, documented exit codes, and explicit configuration precedence.
- Verified backup creation with isolated archive import and index verification.
- Non-mutating restore planning and isolated restore application with restored-record read-back verification.
- Isolated DR drill with read verification, duplicate-safe write verification, index verification, and mandatory cleanup.
- Failure-injection coverage proving interrupted isolated restore does not leave partial state.
- v0.7.0 to v0.8.0 systemd upgrade and compatible rollback procedures.
- Ubuntu fresh-runner acceptance using release-built binaries installed into `/usr/local/bin`.

### Changed

- All Rust workspace packages and `Cargo.lock` are versioned as `0.8.0`.
- Operator acceptance validates release-built installed binaries instead of relying on `cargo run`.
- Storage diagnosis inspects the generation pointer, derived index consistency, backup inventory structure, maintenance workspaces, and Linux disk capacity without mutating storage.
- Root documentation points to the v0.8.0 release checklist, release notes, operator runbook, CLI contract, supported-platform contract, and upgrade/rollback guide.

### Compatibility and safety

- No new storage format, public object model, service boundary, or implicit migration is introduced.
- Restore refuses symbolic links, active state/data directories, non-empty targets, and structurally incomplete archives.
- Unknown-newer, corrupt, contradictory, or unsupported durable state continues to fail closed.
- Canonical storage remains authoritative; indexes remain derived, verifiable, and rebuildable.
- Quarantine inspection remains on the existing admin HTTP/RBAC surface.
- Replacement and cleanup remain explicit proof-bound operations governed by existing verifiers and runbooks.

### Known limitations

- Cross-service trace correlation is not introduced in v0.8.0.
- Automatic discovery and semantic verification of every historical replacement/cleanup transaction workspace is deferred until a formal workspace-root discovery contract exists.
- Multi-node operational coordination and distributed locking are not included.

## [0.7.0] - 2026-07-21

### Added

- Versioned storage-format manifest and deterministic storage inspection.
- Source-bound migration plans, durable journals, and fail-closed state classification.
- Verified migration snapshots bound to the plan ID and source inventory digest.
- Explicit migration apply, verify, commit, resume, and rollback execution.
- Dedicated `lingonberry-storage-migrate` operator CLI.
- v0.4.0-equivalent persistent fixture and migration integration coverage.

### Changed

- All Rust workspace packages and `Cargo.lock` are versioned as `0.7.0`.
- Existing unversioned single-node durable state is treated as legacy storage requiring verified backup before migration.

### Compatibility and safety

- Ordinary startup does not perform implicit migration.
- Unknown newer formats, malformed manifests, unsupported layouts, symlinks, special files, missing backup evidence, and changed-after-plan state fail closed.
- Storage format v1 migration does not rewrite canonical durable files; it introduces a verified format manifest.
- Protocol and public object-lifecycle contracts remain unchanged.

### Known limitations

- Automatic downgrade is not supported; downgrade requires restoration of a compatible verified backup.
- Multi-node migration coordination and distributed locking are not included.

## [0.6.0] - 2026-07-20

### Added

- Signed append-only Transition Objects through dedicated `POST /v1/transitions` handling.
- Replace and withdraw transitions with duplicate, immutable-conflict, orphan-retention, authority, supersession, and ambiguous-head contracts.
- Durable target-scoped reevaluation, generation coalescing, restart reconciliation, and a dedicated reevaluation CLI.
- Deterministic target evidence generations, including classified unsupported, corrupt, and unreadable markers.
- Last-known-good effective views with separate semantic and observation checkpoints.
- Stable bounded diagnostics and generation-fixed pagination, retention, cursor-lease, read-guard, and heartbeat contracts.

### Changed

- All Rust workspace packages and `Cargo.lock` are versioned as `0.6.0`.
- Original Knowledge Objects remain immutable; replacement and withdrawal are represented as derived effective views.

### Compatibility and safety

- v0.5.0 publish, retrieval, query, indexing, quarantine, backup, cleanup, archive, and immutable-evidence paths remain available.
- Unauthorized, unknown, incomplete, stale, corrupt, or ambiguous transition state fails closed.
- Missing-target transitions remain evidence but do not affect an effective view until reevaluated.
- Incomplete observations cannot overwrite the last-known-good semantic checkpoint.

### Known limitations

- Complete external delegation and revocation evaluation is not included.
- Multi-node queue coordination and distributed snapshot locking are not included.
- Durable cursor lease and read-guard storage remains deployment-specific.

## [0.5.0] - 2026-07-19

- Added versioned publish, retrieval, and basic-query contracts with stable machine codes.
- Added deterministic duplicate/conflict classification and shared CLI／HTTP ingestion orchestration.
- Added deterministic index generations, verification, atomic checkpoints, catch-up, and restart/recovery smoke coverage.
- Canonical storage remains authoritative over derived index state.

## [0.4.0] - 2026-07-17

- Added deterministic quarantine replacement retention evaluation and a retained-generation floor.
- Added durable terminal completion evidence, proof-bound cleanup authorization, and resumable cleanup journals.
- Added path-level recovery, operator runbooks, crash matrices, and release smoke coverage.

## [0.3.0] - 2026-07-15

- Added verified backup-bound replacement transactions and generation-directory publication.
- Added deterministic transaction journals, resume, rollback, structured status, metrics, and failure injection.
- Preserved immutable evidence ledgers and archive segments under fail-closed recovery semantics.

## [0.2.0] - 2026-07-12

- Added persistent quarantine lifecycle, revalidation, promotion, dismissal, and permanent rejection.
