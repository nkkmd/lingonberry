# Glossary / 用語集

[English](#english) | [日本語](#日本語)

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

## 日本語

### Knowledge Object

Lingonberryがaccept、store、retrieve、evaluateするprotocol-levelのsemantic unitです。必須構造とvalidation behaviorは、repository内のschemaおよびprotocol contractで定義されます。carrierを通じて到着しただけではauthoritativeになりません。

### canonical representation

文書化されたcanonicalization ruleによって生成される決定的な表現です。canonical byteはcompatibility-relevantであり、規定されたidentity、digest、signature、duplicate、conflict behaviorに使用されます。

### canonical identifier

identifierおよびcanonicalization contractに従って検証されるprotocol-level object identifierです。carrier-local identifierやstorage-internal identifierとは異なります。

### protocol-native wire format

対応carrierが運ぶLingonberry定義のwire representationです。任意の外部protocol formatではありません。外部変換にはversioned adapterが必要です。

### carrier

文書化されたHTTP carrierやfile/archive carrierなどの具体的なtransport／framing機構です。protocol materialを運びますが、semanticを定義しません。詳細は[`CARRIER.md`](./CARRIER.md)を参照してください。

### carrier identity

archive key、request reference、record URI、event identifier、stream offsetなど、addressing、retrieval、transport deduplication、auditに使用するcarrier-local referenceです。canonical identityではありません。

### storage-internal key

storage backendが使用するimplementation keyです。明示的なcontractがない限りpublic protocol identifierではありません。

### identity key

現在のidentity ruleでobject identityのbindingまたはresolutionに使用する決定的なkeyです。正確なbasisとversionはidentity／protocol contractで定義され、carrier endpoint locationには依存しません。

### identity claim

identity bindingを主張するversioned evidenceです。claimが証明するのはverification contractで確立される範囲だけで、object内容の真実性を独立して証明するものではありません。

### provenance

origin、custody、issuer、transformation、transport historyを記述するevidenceです。auditabilityを支えますが、semantic truthやauthorizationと同義ではありません。

### raw reference

関連contractが要求する場合に、audit、replay、reconstructionへ使用するraw requestまたはcarrier evidenceへの参照です。canonical storageの代替ではありません。

### lineage

object間のderivation、revision、supersession、translation、その他文書化された履歴関係を表す明示的relationです。

### relation

protocol schemaに従うKnowledge Object間のsemantic linkです。derived viewが解釈してもimmutableなsource objectを書き換えません。

### transition

transition authority、ordering、supersession、reevaluation contractに従ってeffective viewへ影響し得る、別個にvalidatedされたevidence objectです。元のKnowledge Objectを書き換えません。

### effective view

canonical objectとaccepted transition evidenceから生成されるderivedかつrebuildableな解釈です。canonical source of truthではなく、governing evidenceがambiguousまたはinvalidな場合はfail closedします。

### derived index

authoritative durable recordから生成されるrebuildableなsearch／lookup structureです。semantic sourceではなく、canonical storageを暗黙に上書きしてはいけません。

### quarantine

canonical acceptanceの対象ではないmaterialを、文書化されたreview、recovery、replacement、dismissal、permanent rejectionのために保持するcontrolled stateです。

### application profile

domain-specificなconstraintやvocabularyをprotocol coreの上に重ねる仕組みです。利用範囲を狭めることはできますが、core canonicalization、identity、validation、compatibility semanticsを暗黙に再定義してはいけません。

### capability

対応behaviorのversioned declarationです。capability negotiationだけではauthorization、data validity、interoperability成功を証明しません。

### protocol

Lingonberry object、identity、wire form、error、compatibility behaviorを統治するversioned semantic／validation contractです。carrierおよびruntime implementationとは異なります。

### relay

文書化されたHTTP、storage、diagnostic、lifecycle behaviorを公開するoperator-facing processです。v1.0が資格確認するのはsingle-node relayであり、federationやconsensusを意味しません。

## Governing references / 関連正本

- [Protocol Contract](../protocols/PROTOCOL_CONTRACT.md)
- [Protocol Identifiers](../protocols/PROTOCOL_IDENTIFIERS.md)
- [Canonicalization](../protocols/CANONICALIZATION.md)
- [Identity and Provenance](../protocols/IDENTITY_AND_PROVENANCE.md)
- [Transition Object](../protocols/TRANSITION_OBJECT.md)
- [Carrier](./CARRIER.md)
