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

**carrier**は、Lingonberryのプロトコルオブジェクトを、データの作成者、ストレージノード、リレー、アーカイブ、利用者の間で運ぶための、具体的な伝送・フレーミング（データの区切り方）の仕組みです。

プロトコルは、データの意味、正規表現、識別情報、検証、互換性の規則を定めます。carrierは、それらのデータを特定の媒体上でどのようにまとめ、宛先を指定し、伝送し、再試行し、保存し、復旧するかを定めます。

### 責務

carrierは次の機能を提供できます。

- プロトコル固有の伝送オブジェクトの区切りを定める
- 接続先のアドレスと、carrier内で使う識別子を提供する
- HTTP、ファイル、アーカイブ、ストリームなど、明示的に対応した媒体でバイト列を伝送する
- 文書に定めた再試行、順序制御、受信確認、対応機能の交渉を行う
- 監査や再実行に必要な、伝送時の生データや記録を保持する
- carrier固有の運用上の制限と、想定される障害を示す

### 責務の範囲外

carrierは次のことをしてはいけません。

- Knowledge Objectの意味を再定義する
- 正規化されたバイト列やcanonical identityを暗黙に書き換える
- 伝送の成功だけで、内容がプロトコル上受け入れられたと判断する
- carrier内の識別子をcanonical identityとして扱う
- プロトコルの検証、認可、ストレージの競合規則を迂回する
- バージョンを明記したアダプター仕様なしに、外部プロトコルとの相互運用性を主張する

### 識別子の境界

- **Canonical identifier（正規識別子）:** プロトコルの仕様に従って導出または検証される、プロトコル上の識別子
- **Carrier identity（carrier内の識別情報）:** HTTP要求の参照、アーカイブのオブジェクトキー、レコードURI、リレーのイベントID、ストリーム内の位置など、carrier内で使う参照
- **Storage-internal key（ストレージ内部キー）:** ストレージの実装が内部で使用するキー

これらは明示的な記録によって関連付けられますが、相互に置き換えることはできません。

### 伝送形式と正規表現

プロトコル固有の伝送形式と正規表現は、Lingonberryの仕様における異なる表現です。carrierは伝送形式を運び、または保存します。正規化と検証によって、正規の意味を持つレコードが決まります。

外部の形式を、そのままLingonberryの伝送形式として扱うことはできません。変換には、来歴を保持し、変換で失われる情報を記録する、バージョン付きの明示的なアダプターが必要です。

### v1.0の対応範囲

Lingonberry v1.0の検証対象は、文書に定めた単一ノードでのHTTPおよびファイル・アーカイブの動作です。複数carrier間の連合、全体での順序保証、分散合意、文書化されていない外部の伝送方式との互換性は保証しません。

## Related contracts

- [Concept Model](./CONCEPT_MODEL.md)
- [Glossary](./GLOSSARY.md)
- [Protocol-native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](../operations/CARRIER_CAPABILITY_NEGOTIATION.md)
