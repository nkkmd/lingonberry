# v0.2.0 Release Checklist

**Status: release candidate** | **Last updated: 2026-07-12**

This checklist is the release gate for Lingonberry v0.2.0. The release preparation pull request may be merged only when automated checks pass. The `v0.2.0` tag may be created only after the merged `main` commit also passes CI.

## Release boundary

- [x] Persistent quarantine lifecycle is included.
- [x] Verified ledger rotation and archive-aware reads are included.
- [x] Archive-inclusive backup v2, verify, and restore are included.
- [x] Non-destructive compaction preview and proof are included.
- [x] Dedicated admin listener and observer/reviewer/operator RBAC are included.
- [x] Legacy token deprecation diagnostics are included.
- [x] Record-rewriting compaction and retention deletion are explicitly deferred.
- [x] Distributed locking and multi-node shared-state support are explicitly deferred.
- [x] The legacy admin-token fallback remains compatible and is not removed in this minor release.

## Version consistency

- [x] `lingonberry-protocol` is `0.2.0`.
- [x] `lingonberry-identity` is `0.2.0`.
- [x] `lingonberry-validation` is `0.2.0`.
- [x] `lingonberry-core` is `0.2.0`.
- [x] `lingonberry-indexer` is `0.2.0`.
- [x] `lingonberry-relay` is `0.2.0`.
- [x] `lingonberry-storage` is `0.2.0`.
- [x] `Cargo.lock` records all workspace packages as `0.2.0`.
- [ ] The final `v0.2.0` tag points to the CI-green merged release commit.

## Automated validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] JavaScript canonicalization tests
- [ ] JavaScript identity tests
- [ ] JavaScript validation tests
- [ ] Pull request is mergeable and all required checks are green
- [ ] Merged `main` is green before tagging

The pull-request CI result may satisfy the first seven items. The final `main` check and tag item must be completed after merge.

## Publish and storage smoke tests

Run from a clean checkout:

```bash
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
cargo run -p lingonberry-relay -- export-archive /tmp/lingonberry-v020-archive
cargo run -p lingonberry-relay -- import-archive /tmp/lingonberry-v020-archive
cargo run -p lingonberry-storage -- capabilities
cargo run -p lingonberry-storage -- ready
```

- [ ] Minimal publish succeeds.
- [ ] Invalid fixtures remain rejected.
- [ ] Archive export/import succeeds.
- [ ] Storage capabilities and readiness succeed.

## Quarantine lifecycle smoke tests

Use an isolated state directory:

```bash
export LINGONBERRY_STATE_DIR=/tmp/lingonberry-v020-state
rm -rf "$LINGONBERRY_STATE_DIR"
mkdir -p "$LINGONBERRY_STATE_DIR"
```

- [ ] A rejected or deferred object is persisted to quarantine.
- [ ] `quarantine-status` reports pending state.
- [ ] An annotation can be appended and listed.
- [ ] A pending record can be promoted by the operator path.
- [ ] A different pending record can be dismissed.
- [ ] A different pending record can be permanently rejected.
- [ ] Promoted, dismissed, and permanently rejected records remain append-only evidence.
- [ ] Status and metrics reflect terminal lifecycle counts.

## Ledger maintenance and recovery smoke tests

```bash
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
lingonberry-quarantine-maintenance rotate quarantine.jsonl
lingonberry-quarantine-maintenance verify-segments

rm -rf /tmp/lingonberry-v020-backup /tmp/lingonberry-v020-restored
lingonberry-quarantine-backup export /tmp/lingonberry-v020-backup
lingonberry-quarantine-backup verify /tmp/lingonberry-v020-backup
lingonberry-quarantine-backup restore \
  /tmp/lingonberry-v020-backup \
  /tmp/lingonberry-v020-restored
```

- [ ] Fresh index verification succeeds.
- [ ] Rotation preserves the ordered logical stream.
- [ ] Segment verification succeeds.
- [ ] Backup v2 includes active ledgers, segment manifest, and listed segments.
- [ ] Backup verification succeeds.
- [ ] Restore into an empty state directory succeeds.
- [ ] Restored segment verification and quarantine status succeed.

## Compaction proof smoke test

```bash
rm -rf /tmp/lingonberry-v020-proof
lingonberry-quarantine-maintenance compaction-preview \
  /tmp/lingonberry-v020-backup \
  /tmp/lingonberry-v020-proof
lingonberry-quarantine-maintenance verify-compaction-proof \
  /tmp/lingonberry-v020-proof
```

- [ ] Proof verification succeeds.
- [ ] `mutationAllowed` is `false`.
- [ ] `rewritePerformed` is `false`.
- [ ] `removableLines` is zero.
- [ ] Runtime state is unchanged.

## Admin RBAC smoke tests

Configure three distinct secrets:

```bash
export LINGONBERRY_ADMIN_OBSERVER_TOKEN=<observer-secret>
export LINGONBERRY_ADMIN_REVIEWER_TOKEN=<reviewer-secret>
export LINGONBERRY_ADMIN_OPERATOR_TOKEN=<operator-secret>
```

- [ ] Public listener returns `404` for admin routes.
- [ ] Missing and invalid admin credentials return equivalent `401` responses.
- [ ] Observer reads succeed.
- [ ] Observer mutations return `403`.
- [ ] Reviewer annotation creation succeeds.
- [ ] Reviewer promotion and permanent rejection return `403`.
- [ ] Operator promotion and permanent rejection succeed.
- [ ] Unauthorized mutation bodies are not interpreted before denial.
- [ ] Authentication audit uses `role: null`.
- [ ] Authorization audit records only the bounded resolved role.
- [ ] Audit records contain no token, body, note, or quarantine payload.

## Legacy token migration diagnostic

```bash
lingonberry-admin-auth-config
```

- [ ] Explicit role configuration reports `legacyOperatorFallbackActive: false`.
- [ ] Explicit role configuration reports `secretsIncluded: false`.
- [ ] A legacy-only test environment reports `LB_ADMIN_LEGACY_TOKEN_DEPRECATED`.
- [ ] A legacy-only test environment reports `actionRequired: true`.
- [ ] Diagnostic output contains no secret or credential fingerprint.

## Documentation and repository hygiene

- [x] `CHANGELOG.md` contains the v0.2.0 entry.
- [x] v0.2.0 release notes exist.
- [x] Current implementation status reflects merged functionality.
- [x] Roadmap index no longer lists implemented quarantine/RBAC work as missing.
- [x] Known limitations are explicit.
- [ ] Relative documentation links are checked.
- [ ] Repository contains no credentials, generated runtime state, or temporary release files.
- [ ] `git status --short` is clean after final validation.

## Final release

After the release PR is merged and `main` is green:

```bash
git switch main
git pull --ff-only
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git tag -a v0.2.0 -m "Lingonberry v0.2.0"
git push origin v0.2.0
```

- [ ] Annotated tag `v0.2.0` is pushed.
- [ ] GitHub Release `Lingonberry v0.2.0` is published.
- [ ] Release notes link to the changelog and migration documentation.
