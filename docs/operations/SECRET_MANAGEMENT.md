# Secret Management Contract

**Status: v1.0.0 pre-release**  
**Normative language: English**

This document defines the secret-handling boundary for the Lingonberry v1.0.0 pre-release line. It covers implemented administrator credentials, systemd environment injection, audit behavior, rotation, and evidence redaction. It does not define a universal secret backend.

Lingonberry v1.0.0 has not been published. The designated pre-version candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Evidence and documentation commits after that candidate do not redefine it.

## 1. Core rules

Secrets must not be placed in:

- protocol objects or carrier metadata;
- canonical storage records;
- `storage-config.json` or other ordinary configuration committed to source control;
- checked-in environment templates;
- command examples, screenshots, issue comments, pull-request text, or release evidence;
- filenames, labels, metric dimensions, or audit fields.

Secrets must be injected by the deployment environment at process start. The repository does not require a specific external secret store.

## 2. Implemented secret surface

The public relay and storage readiness gate do not require authentication credentials for their basic checked-in reference-node startup paths.

The administrator HTTP surface requires at least one configured token when `serve-admin-http` is used. The implemented role variables are:

```text
LINGONBERRY_ADMIN_OBSERVER_TOKEN
LINGONBERRY_ADMIN_REVIEWER_TOKEN
LINGONBERRY_ADMIN_OPERATOR_TOKEN
```

The compatibility variable is:

```text
LINGONBERRY_ADMIN_TOKEN
```

`LINGONBERRY_ADMIN_TOKEN` is a legacy operator fallback. It is used only when no role-specific operator token is configured. New deployments should use the role-specific variables.

Configured values:

- must not be empty after trimming;
- must be pairwise distinct across configured roles;
- are compared to bearer credentials with the implementation's constant-time comparison routine;
- must never be printed as part of normal startup, authorization, or audit output.

The administrator listener refuses startup when no accepted credential is configured.

## 3. Role boundary

The implemented authorization hierarchy is:

| Role | Observe | Annotate | Operate |
|---|---:|---:|---:|
| `observer` | yes | no | no |
| `reviewer` | yes | yes | no |
| `operator` | yes | yes | yes |

Typical observe operations include authenticated metrics and quarantine reads. Annotation requires reviewer or operator authority. Promotion, permanent rejection, and other mutating administrator operations require operator authority.

A bearer token grants only the role associated with the matching configured credential. Operators must not reuse one value for multiple roles.

## 4. Injection with the reference systemd units

The checked-in units read optional environment files:

```text
/etc/lingonberry/relay.env
/etc/lingonberry/storage.env
```

The checked-in templates contain paths and listen settings only; they do not contain secret placeholders or example token values.

For a deployment that enables the administrator listener, credentials may be supplied through a root-managed environment file or another systemd-supported secret-injection mechanism. The deployment must ensure that:

- the file is not tracked by Git;
- the file owner and mode restrict reading to the intended administrative boundary;
- unprivileged users cannot modify the file;
- backups, support bundles, and configuration captures exclude the values;
- shell history does not receive literal tokens;
- process restart is coordinated after a credential change.

A conventional root-managed file should normally be owned by `root` and not be world-readable. Exact ownership and mode remain deployment decisions because the repository does not install host files directly.

The leading `-` in the checked-in `EnvironmentFile=-...` directives means a missing file does not itself fail unit loading. It does not make missing administrator credentials acceptable when the administrator listener is started.

## 5. Do not pass tokens on command lines

Operators must not place bearer tokens directly in:

- unit `ExecStart` arguments;
- shell command history;
- process titles;
- URLs or query parameters;
- test names or fixture paths.

HTTP clients should send credentials through the `Authorization` header:

```text
Authorization: Bearer <token>
```

Examples and evidence must use placeholders rather than live values.

## 6. Audit behavior

Authentication and authorization failures are appended to:

```text
<state-dir>/admin-auth-audit.jsonl
```

The implemented audit record may contain:

- attempted time;
- remote address;
- HTTP method;
- request path;
- resolved role when available;
- outcome code.

The bearer token is not an audit field and must not be added to the audit record.

Audit records are sensitive operational data even though they exclude tokens. They may reveal administrator endpoints, source addresses, activity timing, and role usage. Access, retention, export, and deletion must follow the node's operational policy.

## 7. Logging and observability

Normal logs may identify which environment-variable name or compatibility path is active, but must not include its value.

The administrator server currently emits a warning when the legacy `LINGONBERRY_ADMIN_TOKEN` operator fallback is active. That warning is safe only because it identifies the variable name and compatibility state, not the token.

Before sharing logs or evidence, operators must inspect for:

- `Authorization` headers;
- copied environment-file contents;
- shell traces such as `set -x`;
- command invocations containing literal credentials;
- core dumps or diagnostic archives containing process environments.

See [Observability Contract](./OBSERVABILITY.md) for the implemented logging and evidence boundary.

## 8. Rotation procedure

Credential rotation must be treated as a controlled deployment change.

1. Generate a new high-entropy value outside the repository.
2. Update the deployment secret source without exposing the value in terminal capture or review text.
3. Confirm that role tokens remain pairwise distinct.
4. Restart the administrator listener so the new environment is loaded.
5. Verify access with the new credential at the minimum required role.
6. Verify that the old credential is rejected.
7. Preserve only redacted evidence of the result.
8. Revoke and securely remove the old value from the deployment secret source.

The current implementation loads administrator credentials at process startup. Editing an environment file without restarting the administrator listener does not rotate the in-memory credential set.

For emergency revocation, stop or isolate the administrator listener before changing credentials when continued access with the old token is unacceptable.

## 9. Backup, restore, migration, and release evidence

Storage backup and migration artifacts must not include deployment environment files unless the operator has deliberately created a separate protected host-configuration backup. Lingonberry storage backups are not a secret-management system.

Release qualification, soak, crash, disk-pressure, and documentation-walkthrough evidence must contain:

- command names and exit classifications;
- redacted configuration identity where needed;
- artifact digests and candidate bindings;
- no bearer tokens or environment-file contents.

If secret exposure is suspected, do not publish the affected evidence. Quarantine it, rotate the credential, produce a sanitized replacement, and record the incident without reproducing the value.

## 10. Development and testing

Tests must use synthetic credentials that cannot authorize a deployed node. Test values must be obviously non-production and scoped to disposable environments.

Developers must not copy production environment files into a repository checkout. Local `.env` or override files containing credentials must remain untracked and should be covered by local exclusion controls.

A passing test with a synthetic token does not prove that production file ownership, rotation, or evidence-redaction controls are correct.

## 11. Non-goals

This contract does not provide:

- a built-in vault or hardware-security-module integration;
- automatic token issuance, expiry, or rotation;
- encrypted environment-file storage;
- multi-party approval for administrator actions;
- credential recovery from Lingonberry backups;
- a guarantee that arbitrary host-level debugging tools cannot read process environments.

These controls may be added by the deployment platform without changing protocol semantics.

## 12. Related documents

- [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)
- [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Observability Contract](./OBSERVABILITY.md)
- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
