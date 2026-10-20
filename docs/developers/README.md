# Developer Documentation / 開発者向け文書

[English](#english) | [日本語](#japanese)

> English is the normative version of this document. The Japanese section is a synchronized translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は同期翻訳です。内容に差異がある場合は英語版を優先します。

**Status: v1.0 pre-release developer entry point** | **Last updated: 2026-10-20**

## English

**Execution hold (2026-10-20):** Reference-host preflight, disk-pressure rehearsal, and the formal 72-hour soak are deferred at the maintainer’s request. Resume only after an explicit maintainer request; v1.0.0 remains unpublished.

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

**実機試験の保留（2026-10-20）:** 基準ホストでの事前検証、ディスク圧迫試験、正式72時間連続稼働試験は、管理者の指示により保留しています。明示的な再開指示を受けてから再開します。v1.0.0は未公開です。

### はじめに

このディレクトリは、アプリケーション、サービス、コネクター、またはプロトコルアダプターをLingonberryと統合する開発者向けです。

#### データの公開機能を実装する開発者

アプリケーションまたはサービスから知識オブジェクト（Knowledge Object）を作成して送信する場合は、[公開クライアントの入門ガイド](./PUBLISHER_QUICKSTART.md)を使用してください。

この文書では次を扱います。

- プロトコルが定める知識オブジェクトの範囲
- 知識オブジェクトを包むHTTP公開要求のデータ構造
- 正規化された署名対象データの構築
- リポジトリ付属のJavaScript生成ツール（参照実装）
- Ed25519署名の生成と強制検証
- HTTP送信と結果処理
- 再試行および仕様適合性の案内

#### リポジトリ統合確認

リポジトリに含まれるサンプルデータをローカルリレーで検証する場合は、[リポジトリ付属のサンプルを使った公開手順](./REPOSITORY_PUBLISH_WALKTHROUGH.md)を使用してください。

こちらはリポジトリ付属の構成を使った連携確認の手順書です。新しい外部公開クライアントを実装するための主要ガイドではありません。

### 文書の配置と役割

`docs/developers/`にはクライアント実装、アプリケーション統合、参照実装の生成ツール、リポジトリ内の構成を使った連携確認の手順を配置します。`docs/operations/`にはリレー、ストレージ、配置、サービスの起動・停止・再起動、復旧、運用担当者向けの手順を配置します。文書の配置は主要な読者と責任境界によって決定します。

#### リレーおよびストレージ開発者

- [リレーの入門ガイド](../operations/RELAY_QUICKSTART.md)
- [ストレージノードの入門ガイド](../operations/STORAGE_NODE_QUICKSTART.md)
- [HTTP carrierの仕様](../operations/HTTP_CARRIER_CONTRACT.md)
- [プロトコル固有の通信形式](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)

### 正本となる仕様

- [プロトコル仕様](../protocols/PROTOCOL_CONTRACT.md)
- [正規化の規則](../protocols/CANONICALIZATION.md)
- [HTTP公開要求の署名仕様](../protocols/HTTP_PUBLISH_SIGNATURE.md)
- [識別情報と来歴情報](../protocols/IDENTITY_AND_PROVENANCE.md)
- [受入方針](../operations/ACCEPTANCE_POLICY.md)

### リリースの状況と適用範囲

最新の公開版は`v0.9.0`です。バージョン更新前の検証対象として固定したv1.0公開候補は次のコミットです。

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

公開候補の検証と文書の手順に沿った動作確認は合格済みです。次に必要なのは基準ホスト上での事前検証です。正式な72時間連続稼働試験、バージョン更新の準備、タグ、GitHub Releaseは未完了です。

これらの開発者向け文書は公開候補を再定義せず、リリースの公開を承認するものでもありません。
