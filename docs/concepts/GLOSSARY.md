# Glossary / 用語集

[English](#english) | [日本語](#japanese)

> English is the normative version of this document. The Japanese section is a synchronized translation.
>
> 英語版がこの文書の正本です。日本語部分は同期された翻訳です。

**Status: reviewed scoped glossary** | **Scope: core concepts used by v1.0 contracts**

## English

### Knowledge Object

The protocol-level semantic unit accepted, stored, retrieved, and evaluated by Lingonberry. Its required structure and validation behavior are defined by the checked-in schema and protocol contracts. A Knowledge Object is not made authoritative merely by arriving through a carrier.

### canonical representation

The deterministic representation produced by the documented canonicalization rules. Canonical bytes are compatibility-relevant and are used by identity, digest, signature, duplicate, and conflict behavior where specified.

### canonical identifier

The protocol-level object identifier validated under the identifier and canonicalization contracts. It is distinct from carrier-local and storage-internal identifiers.

### protocol-native wire format

The Lingonberry-defined wire representation transported by supported carriers. It is not an arbitrary external protocol format. External conversion requires a versioned adapter.

### carrier

A concrete transport and framing mechanism, such as the documented HTTP or file/archive carrier. A carrier moves protocol material but does not define its semantic meaning. See [`CARRIER.md`](./CARRIER.md).

### carrier identity

A carrier-local reference used for addressing, retrieval, transport deduplication, or audit, such as an archive key, request reference, record URI, event identifier, or stream offset. It is not canonical identity.

### storage-internal key

An implementation key used by a storage backend. It is not a public protocol identifier unless an explicit contract says otherwise.

### identity key

The deterministic key used by the current identity rule to bind or resolve object identity. Its exact basis and version are defined by the identity and protocol contracts, not by carrier endpoint location.

### identity claim

Versioned evidence asserting an identity binding. A claim proves only what its verification contract establishes; it does not independently prove that the object's content is true.

### provenance

Evidence describing origin, custody, issuer, transformation, or transport history. Provenance supports auditability but is not synonymous with semantic truth or authorization.

### raw reference

A reference to retained raw request or carrier evidence used for audit, replay, or reconstruction where the relevant contract requires it. A raw reference does not replace canonical storage.

### lineage

Explicit relationships describing derivation, revision, supersession, translation, or another documented history relation between objects.

### relation

A semantic link between Knowledge Objects under the protocol schema. Relations do not alter the immutable source object merely because a derived view interprets them.

### transition

A separately validated evidence object that may affect an effective view according to transition authority, ordering, supersession, and reevaluation contracts. A transition does not rewrite the original Knowledge Object.

### effective view

A derived, rebuildable interpretation of canonical objects and accepted transition evidence. It is not the canonical source of truth and must fail closed when the governing evidence is ambiguous or invalid.

### derived index

A rebuildable search or lookup structure generated from authoritative durable records. The index is not the semantic source and must not silently override canonical storage.

### quarantine

A controlled state for material that is not eligible for canonical acceptance but must be retained for review, recovery, replacement, dismissal, or permanent rejection under documented rules.

### application profile

A domain-specific set of constraints or vocabulary layered above the protocol core. A profile may narrow use but must not silently redefine core canonicalization, identity, validation, or compatibility semantics.

### capability

A versioned declaration of supported behavior. Capability negotiation does not by itself prove authorization, data validity, or successful interoperability.

### protocol

The versioned semantic and validation contract governing Lingonberry objects, identities, wire forms, errors, and compatibility behavior. The protocol is distinct from its carriers and runtime implementations.

### relay

The operator-facing process that exposes documented HTTP, storage, diagnostic, and lifecycle behavior. v1.0 qualifies a single-node relay; the term does not imply federation or consensus.

<a id="japanese"></a>

## 日本語

### Knowledge Object

Lingonberryが受け入れ、保存、取得、評価する、プロトコル上の意味の単位です。必須の構造と検証時の動作は、リポジトリ内のスキーマとプロトコル仕様で定めます。carrierを通じて届いただけでは、正本として認められません。

### canonical representation

文書に定めた正規化規則によって、同じ入力から同じ結果が得られる表現です。正規化されたバイト列は互換性に関わり、仕様に従って識別、ダイジェスト、署名、重複・競合の判定に使われます。

### canonical identifier

識別子と正規化の仕様に従って検証される、プロトコル上のオブジェクト識別子です。carrier内の識別子やストレージ内部の識別子とは区別します。

### protocol-native wire format

対応するcarrierが運ぶ、Lingonberry固有の伝送形式です。任意の外部プロトコルの形式をそのまま使うものではありません。外部形式との変換には、バージョン付きアダプターが必要です。

### carrier

文書に定めたHTTPやファイル・アーカイブなどによる、具体的な伝送・フレーミングの仕組みです。プロトコルのデータを運びますが、その意味は定義しません。詳細は[`CARRIER.md`](./CARRIER.md)を参照してください。

### carrier identity

宛先指定、取得、伝送上の重複排除、監査に使う、carrier内の参照です。アーカイブキー、要求の参照、レコードURI、イベント識別子、ストリーム内の位置などが該当します。canonical identityとは区別します。

### storage-internal key

ストレージの実装が内部で使用するキーです。仕様に明記されていない限り、公開プロトコルの識別子ではありません。

### identity key

現在の識別規則に従い、オブジェクトの同一性を対応付けたり判定したりするための、決定的なキーです。何を基に生成するかと、そのバージョンは、識別情報とプロトコルの仕様で定めます。carrierの接続先の場所によって決まるものではありません。

### identity claim

識別情報の対応関係を主張する、バージョン付きの証明データです。証明できるのは検証仕様で定めた範囲に限られ、オブジェクトの内容が真実であることを独立して証明するものではありません。

### provenance

出所、管理の経緯、発行者、変換、伝送の履歴を示す記録です。監査を可能にしますが、内容の真実性や認可と同じ意味ではありません。

### raw reference

関連する仕様で必要とされる場合に、監査、再実行、再構築に使う、保存済みの生の要求やcarrierの記録への参照です。正本となるストレージの代わりにはなりません。

### lineage

オブジェクト間の派生、改訂、置換、翻訳など、文書に定めた履歴上の関係を明示するものです。

### relation

プロトコルのスキーマに従ってKnowledge Object同士を結ぶ、意味上の関係です。派生ビューがこの関係を解釈しても、変更不可の元オブジェクトを書き換えることはありません。

### transition

遷移の権限、順序、置換、再評価の仕様に従って、有効ビューに影響を与え得る、独立した検証対象の証明オブジェクトです。元のKnowledge Objectは書き換えません。

### effective view

正規オブジェクトと受理済みの遷移の証明データから導く、再構築可能な解釈です。意味上の正本ではありません。判断の根拠となる証明データが曖昧または無効な場合は、安全側に処理を拒否します。

### derived index

正本となる永続レコードから生成する、再構築可能な検索・参照用の構造です。意味上の正本ではなく、正本となるストレージを暗黙に上書きしてはいけません。

### quarantine

正規データとして受け入れられないものを、文書に定めた確認、復旧、置換、除外、恒久的な拒否のために保持する、管理された状態です。

### application profile

プロトコルの中核仕様の上に重ねる、分野固有の制約や語彙のまとまりです。利用範囲を狭めることはできますが、中核となる正規化、識別、検証、互換性の意味を暗黙に再定義してはいけません。

### capability

対応する動作を示す、バージョン付きの宣言です。対応機能を交渉しただけでは、認可、データの妥当性、相互運用の成功は証明できません。

### protocol

Lingonberryのオブジェクト、識別情報、伝送形式、エラー、互換性の動作を定める、バージョン付きの意味と検証の仕様です。carrierや実行時の実装とは区別します。

### relay

文書に定めたHTTP、ストレージ、診断、起動・停止などの動作を提供する、運用担当者向けのプロセスです。v1.0の検証対象は単一ノードのリレーであり、連合や合意の機能を意味するものではありません。

## Governing references / 関連正本

- [Protocol Contract](../protocols/PROTOCOL_CONTRACT.md)
- [Protocol Identifiers](../protocols/PROTOCOL_IDENTIFIERS.md)
- [Canonicalization](../protocols/CANONICALIZATION.md)
- [Identity and Provenance](../protocols/IDENTITY_AND_PROVENANCE.md)
- [Transition Object](../protocols/TRANSITION_OBJECT.md)
- [Carrier](./CARRIER.md)
