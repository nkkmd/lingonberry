# v1.0.0 Candidate-Bound Crash-Matrix Driver

**Status: active-candidate rehearsal implementation**  
**Candidate:** `8c6b48082205a3af555130eec1f3e7d2ac8811fe`  
**Last updated:** 2026-07-27

## Purpose

The formal soak requires six complete replacement and cleanup crash-matrix cycles. No public operator CLI exposes the complete failure-injection matrix. The qualification surface is therefore the integration-test binary built from the exact fixed candidate.

This does not redefine the public operator contract. It provides candidate-bound evidence for a non-public maintenance and recovery surface already covered by the release qualification suite.

## Active launcher

- `scripts/run-v1-crash-matrix-driver.sh`

The reviewed implementation is retained in:

- `scripts/run-v1-crash-matrix-driver-legacy.sh`

The launcher fixes `CANDIDATE_SHA` to the active candidate and rejects any different value before delegating to the retained implementation.

## Evidence binding

The driver must:

1. verify the exact candidate commit and a clean candidate checkout;
2. record the `Cargo.lock` digest and Rust toolchain identity;
3. build `quarantine_replacement_crash_matrix` once with `cargo test --no-run --message-format=json`;
4. identify exactly one executable test binary from Cargo metadata;
5. record and continuously re-check the binary SHA-256;
6. enumerate exactly the four expected tests before any cycle;
7. execute the same binary with `--test-threads=1` for every cycle;
8. retain per-cycle stdout, stderr, exit status, timestamps, and checksums;
9. reject missing, added, filtered, partial, or digest-mismatched cycles;
10. reject reuse of an existing evidence directory.

## Expected tests

- `injected_journal_failures_leave_a_retryable_empty_workspace`
- `injected_commit_transition_failure_resumes_without_second_switch`
- `injected_rollback_pointer_restore_failure_keeps_target_until_retry`
- `injected_rolled_back_transition_failure_retries_after_pointer_restore`

One successful invocation containing all four tests is one crash-matrix cycle.

## Rehearsal boundary

The CI rehearsal checks out candidate `8c6b4808…`, executes two cycles, verifies bundle checksums, and proves a candidate mismatch fails closed. It always records:

```json
{
  "qualification": false,
  "qualifyingPass": false
}
```

The formal soak requires at least six cycles distributed throughout the 72-hour schedule. Rehearsal success does not start or pass the formal soak.
