# Lingonberry v1.0.0 Qualification Status / 検証状況

[English](#english) | [日本語](#japanese)

> English is normative. The Japanese summary translates the current status and next steps; detailed technical evidence remains in English.
> 英語を正本とします。日本語の要約では現在の状況と次の工程を説明し、技術的な検証記録の詳細は英語で記載します。

**Status: repository evidence complete; reference-host execution deferred** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issues: #332, #335, #341**

**Execution hold — 2026-10-20:** At the maintainer’s request, privileged reference-host preflight, disk-pressure rehearsal, and the formal 72-hour soak are deferred. Repository preparation and recorded candidate results remain intact; this hold is neither a test failure nor a passed gate. Resume only after an explicit maintainer request, rechecking the fixed candidate, binary identities, retained artifacts, and host prerequisites. Version 1.0.0, tagging, and publication remain pending.

## English

## 1. Active candidate

The fixed v1.0.0 pre-version candidate is:

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

It was designated by the merge of PR #333 and recorded by PR #334. Documentation-only evidence commits after this SHA do not move the candidate.

The previous candidate `f9543019f2c219aea3b085ff90f2da201b268a48` and all executable evidence bound to it are historical and cannot authorize the active runtime.

## 2. Completed runtime correction

PR #331 completed the release-blocking publisher-authentication correction:

- verifies Ed25519 publisher signatures immediately after JSON parsing;
- verifies before acceptance policy, quarantine, duplicate/conflict classification, raw-request append, and canonical storage;
- rejects malformed encodings with `LB_PUBLISH_SIGNATURE_MALFORMED`;
- rejects cryptographically invalid signatures with `LB_PUBLISH_SIGNATURE_INVALID`;
- fails closed on verifier execution failures with `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`;
- covers the checked-in valid vector and tampered-signature path in Rust ingestion;
- preserves intended validation and conflict coverage with correctly signed fixtures.

## 3. Repository-side qualification gates

| Gate | State | Evidence |
|---|---|---|
| Candidate designation | PASS | PR #333; exact SHA recorded by PR #334 |
| Standard CI | PASS | PR #331, #333, #334, #338, and #340 checks |
| Rust and JavaScript regressions | PASS | Workspace tests, Clippy, conformance, signed vectors |
| Documentation inventory and bilingual checks | PASS | Repository CI |
| Documentation freeze check | PASS | Repository CI and active walkthrough |
| Candidate-delta security code review | PASS | `V1_0_SECURITY_DIFF_REVIEW.md` |
| Exact candidate qualification | PASS | run `30238378797`; artifact `8642393171` |
| Candidate documentation walkthrough | PASS | run `30239602412`; artifact `8642773653` |
| Privileged reference-host preflight | PENDING | Execute frozen host checklist |
| Formal 72-hour soak | PENDING | Start only after reference-host GO |

## 4. Active qualification evidence

| Field | Value |
|---|---|
| Qualification run | `30238378797` |
| Qualification artifact | `8642393171` |
| Artifact name | `v1-qualification-8c6b48082205a3af555130eec1f3e7d2ac8811fe-1` |
| Artifact digest | `sha256:c30a0472f6ea07f3e395c9a27c67d1460b8f35a13a7afd397bd0e5895cb93b3e` |
| Candidate SHA in bundle | `8c6b48082205a3af555130eec1f3e7d2ac8811fe` |
| `lingonberry-storage` SHA-256 | `737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507` |
| `lingonberry-relay` SHA-256 | `23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c` |
| Gate result | 12 of 12 passed |
| Manifest verification | Every `SHA256SUMS` entry verified |
| Independent ZIP digest | Matched GitHub artifact digest |

## 5. Active documentation walkthrough evidence

| Field | Value |
|---|---|
| Walkthrough run | `30239602412` |
| Walkthrough artifact | `8642773653` |
| Artifact name | `v1-documentation-walkthrough-8c6b48082205a3af555130eec1f3e7d2ac8811fe-1` |
| Artifact digest | `sha256:9b954ada86f86e5da4966951039af9dddc2eddb3d49c996d09256e4cad598338` |
| Candidate and binary identities | Matched qualification evidence |
| Procedure result | 16 of 16 passed |
| Manifest verification | All 34 `SHA256SUMS` entries verified |
| Independent ZIP digest | Matched GitHub artifact digest |

The walkthrough covers valid, changed, malformed, and verifier-failure signature behavior and confirms duplicate/conflict processing does not bypass authentication.

## 6. Security delta acceptance

The intentional production delta from the superseded candidate is limited to:

- Ed25519 publisher-signature verification at the beginning of ingestion;
- stable malformed, invalid-signature, and verifier-error result codes;
- fail-closed behavior before every terminal, quarantine, duplicate/conflict, raw append, and storage path;
- valid signed fixture replacement for existing schema/conflict contract tests.

No storage format, migration rule, canonical object format, public Rust API, backup/restore format, or recovery procedure is intentionally changed.

## 7. Reference-host preparation inputs

Freeze these values before provisioning or execution:

- Ubuntu Server 24.04 LTS, x86_64, systemd reference platform;
- exact candidate checkout and the recorded storage and relay hashes;
- service user, directories, ownership, and environment files;
- command map and threshold files under `deploy/soak/`;
- evidence output location with sufficient disk and inode capacity;
- UTC start/end timestamps, operator identity, and deviation log location;
- stop conditions for panic, OOM, corruption, divergence, unsafe recovery, verifier failure, or evidence loss.

## 8. Execution order

1. retain the merged repository evidence update;
2. await an explicit maintainer request and recheck the fixed candidate, binary identities, retained artifacts, and host prerequisites;
3. execute privileged reference-host preflight and disk-pressure rehearsal;
4. freeze host-specific command-map and threshold inputs;
5. start the formal 72-hour soak from the fixed candidate;
6. verify evidence completeness and record residual-risk disposition;
7. prepare and review version `1.0.0`;
8. validate the merged release commit, tag, publish, and record final evidence.

## 9. Go/no-go decision

**Current decision: Repository prerequisites satisfied; reference-host execution deferred by maintainer request; NO-GO for formal soak until that preflight passes.**

The exact candidate qualification artifact and active-candidate documentation walkthrough have been independently verified and recorded. Formal soak authorization remains conditional on successful privileged reference-host preparation.

<a id="japanese"></a>

## 日本語

### 現在の状況

リポジトリ側の準備と公開候補の検証は完了しています。管理者の指示により、基準ホストでの事前検証、ディスク圧迫試験の予行演習、正式72時間連続稼働試験は、2026-10-20から保留中です。保留は試験の失敗でも合格でもありません。v1.0.0へのバージョン更新、タグの作成、公開も未完了です。

### 検証対象と完了済みの確認

バージョン更新前の公開候補は`8c6b48082205a3af555130eec1f3e7d2ac8811fe`に固定しています。旧候補`f9543019f2c219aea3b085ff90f2da201b268a48`に対応する実行結果は過去の記録であり、現在の候補の承認には使用しません。

- 公開者のEd25519署名を、JSONの解析直後、受入判断や保存処理より前に検証するよう修正済みです。形式不正、無効な署名、署名検証基盤の障害を拒否します。
- 公開候補の検証は12項目すべて合格しています。実行IDは`30238378797`、成果物IDは`8642393171`です。
- 文書に沿った動作確認は16手順すべて合格しています。実行IDは`30239602412`、成果物IDは`8642773653`です。
- 成果物のハッシュ値、バイナリの識別情報、`SHA256SUMS`の各項目は、独立した方法で検証済みです。手順の動作確認に対応する`SHA256SUMS`は34項目すべて確認しています。
- セキュリティ差分の確認、文書一覧、英日併記、文書の確定範囲の検査は合格しています。

バイナリのハッシュ値などの詳細は、上記の英語部分と[リリースの検証記録](./V1_0_RELEASE_EVIDENCE.md)を参照してください。

### 再開後の工程

1. 統合済みのリポジトリ側の検証記録を保持する
2. 明示的な再開指示を受け、固定した公開候補、バイナリ、保存済みの検証記録、ホストの前提条件を再確認する
3. 管理者権限で基準ホストの事前検証と、ディスク圧迫試験の予行演習を行う
4. ホスト固有の実行コマンドの対応表と、しきい値を固定する
5. 固定した公開候補で正式72時間連続稼働試験を始める
6. 検証記録が揃っていることを確認し、残るリスクへの判断を記録する
7. バージョン`1.0.0`の準備とレビューを行う
8. 統合後のリリースコミットを検証し、タグを作成して公開し、最終的な検証記録を残す

基準環境はUbuntu Server 24.04 LTS / x86_64 / systemdです。実行前に、公開候補とバイナリのハッシュ値、サービス用ユーザー、ディレクトリと所有権、環境設定ファイル、計測のしきい値、記録先の容量とinode数、UTC時刻、実施者、計画からの逸脱の記録先を確定します。パニック、メモリ不足、破損、状態の不一致、安全でない復旧、署名検証基盤の障害、検証記録の消失が起きた場合の停止条件も確認します。

### 実施可否の判断

リポジトリ側の前提条件は満たしていますが、実機試験は保留中です。管理者権限による基準ホストでの事前検証に合格するまで、正式72時間連続稼働試験は開始できません。
