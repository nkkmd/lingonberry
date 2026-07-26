# Changelog

All notable changes to Lingonberry are documented in this file. Detailed operational contracts and release notes are retained under `docs/`.

English is normative for the unreleased v1.0.0 summary. The Japanese section is an operational translation and does not define an independent contract.

## [Unreleased]

### v1.0.0 preparation summary

Lingonberry v1.0.0 is not released. The fixed pre-version qualification candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation, evidence, and tooling changes made after that commit do not redefine the candidate or replace candidate-bound executable evidence.

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
- Workspace/package version preparation for `1.0.0` has not been performed.
- The reviewed release PR, merged-release-commit validation, annotated tag, GitHub Release, and final publication evidence do not yet exist.

The eventual `1.0.0` release entry must be created only after all mandatory gates pass. It must identify the merged release commit and publication date rather than treating this unreleased summary as release evidence.

### v1.0.0 準備状況（日本語）

Lingonberry v1.0.0 は、まだリリースされていません。固定されたversion準備前のqualification candidateは `f9543019f2c219aea3b085ff90f2da201b268a48` のままです。このcommit以後の文書・証跡・tooling変更はcandidateを再定義せず、candidateに紐づく実行証跡を置き換えません。

#### 完了済みの準備

- Ubuntu 24.04 x86_64上で、release buildされたstorage／relay binary、checksum付き保存証跡、記録済みbinary digestを用いたcandidate qualificationが成功しています。
- Security／compatibility reviewでは、未解決のCritical、High、release-blocking Medium findingはありません。
- Candidate documentation walkthroughは、記録対象の全procedureに成功しています。
- Protocol、transition、effective view、versioning、operator、storage、migration、recovery、evidenceの各contractを、固定runtime candidateを変更せずに確認・正規化しました。
- Documentation inventoryとfreeze checkにより、残るrelease-blocking文書作業を明示的に追跡しています。

#### 互換性の境界

- 固定candidateのproduction protocolおよびKnowledge Object schemaは`0.1.0`のままです。
- Candidate以後の文書・tooling commitは、candidateのprotocol挙動、durable storage format、公開CLI／HTTP挙動、migration semantics、release build済みbinaryを変更しません。
- Canonical storageが引き続き正本であり、indexとeffective viewは検証・再構築可能な派生状態です。
- Unknown、unsupported、contradictory、corrupt、stale、incompleteな状態は、該当contractに従ってfail closedを維持します。

#### 未完了

- Formal continuous 72-hour qualification soakは未実施です。
- Privileged reference-host qualificationは未完了です。
- Workspace／packageの`1.0.0` version準備は未実施です。
- Reviewed release PR、merged release commitの検証、annotated tag、GitHub Release、最終publication evidenceは未作成です。

最終的な`1.0.0` release entryは、すべての必須gate成功後にのみ作成します。そのentryでは、この未リリース要約をrelease evidenceとして扱わず、merged release commitと公開日を明示しなければなりません。

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
