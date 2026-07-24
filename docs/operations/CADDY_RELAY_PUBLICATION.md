# Caddy Relay Publication

**Status: v1.0 pre-release normative**

This document defines the supported deployment boundary for publishing the Lingonberry reference relay through Caddy. English is normative.

## 1. Scope

Caddy is an external reverse proxy and TLS termination layer. It is not a Lingonberry carrier, protocol component, storage backend, acceptance-policy engine, or administrator authorization layer.

The Lingonberry repository does not ship or maintain a Caddy systemd unit. Operators must install and operate Caddy through an appropriate distribution or vendor-supported service definition.

## 2. Deployment boundary

The reference publication topology is:

```text
Internet or controlled client network
        |
        v
Caddy: public TLS, host routing, request limits, access logs
        |
        v
Lingonberry public relay listener: internal address
        |
        v
Lingonberry storage backend
```

The public relay listener exposes the implemented public HTTP carrier routes. The authenticated quarantine administrator listener, where enabled, is a separate process surface and must not be made public merely because the public relay is proxied.

The storage readiness command is a oneshot validation gate, not a long-running storage daemon and not a network upstream for Caddy.

## 3. Required exposure rules

Operators must apply all of the following:

- expose the Caddy address, not the relay's internal address;
- bind the relay to loopback or an explicitly controlled internal interface;
- prevent direct external access to the relay listener with host firewall or network policy;
- keep the administrator listener on a separate restricted address and access path;
- preserve the HTTP method, path, body, and publisher signature material without application-layer rewriting;
- use HTTPS for Internet-facing publication;
- validate and reload Caddy configuration through the installed Caddy service tooling;
- record the Caddy configuration and host-local overrides in deployment and qualification evidence.

A reverse proxy does not make an unqualified host suitable for v1.0 production publication.

## 4. Public route boundary

The core public relay routes include:

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/v1/ready` | Confirm that the public listener accepted and routed a readiness request |
| `GET` | `/v1/capabilities` | Read implemented capability discovery |
| `POST` | `/v1/objects` | Publish one signed HTTP publish-request envelope |
| `GET` | `/v1/objects/<canonical-id>` | Retrieve one stored object |

Additional transition and effective-view routes are governed by their checked-in contracts and capability surface.

Caddy must not synthesize success responses for these paths. It must forward the relay's actual HTTP status and response body.

## 5. Minimal same-host configuration

The following example publishes a relay bound to `127.0.0.1:8787`:

```caddyfile
relay.example.org {
    reverse_proxy 127.0.0.1:8787
}
```

Replace `relay.example.org` with the controlled public hostname. Ensure DNS and inbound network policy are configured before relying on automatic certificate issuance.

Do not use `handle_path /v1/*` for the relay unless the resulting upstream path has been verified. `handle_path` strips the matched prefix, which would change `/v1/ready` into `/ready` and break the Lingonberry route contract. To restrict routing without stripping the prefix, use a matcher with `handle` or `reverse_proxy` directly.

For example:

```caddyfile
relay.example.org {
    @lingonberry path /v1/*
    handle @lingonberry {
        reverse_proxy 127.0.0.1:8787
    }

    handle {
        respond "Not Found" 404
    }
}
```

## 6. Recommended publication controls

The exact external controls depend on the deployment environment, but an Internet-facing configuration should explicitly review:

- maximum accepted request-body size;
- request and idle timeouts;
- connection and request-rate controls;
- trusted proxy and client-address handling;
- access-log destination, rotation, and retention;
- sensitive-header redaction;
- TLS policy and certificate lifecycle;
- firewall rules preventing relay bypass;
- monitoring of Caddy and relay process availability.

These controls belong to deployment policy. Their presence must not be inferred from Lingonberry's bounded reference HTTP parser or from a successful `/v1/ready` request.

## 7. Logging and sensitive data

Caddy access logs may contain public paths, canonical identifiers, client addresses, response statuses, and request metadata. Operators must not configure logs that capture:

- administrator Bearer credentials;
- publisher private keys;
- environment-file contents;
- unredacted sensitive request bodies;
- secret-bearing headers introduced by local infrastructure.

Publisher public keys and signatures are protocol inputs, but logging full publication bodies is still an operational data-retention decision and must follow the applicable policy.

Caddy logs and Lingonberry process logs are separate evidence sources. Neither is a substitute for storage verification or ingestion-result inspection.

## 8. Compression and body integrity

Response compression may be enabled only after compatibility testing with the intended clients. Caddy must forward the original publication request body bytes required by the HTTP publish signature contract.

Operators must not add transformations that parse and reserialize signed JSON before it reaches the relay. Header normalization performed by an HTTP proxy must not remove required content framing or authentication material for the selected route.

## 9. Administrator listener

Administrator quarantine routes must remain separated from the public listener. The recommended default is:

- public relay: proxied by the public Caddy virtual host;
- administrator listener: bound to loopback, a management network, or another access-controlled interface;
- no public wildcard route forwarding administrator paths;
- independent authentication and authorization verification before use.

A public route returning `404` for an administrator path is expected behavior. Do not rewrite that response into a public administrator endpoint.

## 10. Configuration validation and reload

Validate the installed configuration before reload:

```bash
sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile
sudo systemctl reload caddy
sudo systemctl is-active caddy
sudo journalctl -u caddy --since today
```

Use the service commands supported by the installed Caddy package. A direct `caddy reload` command may be appropriate in some environments, but it is not a Lingonberry-maintained deployment contract.

After any Caddy, DNS, certificate, firewall, relay-listener, or route change, repeat the publication verification steps.

## 11. Verification procedure

From a controlled external client:

1. confirm the public hostname resolves to the intended endpoint;
2. confirm HTTPS certificate and hostname validation succeed;
3. request `GET /v1/ready` through Caddy;
4. request `GET /v1/capabilities` through Caddy;
5. publish the checked-in signed fixture through `POST /v1/objects` and retain the full HTTP status and body;
6. repeat the same publication and confirm explicit duplicate semantics;
7. retrieve the returned canonical ID through Caddy;
8. exercise one rejected fixture and retain the classified response;
9. confirm a known administrator path is not exposed by the public virtual host;
10. confirm the relay's internal port is unreachable from the external client;
11. run the storage `ready`, `doctor`, or `verify` checks required by the operator runbook;
12. record the tested repository commit, Caddy configuration digest, listener addresses, DNS result, certificate identity, firewall state, and evidence classification.

A successful proxy test proves only the tested publication path. It does not prove backup validity, migration safety, crash recovery, disk-pressure behavior, formal soak completion, or privileged reference-host qualification.

## 12. Failure interpretation

| Observation | Likely boundary |
|---|---|
| TLS or hostname failure | DNS, certificate, or Caddy publication layer |
| `502` or `503` from Caddy | relay upstream unavailable, wrong bind address, or local policy failure |
| relay `404` | unknown public route or intentionally hidden administrator route |
| relay `405` | route exists but method is unsupported |
| publish `4xx` with ingestion result | request, validation, signature, identity, or acceptance outcome |
| publish `409` | canonical storage conflict |
| publish `5xx` | relay, storage, or quarantine operational failure |
| `/v1/ready` succeeds but storage verification fails | public listener is reachable; node is not operationally qualified |

Do not convert relay failures into generic `200` responses at Caddy.

## 13. Change control

Review and retain evidence for changes to:

- public hostname or DNS;
- TLS or certificate policy;
- relay upstream address;
- path matching or rewriting;
- request-size, timeout, or rate controls;
- trusted-proxy configuration;
- access logging or redaction;
- firewall or security-group rules;
- public versus administrator listener exposure.

Host-local Caddy changes are outside repository CI unless they are reproduced in checked-in qualification evidence. A repository documentation walkthrough does not validate an unreviewed production Caddyfile.

## 14. Release boundary

The designated v1.0.0 pre-version candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Documentation or deployment guidance committed after that candidate does not redefine it. Formal 72-hour soak, privileged reference-host qualification, version update, release PR, tag, and GitHub Release remain pending.

## Related documents

- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [Relay Quickstart](./RELAY_QUICKSTART.md)
- [Systemd Service Contract](./SYSTEMD_UNIT_TEMPLATES.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Observability](./OBSERVABILITY.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
