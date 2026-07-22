# Lingonberry v1.0.0 Qualification Soak Plan

**Status: pre-execution normative plan** | **Target: v1.0.0** | **Tracking issue: #114** | **Parent issue: #109**

## 1. Purpose

This document defines the final qualification soak required before Lingonberry v1.0.0 may be released.

The soak is broader than the bounded v0.9 hardening soak. It must exercise the designated v1.0.0 candidate as a continuously operated single-node service, retain operational telemetry, inject safe and reproducible failures, and prove that durable state remains correct and recoverable.

Historical v0.9 soak evidence is supporting precedent only. The pass/fail decision for v1.0.0 must be based on evidence tied to the designated candidate commit and candidate-built artifacts.

## 2. Scope and non-goals

### 2.1 In scope

- Ubuntu Server 24.04 LTS, x86_64, systemd reference deployment
- candidate-built release binaries
- publish, retrieve, query, restart, verify, rebuild, backup, restore, replacement, and cleanup workflows
- malformed and resource-boundary inputs
- abrupt process termination and controlled disk-pressure scenarios
- durable-state, index, journal, proof, archive, workspace, memory, file-descriptor, and disk telemetry
- deterministic recovery and fail-closed behavior

### 2.2 Non-goals

- multi-node coordination or distributed consistency
- production traffic simulation at internet scale
- destructive host power testing that cannot preserve evidence safely
- unsupported filesystems, architectures, init systems, or operating systems
- performance benchmarking as a release claim

A non-goal must not be presented as a supported v1 capability.

## 3. Qualification subject

Before execution, the soak manifest must record:

- candidate Git commit SHA
- candidate version and release-candidate identifier
- source archive digest
- binary/package digest for every installed artifact
- Rust and JavaScript toolchain versions used to build and test
- build workflow and run identifier
- dependency lockfile digest
- soak-plan revision SHA
- compatibility-policy revision SHA
- start and end timestamps in UTC

No source, dependency lockfile, build setting, runtime binary, protocol version, schema version, or durable format may change during a passing soak.

A change to the qualification subject invalidates the run unless the change is explicitly classified as evidence-only and cannot affect runtime behavior. Runtime-affecting changes require a new soak.

## 4. Reference environment

The required reference environment is:

- Ubuntu Server 24.04 LTS
- x86_64
- systemd-managed service
- local durable storage on a supported Linux filesystem
- network access restricted to the test harness and required package/bootstrap endpoints
- synchronized UTC clock

The environment manifest must record:

- kernel version
- filesystem type and mount options
- CPU model and logical CPU count
- total RAM and swap configuration
- total and initially available disk space
- open-file and process limits
- systemd unit contents and effective overrides
- Lingonberry configuration with secrets redacted
- environment-variable names and effective non-secret values
- log retention and rotation settings

## 5. Service configuration and limits

The soak must use documented operator configuration rather than source-tree assumptions.

The manifest must identify:

- listening address and port
- storage, index, archive, backup, quarantine, evidence, and workspace paths
- parser byte and nesting limits
- request/body and concurrency limits
- authentication mode
- retention and cleanup settings
- systemd restart policy
- service memory and file-descriptor limits
- journal retention limits

Safety-sensitive limits must not be disabled merely to obtain a pass.

## 6. Duration and iteration requirements

### 6.1 Minimum duration

The final soak must run for at least **72 continuous hours** after baseline initialization.

The clock pauses only when the plan itself requires an offline isolated-restore or host-maintenance step. Paused time does not count toward the 72-hour minimum.

### 6.2 Minimum successful workload

The run must complete at least:

- 10,000 accepted publish operations
- 10,000 post-publish retrieval checks
- 5,000 query operations across supported filters and pagination boundaries
- 48 graceful service restarts
- 12 abrupt process terminations
- 12 storage/index consistency verifications
- 4 deterministic index rebuilds
- 6 verified backups
- 3 isolated restore drills
- 6 complete replacement/cleanup crash-matrix cycles
- 1,000 malformed or invalid protocol inputs
- 200 oversized input attempts around the configured byte boundary
- 200 deeply nested input attempts around the nesting boundary
- 2 controlled disk-pressure scenarios

If 72 hours elapse before these counts are reached, the run continues until both the duration and workload minima are satisfied.

### 6.3 Workload distribution

Normal traffic and qualification activities must be distributed across the run. A run does not pass by executing all disruptive checks only at the beginning or end.

## 7. Baseline initialization

Before the timed soak begins:

1. install candidate-built artifacts using published installation procedures
2. record the complete environment and candidate manifest
3. initialize empty supported durable state
4. start the systemd service
5. verify health and readiness
6. publish and retrieve a baseline object corpus
7. record canonical object digests and expected query results
8. run storage and index verification
9. create and verify a baseline backup
10. confirm that signature and temporary workspaces are empty or documented

Any unexplained pre-existing state invalidates the environment.

## 8. Continuous workload

### 8.1 Publish workload

The harness must publish a deterministic mix of:

- distinct valid objects
- exact duplicates
- identity-equivalent duplicates where supported
- deliberate conflicts
- supported object types and protocol versions
- boundary-sized but valid objects

Expected accept, duplicate, conflict, defer, or reject outcomes must be known before submission and checked by machine-readable code.

### 8.2 Retrieve and query workload

Every accepted object must be sampled for retrieval and query visibility. Checks include:

- direct retrieval by identifier
- supported filters
- pagination boundaries
- deterministic ordering where documented
- effective-view behavior
- persistence across restart
- consistency after index rebuild

### 8.3 Restart workload

Graceful restarts must use documented systemd procedures. The harness must verify after each restart:

- service activation
- health and readiness
- durable object availability
- index consistency
- absence of contradictory journal or generation state
- bounded workspace residue

## 9. Backup, verification, and isolated restore cadence

At least every 12 hours:

1. create a backup using the documented command
2. verify the backup and its binding evidence
3. retain the manifest, digest, command output, and exit status

At least three times during the run:

1. select a verified backup
2. restore into an isolated, inactive target
3. reject active, non-empty, unsafe, or symlinked targets in separate negative checks
4. verify restored durable state
5. rebuild and verify the restored index
6. run publish, retrieve, query, restart, and consistency checks against the restored instance

The primary soak instance must not be replaced by the isolated restore target.

## 10. Index verification and rebuild cadence

At least every 6 hours, run index/storage consistency verification.

At least four times:

1. retain a last-known-good index snapshot or evidence record
2. introduce only a documented, safe derived-state invalidation in the test environment
3. prove verification detects the invalid state
4. rebuild from canonical durable storage
5. reverify deterministic observable state
6. confirm canonical durable bytes were not changed by rebuild

An index that silently becomes the semantic source of truth is a release blocker.

## 11. Replacement and cleanup crash matrix

The complete documented replacement and cleanup crash-point matrix must run at least six times during the soak.

Each cycle must cover applicable interruption points around:

- authorization and proof validation
- staging
- durable journal publication
- generation or pointer publication
- archive/evidence publication
- cleanup authorization
- destructive mutation
- final verification

For each injected interruption, the harness must record:

- exact crash point
- operation identifier
- pre-crash durable evidence
- process termination method
- restart/recovery result
- resume or rollback result
- final proof and subject binding
- final object, archive, quarantine, and index state

Every outcome must resolve to a documented complete, resumable, rolled-back, or fail-closed state. Ambiguous success is failure.

## 12. Malformed and boundary-input workload

The soak must repeatedly submit:

- invalid JSON
- truncated JSON
- unsupported shapes and versions
- missing required fields
- invalid identifiers and signatures
- contradictory claims
- inputs immediately below, at, and above `MAX_JSON_INPUT_BYTES`
- nesting immediately below, at, and above `MAX_JSON_NESTING_DEPTH`

Required properties:

- no panic, abort, or service-wide availability loss
- deterministic machine-readable classification
- bounded CPU, memory, disk, and workspace use
- no accepted partial or contradictory durable state
- no secret-bearing diagnostic output

## 13. Signature-workspace accumulation checks

Before and after every abrupt termination, record the signature-workspace inventory.

After restart:

- normal completed operations must leave no unexpected workspace residue
- abrupt termination residue must be visible to documented diagnostics
- accumulation must remain bounded
- remediation must use documented operator procedures
- remediation must not bypass signature, proof, or subject validation

Any monotonic unexplained growth is a release blocker.

## 14. Safe disk-pressure scenarios

Run at least two controlled disk-pressure scenarios in an isolated test volume or quota-controlled environment.

The scenario must:

- preserve enough host capacity for logs and recovery evidence
- approach the documented low-space threshold without endangering the host
- exercise a write-bearing operation
- observe whether the operation completes atomically or fails closed
- restore free space using test-harness-owned files only
- restart and verify storage, journal, index, archive, proof, and workspace state

The harness must never delete Lingonberry state manually to recover disk space.

Unexpected partial publication, contradictory generation state, missing recovery evidence, or inability to restart safely is failure.

## 15. Abrupt-process-termination scenarios

Use explicit process-level termination, including SIGKILL where safe, rather than uncontrolled physical power loss.

Apply abrupt termination during:

- ordinary publish activity
- index verification or rebuild at documented safe injection points
- backup creation before verification
- replacement and cleanup crash points
- signature verification workspace activity

After each termination:

1. retain system and application logs
2. record filesystem and workspace inventory
3. restart through systemd
4. run doctor/verification commands
5. confirm deterministic recovery or fail-closed classification
6. confirm no accepted contradictory state

Physical power-loss testing is not mandatory unless a safe evidence-preserving harness exists. The residual boundary must be documented in final evidence.

## 16. Telemetry contract

### 16.1 Sampling cadence

Collect normal telemetry at least once per minute. Collect an immediate snapshot before and after disruptive operations.

### 16.2 Required host and service telemetry

Retain at minimum:

- UTC timestamp and soak phase
- service state, PID, restart count, and uptime
- process RSS and virtual memory
- process CPU consumption
- process and service file-descriptor count
- host available memory and swap use
- filesystem used and available bytes/inodes
- storage, index, journal, archive, proof, backup, evidence, quarantine, and workspace byte counts
- file counts for journal, proof, archive, evidence, quarantine, and temporary workspaces
- accepted, duplicate, conflict, deferred, and rejected operation counts
- HTTP status and machine-readable diagnostic counts
- backup, restore, verify, rebuild, replacement, and cleanup result counts
- panic, abort, crash, OOM, restart, and readiness-failure counts

### 16.3 Growth analysis

For every durable or temporary area, the final analysis must distinguish:

- expected growth caused by accepted workload
- bounded reusable workspace
- retained evidence required by policy
- unexpected or unexplained growth

Raw byte growth alone is not failure when it is attributable to accepted durable objects or required evidence. Unexplained monotonic growth, growth after cleanup beyond documented retention, or resource exhaustion is failure.

### 16.4 Resource ceilings

Absolute ceilings must be derived from the candidate's documented configuration and reference-host capacity before execution.

The manifest must state alert and stop thresholds for:

- memory and swap
- file descriptors
- free disk bytes and inodes
- journal/evidence/workspace size
- service restart rate
- health/readiness failure duration

Thresholds may not be invented after observing unfavorable results.

## 17. Stop conditions

Stop the run immediately and preserve evidence when any of the following occurs:

- panic, abort, or unexplained process termination
- OOM kill
- accepted invalid, contradictory, or unverifiable state
- canonical object corruption or identity drift
- unexplained object/index divergence
- backup verification bypass or failed isolated restore
- unrecoverable partial migration, replacement, or cleanup state
- unsafe destructive action without exact authorization and proof binding
- unbounded or threshold-exceeding resource/workspace growth
- repeated readiness failure beyond the predefined threshold
- secret disclosure in logs or diagnostics
- Critical, High, or release-blocking Medium defect

A stopped run cannot be resumed and counted as the same passing soak. After correction, a new candidate-bound run is required.

## 18. Pass criteria

The soak passes only when all required duration, workload, telemetry, and evidence requirements are satisfied and:

- zero panic or abort occurred
- zero canonical corruption or accepted contradictory state occurred
- zero unexplained object/index divergence occurred
- no unbounded resource, journal, evidence, or workspace growth occurred
- all injected failures produced a documented recoverable or fail-closed state
- all backup, restore, verification, and rebuild checks were deterministic
- replacement and cleanup proof/subject binding remained exact
- parser and signature workloads remained bounded
- no Critical, High, or release-blocking Medium defect was discovered
- every deviation and residual risk has explicit disposition

A warning-only result is not automatically a pass. The final evidence must explain why each warning is compatible with this plan and the v1 compatibility policy.

## 19. Evidence format and retention

The run must produce a self-contained evidence bundle containing:

- `manifest.json`: candidate, build, environment, configuration, limits, thresholds, and timestamps
- `timeline.jsonl`: ordered workload and failure-injection events
- `telemetry.csv` or `telemetry.jsonl`: minute-level and disruptive-event telemetry
- `operations/`: machine-readable outputs and exit statuses
- `logs/`: application, systemd, kernel, and harness logs with secrets redacted
- `backups/`: backup manifests, verification results, and digests
- `restores/`: isolated-restore manifests and verification results
- `index/`: verify/rebuild evidence
- `crash-matrix/`: interruption-point evidence
- `security/`: malformed, oversized, nested, and signature-workspace results
- `summary.md`: counts, deviations, residual risks, and pass/fail decision
- `SHA256SUMS`: digest for every retained evidence file

The bundle must be uploaded as an immutable or digest-addressed release qualification artifact. The release evidence document must record the artifact identifier, digest, candidate SHA, and retention endpoint.

Minimum retention is through v2.0.0 publication or two years after v1.0.0 publication, whichever is later. Repository-hosting constraints may require external archival; the final evidence must identify the authoritative location.

## 20. Execution roles and approvals

The execution record must identify:

- qualification operator
- candidate/build approver
- security findings reviewer
- final release decision owner

The same person may perform multiple roles, but the evidence must show each decision explicitly.

## 21. Final release evidence

`docs/roadmap/V1_0_RELEASE_EVIDENCE.md` must record:

- candidate commit and artifact digests
- soak start/end and counted duration
- workload totals
- telemetry thresholds and observed maxima
- every failure injection and outcome
- every deviation and residual risk
- evidence-bundle identifier and digest
- open defect counts by severity
- final PASS or FAIL decision

A PASS decision applies only to the exact candidate and evidence bundle named in that document.

## 22. Completion checklist

Before execution:

- [ ] Candidate and build provenance template is complete
- [ ] Reference environment and systemd configuration are recorded
- [ ] Resource ceilings and stop thresholds are fixed
- [ ] Harness commands and failure injection points are reviewed
- [ ] Evidence paths and retention location are prepared
- [ ] Security and destructive-operation safeguards are reviewed

During execution:

- [ ] Duration and workload counters are retained
- [ ] Minute-level telemetry is continuous
- [ ] Disruptive-event snapshots are retained
- [ ] Backup, restore, index, and crash-matrix cadence is satisfied
- [ ] Deviations are recorded immediately

After execution:

- [ ] Evidence bundle digests verify
- [ ] Telemetry growth analysis is complete
- [ ] Defects and residual risks have explicit disposition
- [ ] `V1_0_RELEASE_EVIDENCE.md` records the final decision
- [ ] The release candidate remains unchanged since qualification

## 23. Release blockers

v1.0.0 publication is blocked when:

- this plan is not reviewed before execution
- candidate or environment provenance is incomplete
- duration or workload minima are not met
- required telemetry or evidence is missing
- a stop condition occurred
- a pass criterion is not demonstrated
- a deviation lacks explicit disposition
- the evidence bundle cannot be verified
- the qualified candidate differs from the release candidate
