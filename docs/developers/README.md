# Developer Documentation / 開発者向け文書

[English](#english) | [日本語](#japanese)

> English is the normative version of this document. The Japanese section is a synchronized translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は同期翻訳です。内容に差異がある場合は英語版を優先します。

**Status: v1.0 pre-release developer entry point** | **Last updated: 2026-07-27**

## English

### Start here

This directory is for developers who integrate applications, services, connectors, or protocol adapters with Lingonberry.

#### Publisher developers

Use [Publisher Quickstart](./PUBLISHER_QUICKSTART.md) when an application or service needs to create and submit Knowledge Objects.

It covers:

- the protocol-native Knowledge Object boundary;
- the HTTP publish-request envelope;
- canonical signing-target construction;
- the checked-in JavaScript reference producer;
- Ed25519 signature generation and enforced verification;
- HTTP submission and result handling;
- retry and conformance guidance.

#### Repository integration checks

Use [Repository Publish Walkthrough](./REPOSITORY_PUBLISH_WALKTHROUGH.md) when validating the checked-in fixture against a locally started relay.

That document is a repository and integration walkthrough. It is not the primary guide for implementing a new external publisher.

### Directory boundary

`docs/developers/` contains client implementation, application integration, reference producer, and repository integration guidance. `docs/operations/` contains relay, storage, deployment, service lifecycle, recovery, and operator procedures. Documents are placed by their primary reader and responsibility boundary.

#### Relay and storage developers

- [Relay Quickstart](../operations/RELAY_QUICKSTART.md)
- [Storage Node Quickstart](../operations/STORAGE_NODE_QUICKSTART.md)
- [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)

### Normative contracts

- [Protocol Contract](../protocols/PROTOCOL_CONTRACT.md)
- [Canonicalization](../protocols/CANONICALIZATION.md)
- [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)
- [Identity and Provenance](../protocols/IDENTITY_AND_PROVENANCE.md)
- [Acceptance Policy](../operations/ACCEPTANCE_POLICY.md)

### Release boundary

The latest published release is `v0.9.0`. The fixed pre-version v1.0 candidate is:

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

Candidate qualification and documentation walkthrough have passed. Privileged reference-host preflight is the next gate. Formal 72-hour soak, version preparation, tag, and GitHub Release remain pending.

These developer documents do not redefine the candidate or authorize release publication.

---

<a id="japanese"></a>

## 日本語

### はじめに

このディレクトリは、アプリケーション、サービス、connector、またはprotocol adapterをLingonberryと統合する開発者向けです。

#### Publisher開発者

アプリケーションまたはサービスからKnowledge Objectを作成して送信する場合は、[Publisher Quickstart](./PUBLISHER_QUICKSTART.md)を使用してください。

この文書では次を扱います。

- protocol-nativeなKnowledge Object境界
- HTTP publish-request envelope
- canonical signing-targetの構築
- リポジトリに含まれるJavaScript reference producer
- Ed25519 signatureの生成と強制検証
- HTTP送信と結果処理
- retryおよびconformanceのガイダンス

#### リポジトリ統合確認

リポジトリに含まれるfixtureをローカルrelayで検証する場合は、[Repository Publish Walkthrough](./REPOSITORY_PUBLISH_WALKTHROUGH.md)を使用してください。

この文書はrepository integrationのwalkthroughです。新しい外部publisherを実装するための主要ガイドではありません。

### ディレクトリの境界

`docs/developers/`にはclient実装、application統合、reference producer、repository integrationのガイダンスを配置します。`docs/operations/`にはrelay、storage、deployment、service lifecycle、recovery、operator手順を配置します。文書の配置は主要な読者と責任境界によって決定します。

#### Relayおよびstorage開発者

- [Relay Quickstart](../operations/RELAY_QUICKSTART.md)
- [Storage Node Quickstart](../operations/STORAGE_NODE_QUICKSTART.md)
- [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)

### 正本となるcontract

- [Protocol Contract](../protocols/PROTOCOL_CONTRACT.md)
- [Canonicalization](../protocols/CANONICALIZATION.md)
- [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)
- [Identity and Provenance](../protocols/IDENTITY_AND_PROVENANCE.md)
- [Acceptance Policy](../operations/ACCEPTANCE_POLICY.md)

### Release境界

最新の公開releaseは`v0.9.0`です。固定されたpre-version v1.0 candidateは次のcommitです。

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

candidate qualificationとdocumentation walkthroughはPASS済みです。次のgateはprivileged reference-host preflightです。正式72時間soak、version preparation、tag、GitHub Releaseは未完了です。

これらの開発者向け文書はcandidateを再定義せず、release publicationを承認するものでもありません。
