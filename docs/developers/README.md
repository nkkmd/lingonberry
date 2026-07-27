# Developer Documentation

**Status: v1.0 pre-release developer entry point**

This directory is for developers who integrate applications, services, connectors, or protocol adapters with Lingonberry. English is normative.

## Start here

### Publisher developers

Use [Publisher Quickstart](./PUBLISHER_QUICKSTART.md) when an application or service needs to create and submit Knowledge Objects.

It covers:

- the protocol-native Knowledge Object boundary;
- the HTTP publish-request envelope;
- canonical signing-target construction;
- the checked-in JavaScript reference producer;
- HTTP submission and result handling;
- retry and conformance guidance;
- the current signature-enforcement limitation.

### Repository integration checks

Use [Knowledge Object Publish Quickstart](../operations/KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md) when validating the checked-in fixture against a locally started relay.

That document is a repository and integration walkthrough. It is not the primary guide for implementing a new external publisher.

### Relay and storage developers

- [Relay Quickstart](../operations/RELAY_QUICKSTART.md)
- [Storage Node Quickstart](../operations/STORAGE_NODE_QUICKSTART.md)
- [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)

## Normative contracts

- [Protocol Contract](../protocols/PROTOCOL_CONTRACT.md)
- [Canonicalization](../protocols/CANONICALIZATION.md)
- [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)
- [Identity and Provenance](../protocols/IDENTITY_AND_PROVENANCE.md)
- [Acceptance Policy](../operations/ACCEPTANCE_POLICY.md)

## Release boundary

The latest published release is `v0.9.0`. The fixed pre-version v1.0 candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

These developer documents do not redefine that candidate or complete the formal 72-hour soak, privileged reference-host qualification, version preparation, tag, or GitHub Release.
