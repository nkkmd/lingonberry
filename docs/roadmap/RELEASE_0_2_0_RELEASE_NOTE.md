# Lingonberry v0.2.0 Release Notes

**Status: released** | **Release date: 2026-07-12**

Lingonberry v0.2.0 advances the project from a protocol and bootstrap implementation release to an operationally safer quarantine lifecycle and administration release.

## Highlights

### Persistent quarantine lifecycle

Invalid or deferred objects can be retained in append-only quarantine state, reviewed, annotated, revalidated, promoted, dismissed, or permanently rejected without deleting the original evidence.

The release includes:

- single and batch promotion;
- dry-run revalidation;
- append-only reviewer annotations;
- manual dismissal;
- permanent rejection;
- status reporting and Prometheus metrics;
- scheduled revalidation support.

### Verified ledger maintenance

Quarantine ledgers now support:

- exact managed-ledger indexing;
- archive-aware ordered reads;
- immutable, byte-preserving rotation;
- archive segment verification;
- archive-inclusive backup v2;
- verified restore;
- non-destructive compaction preview and semantic proof.

Compaction policy v1 deliberately authorizes no record rewriting or deletion. Every valid line remains retained.

### Role-scoped administration

The dedicated admin HTTP listener supports three independent roles:

- `observer`: read-only status, metrics, records, resolutions, annotations, and permanent-rejection state;
- `reviewer`: observer permissions plus annotation creation;
- `operator`: reviewer permissions plus promotion and permanent rejection.

Missing or invalid credentials return `401 Unauthorized`. Authenticated credentials without permission return `403 Forbidden` before request bodies are read.

### Legacy token migration

`LINGONBERRY_ADMIN_TOKEN` is deprecated as an operator fallback. Deployments should use:

```text
LINGONBERRY_ADMIN_OBSERVER_TOKEN
LINGONBERRY_ADMIN_REVIEWER_TOKEN
LINGONBERRY_ADMIN_OPERATOR_TOKEN
```

Run the secret-free diagnostic in the service environment:

```bash
lingonberry-admin-auth-config
```

The legacy fallback remains available in v0.2.0 and is targeted for removal only in a future major release after the documented migration conditions are met.

## Upgrade notes

1. Update all Lingonberry workspace packages together to v0.2.0.
2. Configure explicit role tokens for the admin listener.
3. Run `lingonberry-admin-auth-config` and confirm `legacyOperatorFallbackActive` is `false`.
4. Before rotating quarantine ledgers, build a fresh index.
5. Use backup v2 before maintenance or operational migration.
6. Keep `quarantine-segments.json` and `quarantine-segments/` with the active ledgers.

Existing backup v1 manifests remain accepted by verify and restore. New exports use backup v2.

## Release validation

The release branch requires:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The JavaScript canonicalization, identity, and validation tests must also pass.

Operational smoke tests are recorded in `RELEASE_0_2_0_CHECKLIST.md`.

## Deferred work

The following are not release blockers and remain deferred:

- record-rewriting compaction;
- retention deletion;
- distributed locking or multi-node consensus;
- remote backup upload;
- backup encryption or cryptographic signing;
- OAuth/OIDC;
- browser sessions and per-record ACLs;
- removal of the deprecated legacy admin token fallback.

## Tag and release

The annotated tag and GitHub Release are published as `v0.2.0` / `Lingonberry v0.2.0`.

Development after this release is tracked in `RELEASE_0_3_0_ROADMAP.md`.
