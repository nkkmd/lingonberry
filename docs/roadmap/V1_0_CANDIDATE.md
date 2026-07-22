# Lingonberry v1.0.0 Pre-Version Candidate

**Status: candidate designation pending merge** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issue: #126** | **Designation date: 2026-07-23**

## 1. Designation rule

The merge commit that introduces this document to `main` is designated as the Lingonberry v1.0.0 pre-version qualification candidate.

The exact candidate SHA is not the pull-request head SHA. It is the resulting `main` merge commit, which must be qualified by the `v1 candidate qualification` workflow through its `push` trigger.

After merge, the exact SHA, workflow run, artifact ID, artifact digest, and binary SHA-256 values must be recorded in `V1_0_RELEASE_EVIDENCE.md` and the tracking issue.

## 2. Candidate scope

The candidate includes:

- the v0.9.0 production implementation and single-node operator baseline;
- the approved normative v1 compatibility policy;
- the completed Rust public API audit;
- qualification, security-diff, documentation-freeze, and soak contracts;
- candidate-bound qualification and documentation-integrity workflows;
- active v1 indexes and the pre-candidate documentation walkthrough record.

The reviewed interval from `v0.9.0` through the pre-candidate head contains no production runtime implementation change under `packages/**`. The v1.0 work in that interval consists of contract finalization, review records, and qualification infrastructure.

## 3. Candidate qualification requirements

The designated merge commit must produce a checksummed qualification artifact containing:

- exact candidate commit;
- repository and workflow provenance;
- Ubuntu 24.04 x86_64 platform record;
- Rust, Cargo, and Node versions;
- release-built `lingonberry-storage` and `lingonberry-relay` binaries;
- binary SHA-256 values;
- per-gate JSON results and complete logs;
- aggregate summary;
- bundle `SHA256SUMS`.

All recorded gates must pass. The artifact candidate commit must equal the pushed `main` merge commit.

## 4. Evidence boundary

Passing candidate qualification does not by itself authorize version preparation or publication.

The following remain mandatory after candidate qualification:

1. reference-platform documentation walkthrough using candidate-built binaries;
2. final candidate security disposition;
3. 72-hour qualification soak and workload floors;
4. residual-risk and deviation review;
5. version `1.0.0` preparation and release-document freeze;
6. reviewed release PR, merged-commit validation, tag, GitHub Release, and final evidence.

## 5. Change control

After designation:

- a runtime-affecting, protocol, durable-format, CLI/HTTP contract, default, migration, or recovery behavior change invalidates candidate-bound executable evidence and requires a new candidate;
- an evidence-only correction must be recorded and reviewed for whether it affects operator acceptance or documentation freeze;
- a command, path, required setting, diagnostic code, or recovery-instruction change invalidates the affected walkthrough evidence;
- no change may silently move the qualified SHA while retaining old binary or soak evidence.

## 6. Post-merge record

To be completed from the `main` push evidence:

```text
candidate merge SHA: Pending
qualification workflow run: Pending
qualification artifact ID: Pending
qualification artifact digest: Pending
lingonberry-storage SHA-256: Pending
lingonberry-relay SHA-256: Pending
standard CI result: Pending
qualification disposition: Pending
```
