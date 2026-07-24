# Container Execution Templates

**Status: v1.0 pre-release normative deployment guidance**  
**Last reviewed: 2026-07-24**

This document defines the minimum container-execution boundaries for Lingonberry v1.0. English is normative.

## 1. Scope and support boundary

The checked-in systemd units remain the reference host integration. Container execution is optional and is supported only when it preserves the same application binaries, configuration precedence, persistent-storage layout, readiness ordering, secret handling, network separation, and evidence requirements.

The repository does not currently define a normative Dockerfile, image registry, Compose file, Kubernetes manifest, or container orchestrator. Image names in this document are operator-supplied, immutable images that contain the intended Lingonberry binaries. An image tag alone is not release evidence; record the image digest and application commit.

Containerization does not change these runtime facts:

- `lingonberry-storage ready` is a finite readiness gate, not a resident storage daemon;
- `lingonberry-storage run` prints a resolved runtime snapshot and exits;
- `lingonberry-relay serve-http` is the long-running public relay process;
- migration, restore, backup, archive import, and quarantine administration are separate controlled operations;
- relay HTTP readiness is not deep storage verification.

## 2. Required filesystem model

Resolve and persist the same storage paths used outside containers:

```text
stateDir
 dataDir
 backupDir
 tempDir
```

Use the implemented storage environment variables rather than the obsolete generic `LINGONBERRY_STATE_DIR` example:

```text
LINGONBERRY_STORAGE_CONFIG
LINGONBERRY_STORAGE_STATE_DIR
LINGONBERRY_STORAGE_DATA_DIR
LINGONBERRY_STORAGE_BACKUP_DIR
LINGONBERRY_STORAGE_TEMP_DIR
```

Active data and backups must not live only in the writable container layer. Mount them from supported host-local storage or equivalent explicitly qualified persistent storage. Keep the backup root separate from the active data root.

## 3. Storage readiness gate

Run the storage gate with the same configuration and mounts that the relay will use:

```bash
docker run --rm \
  --user 10001:10001 \
  --env-file /etc/lingonberry/storage.env \
  --mount type=bind,src=/var/lib/lingonberry,dst=/var/lib/lingonberry \
  --mount type=bind,src=/var/backups/lingonberry,dst=/var/backups/lingonberry \
  <storage-image-by-digest> \
  lingonberry-storage ready
```

Before first startup, upgrade, restore, or qualification, also inspect the resolved configuration and diagnostics:

```bash
docker run --rm ... <storage-image-by-digest> lingonberry-storage config
docker run --rm ... <storage-image-by-digest> lingonberry-storage doctor
docker run --rm ... <storage-image-by-digest> lingonberry-storage verify
```

`ready` and `doctor` permit warning-only reports. `verify` fails on warnings and failed checks. Do not treat a successful container start, image pull, or process exit as equivalent to these checks.

## 4. Relay process

Start the relay only after the storage readiness gate succeeds:

```bash
docker run --rm \
  --name lingonberry-relay \
  --user 10001:10001 \
  --env-file /etc/lingonberry/relay.env \
  --mount type=bind,src=/var/lib/lingonberry,dst=/var/lib/lingonberry \
  --publish 127.0.0.1:8787:8787 \
  --read-only \
  --tmpfs /tmp:rw,nosuid,nodev,noexec \
  <relay-image-by-digest> \
  lingonberry-relay serve-http 0.0.0.0:8787
```

The example publishes the relay only on the host loopback address so that a separately managed reverse proxy can provide public TLS. Do not publish the administrator listener through the public proxy. Give the relay only the writable paths it requires; it must not receive backup-root write access merely because the readiness gate has it.

The container supervisor or orchestrator must encode the dependency on a successful storage gate. A simple start order without failure propagation is insufficient.

## 5. Reverse proxy

Caddy is not a Lingonberry carrier or a checked-in Lingonberry container image. Operate it using a reviewed vendor or distribution image and preserve the `/v1` request path, method, body, and signature material.

```bash
docker run --rm \
  --name lingonberry-caddy \
  --publish 80:80 \
  --publish 443:443 \
  --mount type=bind,src=/etc/caddy/Caddyfile,dst=/etc/caddy/Caddyfile,ro \
  --mount type=volume,src=caddy_data,dst=/data \
  --mount type=volume,src=caddy_config,dst=/config \
  <reviewed-caddy-image-by-digest> \
  caddy run --config /etc/caddy/Caddyfile --adapter caddyfile
```

Validate the Caddy configuration before replacement or reload. Follow [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md); in particular, do not use a path handler that strips the required `/v1` prefix.

## 6. Secrets and identity

- Inject environment files or orchestrator secrets at runtime; do not bake them into images.
- Do not put administrator Bearer tokens, private keys, or sensitive values in command-line arguments, image labels, Compose files committed to the repository, or evidence bundles.
- Publisher signatures are request material and are not administrator authentication.
- Run containers as a non-root UID/GID with ownership matching the mounted paths.

## 7. Network boundary

Use separate exposure surfaces:

- public reverse proxy: Internet-facing `80`/`443` where applicable;
- public relay: internal or host-loopback listener;
- administrator listener: separate private binding with configured role credentials;
- storage paths: filesystem access only, not a public network service.

Container DNS, bridge networking, or an orchestrator service name does not itself provide authentication, authorization, TLS, or tenant isolation.

## 8. Lifecycle and mutation rules

Before migration, restore, or replacement cleanup, stop every relay instance that can write the active storage. Ordinary container startup or restart must never perform an implicit migration.

Backups, archive exports, migration backups, qualification evidence, and formal-soak evidence are distinct artifacts. A named volume is not a verified backup. Test restore and rollback using the governing runbooks.

Use normal termination (`SIGTERM`) and a bounded stop timeout. Preserve logs and state needed to classify an incomplete write or failed shutdown.

## 9. Verification

After startup, verify in this order:

1. storage `config`;
2. storage `ready` and `doctor`;
3. storage `verify` when warnings must block the operation;
4. relay CLI readiness;
5. `GET /v1/ready` and `GET /v1/capabilities` through the internal listener;
6. the same HTTP checks through the public proxy;
7. one signed publication, duplicate replay, retrieval, and classified rejection;
8. administrator-route isolation from the public listener;
9. persistent state after a controlled container replacement.

Record image digests, application commit, container runtime and version, host kernel, mounts, network bindings, resolved configuration, and complete command outcomes. A disposable container walkthrough is not privileged reference-host qualification and is not formal 72-hour soak evidence.

## 10. Compatibility and change control

A container template requires compatibility and qualification review when it changes:

- the application binary or command mode;
- configuration precedence or environment names;
- UID/GID or filesystem ownership;
- active, backup, or temporary path mounts;
- relay startup dependency on storage readiness;
- public or administrator listener exposure;
- secret injection;
- stop/restart behavior;
- image provenance or base image;
- host-local versus remote storage semantics.

The fixed v1.0.0 pre-version candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Documentation and tooling commits after that candidate do not redefine it. Formal reference-host qualification, the 72-hour soak, version update, release PR, tag, and GitHub Release remain pending.

## Related documents

- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
- [Relay and Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
