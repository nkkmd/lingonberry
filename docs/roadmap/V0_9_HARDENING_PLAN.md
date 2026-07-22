# v0.9.0 Release Candidate Hardening Plan

**Status: active** | **Parent issue: #107** | **Branch: `release/v0.9.0-release-candidate-hardening`** | **Last updated: 2026-07-22**

## 1. Goal

v0.9.0はv1.0直前のhardening releaseである。新しいproduct capabilityを広げず、既存のsingle-node implementationについて公開契約、安全境界、障害時挙動、長時間運用の安定性を検証し、freeze candidateとして確定する。

## 2. Workstreams

### W1: Public surface and contract freeze

Deliverables:

- Rust public API inventory
- protocol／API／storage freeze candidate
- compatibility matrix review
- version-axis review
- semver and breaking-change policy

Exit criteria:

- accidental public APIが特定され、縮小または明示的に承認されている
- fixture、specification、implementationの差異がない
- unknown newer contractはfail closedである

### W2: Security review and regression

Deliverables:

- security review matrix
- threat-boundary inventory
- finding ledger
- targeted regression tests
- information-leakage and authorization-ordering review

Exit criteria:

- Critical／High findingがゼロ
- fixed findingにregression testがある
- Medium／Low findingのdispositionが記録されている

### W3: Fuzzing and property testing

Initial vertical slice:

1. protocol parser
2. semantic validator
3. identifier／digest verifier

Expansion:

4. journal parser
5. recovery classifier
6. index／segment reader

Operating model:

- pull request CIではdeterministic bounded regression corpusを実行する
- 長時間fuzzはmanualまたはscheduled workflowで実行する
- crash corpusはminimizeし、永続regression fixtureへ昇格する
- timeout、memory、artifact retentionを明示する

Exit criteria:

- targetごとにpanic、crash、unbounded allocation、unbounded recursionを検出できる
-既知corpusがCIで再現されない

### W4: Production-like acceptance and soak

Reference topology:

- Ubuntu Server 24.04 LTS
- x86_64
- systemd
- non-root service user
- release-built binaries
- local single-node storage／index／relay

Scenario:

```text
install
→ configure
→ doctor / ready
→ sustained publish / read / query
→ periodic restart
→ backup create / verify
→ isolated restore
→ index verify / rebuild
→ representative maintenance operations
→ failure injection
→ recovery verification
```

Observations:

- process memory and growth trend
- file descriptor count
- data／index／backup／workspace disk growth
- request latency and error rate
- journal／pointer／manifest consistency
- recovery time and operator-visible diagnostics

Exit criteria:

- defined durationを完走する
- invariant violation、silent corruption、unbounded growthがない
- restart／failure injection後にdeterministic recoveryが成立する

### W5: Packaging and documentation freeze

Deliverables:

- release candidate artifacts and checksums
- clean-host installation acceptance
- v0.8.0 upgrade／compatible rollback acceptance
- supported／best-effort platform statement
- installation、configuration、protocol、API、security、upgrade、backup／restore、operations、troubleshooting文書
- release checklist、release notes、known issues

Exit criteria:

- 文書だけでreference-platform acceptanceを再現できる
- release artifactと文書が同一commitを参照する

## 3. Implementation order

1. release checklist、hardening plan、security reviewを固定する
2. source／test／workflow inventoryを作る
3. public API inventoryとfreeze-candidate差分を作る
4. parser／validator／identifier／digest fuzz vertical sliceを実装する
5. targeted security regressionを追加する
6. journal／recovery／indexへcoverageを拡張する
7. soak harnessとartifact schemaを実装する
8. reference-platform RC acceptanceを実行する
9. documentation freezeとknown-issue triageを行う
10. release gateを満たしてv0.9.0を公開する

## 4. Change control

v0.9.0中の変更は次のいずれかに分類する。

- `hardening`: safety、validation、durability、boundednessの改善
- `regression-test`: 既存契約の固定
- `contract-freeze`: specification／fixture／compatibility policyの明確化
- `tooling`: fuzz、soak、acceptance、artifact collection
- `documentation`: operator／developer contractの凍結

新しいuser-facing capabilityは原則としてv1.1以降へ送る。例外は、v1.0の既存release gateを成立させるため不可欠であり、compatibility impactが明示された場合に限る。

## 5. Evidence model

各workstreamは次をrelease evidenceとして残す。

-対象commit SHA
- command／workflow name
- run IDまたは実行日時
- environment／platform
- pass criteria
- result
- retained artifact location
- unresolved risk

口頭確認や再現不能なmanual resultだけではrelease gateを満たさない。

## 6. Immediate next actions

- source treeとpublic item inventoryを作成する
- protocol、validation、identity crateのparser／validator surfaceを特定する
-既存CI workflowとtest commandをinventory化する
- fuzz targetを追加可能な最小workspace構成を決める
- path／symlink／input bound／authorization orderingの既存regression coverageを照合する
