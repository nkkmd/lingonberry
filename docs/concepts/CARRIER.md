# Carrier

[English](#english) | [日本語](#japanese)

> English is the normative version of this document. The Japanese section is a synchronized translation.
>
> 英語版がこの文書の正本です。日本語部分は同期された翻訳です。

**Status: reviewed concept contract** | **Scope: v1.0 single-node architecture**

## English

### Definition

A **carrier** is a concrete transport and framing mechanism that moves a Lingonberry protocol object between producers, storage nodes, relays, archives, or consumers.

A carrier is not the protocol itself. The protocol defines semantic meaning, canonical representation, identity, validation, and compatibility rules. A carrier defines how protocol material is packaged, addressed, transmitted, retried, stored, or recovered in a particular medium.

### Responsibilities

A carrier may:

- frame protocol-native wire objects;
- provide endpoint addressing and carrier-local identifiers;
- transport bytes over HTTP, files, archives, streams, or another explicitly supported medium;
- provide retry, ordering, acknowledgement, or capability negotiation when documented;
- retain raw transport evidence required for audit or replay;
- expose carrier-specific operational limits and failure modes.

### Non-responsibilities

A carrier must not:

- redefine the semantic meaning of a Knowledge Object;
- silently rewrite canonical bytes or canonical identity;
- treat transport success as semantic acceptance;
- promote a carrier-local identifier to canonical identity;
- bypass protocol validation, authorization, or storage conflict rules;
- imply interoperability with an external protocol without a versioned adapter contract.

### Identity boundaries

- **Canonical identifier:** protocol-level identity derived or validated under the protocol contract.
- **Carrier identity:** a carrier-local reference such as an HTTP request reference, archive object key, record URI, relay event ID, or stream offset.
- **Storage-internal key:** an implementation key used by a storage backend.

These identifiers may be linked by explicit evidence, but they are not interchangeable.

### Wire and canonical forms

The protocol-native wire format and canonical representation are different representations within the Lingonberry contract. A carrier transports or stores the wire representation; canonicalization and validation determine the canonical semantic record.

External formats are not automatically Lingonberry wire formats. Conversion requires an explicit, versioned adapter that preserves provenance and records any lossy mapping.

### v1.0 boundary

Lingonberry v1.0 qualifies documented single-node HTTP and file/archive behavior. It does not guarantee multi-carrier federation, global ordering, distributed consensus, or compatibility with an undocumented external transport.

<a id="japanese"></a>

## 日本語

### 定義

**carrier**は、Lingonberryのprotocol objectをproducer、storage node、relay、archive、consumerの間で運ぶ具体的なtransport／framing機構です。

carrierはprotocolそのものではありません。protocolはsemantic、canonical representation、identity、validation、compatibility ruleを定義します。carrierは、特定の媒体上でprotocol materialをどのようにpackaging、addressing、transmission、retry、storage、recoveryするかを定義します。

### 責務

carrierは次の機能を提供できます。

- protocol-native wire objectのframing
- endpoint addressingとcarrier-local identifier
- HTTP、file、archive、stream、その他明示的に対応した媒体でのbyte transport
- 文書化されたretry、ordering、acknowledgement、capability negotiation
- auditやreplayに必要なraw transport evidenceの保持
- carrier固有の運用上限とfailure modeの公開

### 責務ではないもの

carrierは次のことをしてはいけません。

- Knowledge Objectのsemanticを再定義する
- canonical byteやcanonical identityを暗黙に書き換える
- transport成功をsemantic acceptanceとみなす
- carrier-local identifierをcanonical identityへ昇格させる
- protocol validation、authorization、storage conflict ruleを回避する
- versioned adapter contractなしに外部protocolとの相互運用性を主張する

### identity境界

- **Canonical identifier:** protocol contractに従って導出または検証されるprotocol-level identity
- **Carrier identity:** HTTP request reference、archive object key、record URI、relay event ID、stream offsetなどのcarrier-local reference
- **Storage-internal key:** storage backendが使用するimplementation key

これらは明示的なevidenceによって関連付けられますが、相互に置き換えることはできません。

### wireとcanonical form

protocol-native wire formatとcanonical representationは、Lingonberry contract内の異なる表現です。carrierはwire representationをtransportまたは保存し、canonicalizationとvalidationがcanonical semantic recordを決定します。

外部formatは自動的にLingonberry wire formatにはなりません。変換には、provenanceを保持し、lossy mappingを記録する明示的でversionedなadapterが必要です。

### v1.0境界

Lingonberry v1.0が資格確認するのは、文書化されたsingle-node HTTPおよびfile/archive behaviorです。multi-carrier federation、global ordering、distributed consensus、文書化されていない外部transportとの互換性は保証しません。

## Related contracts

- [Concept Model](./CONCEPT_MODEL.md)
- [Glossary](./GLOSSARY.md)
- [Protocol-native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](../operations/CARRIER_CAPABILITY_NEGOTIATION.md)
