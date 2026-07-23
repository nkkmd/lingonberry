# Systemd Service Contract

**Status: active** | **Reference platform: Ubuntu Server 24.04 LTS / x86_64 / systemd** | **Last updated: 2026-07-23**

## Purpose

This document explains how Lingonberry's checked-in systemd units are used and maintained. It does not duplicate the unit contents. The normative service definitions are:

- [`deploy/systemd/lingonberry-storage-ready.service`](../../deploy/systemd/lingonberry-storage-ready.service)
- [`deploy/systemd/lingonberry-relay.service`](../../deploy/systemd/lingonberry-relay.service)

When this document, an older runbook, or a locally copied example differs from those files, the checked-in files take precedence.

## Service model

Lingonberry's reference single-node deployment uses two units:

1. `lingonberry-storage-ready.service` is a oneshot readiness gate. It validates the storage environment before the relay starts. It is not a long-running storage daemon.
2. `lingonberry-relay.service` is the long-running HTTP relay. It requires the storage readiness gate and starts only after that gate succeeds.

The relay dependency is explicit through `Requires=` and `After=`. Operators must not replace the readiness gate with an unconditional ordering-only dependency.

## Installed paths

The reference deployment expects:

```text
/usr/local/bin/lingonberry-storage
/usr/local/bin/lingonberry-relay
/etc/lingonberry/storage.env
/etc/lingonberry/relay.env
/etc/systemd/system/lingonberry-storage-ready.service
/etc/systemd/system/lingonberry-relay.service
/var/lib/lingonberry
/var/backups/lingonberry
```

The service account is `lingonberry:lingonberry`.

## Installation

Install the checked-in units without editing them in place:

```bash
sudo install -D -m 0644 \
  deploy/systemd/lingonberry-storage-ready.service \
  /etc/systemd/system/lingonberry-storage-ready.service
sudo install -D -m 0644 \
  deploy/systemd/lingonberry-relay.service \
  /etc/systemd/system/lingonberry-relay.service
sudo systemctl daemon-reload
```

Place host-specific values in the environment files rather than modifying `ExecStart` or embedding secrets in unit files.

Validate the installed units before enabling them:

```bash
systemd-analyze verify \
  /etc/systemd/system/lingonberry-storage-ready.service \
  /etc/systemd/system/lingonberry-relay.service
sudo systemctl cat lingonberry-storage-ready.service
sudo systemctl cat lingonberry-relay.service
```

Use the operator runbook for directory creation, environment-file permissions, binary installation, and lifecycle commands.

## Security and filesystem boundaries

The checked-in units deliberately include:

- an unprivileged dedicated user and group;
- optional environment files under `/etc/lingonberry`;
- `NoNewPrivileges=yes`;
- `PrivateTmp=yes`;
- `ProtectSystem=strict`;
- `ProtectHome=yes`;
- explicit writable-path allowlists;
- bounded stop behavior for the relay;
- restart-on-failure for the relay only.

Do not remove hardening directives merely to make a local path layout work. Change the host layout or add a reviewed systemd drop-in with the narrowest required permission.

The readiness gate may write only to the state and backup roots allowed by its unit. The relay may write only under the Lingonberry state root allowed by its unit. Active canonical storage must remain on a supported local filesystem as defined by [`SUPPORTED_PLATFORMS.md`](./SUPPORTED_PLATFORMS.md).

## Environment files

Environment files are configuration inputs, not shell scripts. Keep them root-owned and non-world-readable when they contain sensitive values.

Typical installation:

```bash
sudo install -d -m 0750 -o root -g lingonberry /etc/lingonberry
sudo install -m 0640 -o root -g lingonberry storage.env /etc/lingonberry/storage.env
sudo install -m 0640 -o root -g lingonberry relay.env /etc/lingonberry/relay.env
```

Do not place credentials directly in unit files, command-line arguments, repository-tracked files, or public diagnostics.

## Start, verify, and stop

```bash
sudo systemctl enable --now lingonberry-storage-ready.service
sudo systemctl enable --now lingonberry-relay.service
systemctl is-active lingonberry-storage-ready.service
systemctl is-active lingonberry-relay.service
journalctl -u lingonberry-storage-ready.service -u lingonberry-relay.service --since today
```

Before treating the node as ready, also run the application-level checks defined in [`V1_0_OPERATOR_RUNBOOK.md`](./V1_0_OPERATOR_RUNBOOK.md).

For a controlled stop:

```bash
sudo systemctl stop lingonberry-relay.service
sudo systemctl stop lingonberry-storage-ready.service
```

## Customization policy

Prefer systemd drop-ins over copying and editing the full unit:

```bash
sudo systemctl edit lingonberry-relay.service
sudo systemctl daemon-reload
systemd-analyze verify /etc/systemd/system/lingonberry-relay.service
```

A customization is outside the reference release contract when it changes any of the following without equivalent review and acceptance evidence:

- service user or group;
- dependency on the storage readiness gate;
- executable path or command mode;
- writable-path boundaries;
- hardening directives;
- stop timeout or signal;
- restart policy;
- environment-file location;
- network exposure.

Local drop-ins must be included in upgrade review, rollback planning, incident evidence, and qualification manifests. A hosted CI pass for the repository unit does not validate an unreviewed host-local drop-in.

## Reverse proxy boundary

Lingonberry does not maintain a Caddy service unit in `deploy/systemd/`. Install and operate Caddy, nginx, or another reverse proxy using the distribution or vendor-supported service definition. Keep the relay bound to the intended internal address and follow [`CADDY_RELAY_PUBLICATION.md`](./CADDY_RELAY_PUBLICATION.md) for publication guidance.

Do not copy the obsolete Caddy example that previously appeared in this document and treat it as part of the Lingonberry service contract.

## Change control

A change to either checked-in unit requires, at minimum:

1. a focused pull request showing the unit diff;
2. `systemd-analyze verify` on the reference platform;
3. operator-acceptance validation;
4. documentation and environment-contract review;
5. upgrade and rollback impact review;
6. requalification when the change affects the frozen candidate or formal host contract.

The designated v1.0.0 pre-version candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Documentation and tooling changes after that commit do not redefine the candidate. Formal reference-host qualification, the 72-hour soak, version update, tag, and GitHub Release remain pending.

## Related documents

- [`V1_0_OPERATOR_RUNBOOK.md`](./V1_0_OPERATOR_RUNBOOK.md)
- [`SUPPORTED_PLATFORMS.md`](./SUPPORTED_PLATFORMS.md)
- [`OPERATOR_CLI_CONTRACT.md`](./OPERATOR_CLI_CONTRACT.md)
- [`SECRET_MANAGEMENT.md`](./SECRET_MANAGEMENT.md)
- [`CADDY_RELAY_PUBLICATION.md`](./CADDY_RELAY_PUBLICATION.md)
