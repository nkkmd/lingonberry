# Lingonberry Protocol Evolution Proposal

**Status: draft**  
**Last updated: 2026-07-11**

## 1. 目的

この文書は、Lingonberry を長期運用可能で相互運用性の高い分散知識コモンズ・プロトコルへ発展させるための改善方針を整理するものです。

Lingonberry はすでに、Knowledge Object、append-only、replayability、provenance、lineage、carrier-neutrality、application profile という強い設計軸を持っています。今後は機能を増やすこと以上に、次を厳密に固定する必要があります。

1. protocol core の不変条件
2. application profile、policy、実装へ委ねる範囲
3. 異なる実装間の決定的な相互運用性
4. 敵対的な公開ネットワークに対する安全性
5. relay 間の効率的かつ検証可能な同期

---

## 2. 最優先の改善項目

### 2.1 Identity の暗号学的強化

semantic identity fingerprint には、暗号学的ハッシュを使用します。

```text
lb:key:<rule-version>:<hash-algorithm>:<digest>
```

候補:

- SHA-256
- SHA-512/256
- BLAKE3
- multihash / multibase

同時に、hash input となる canonical byte representation を固定します。

- UTF-8
- Unicode 正規化方式
- JSON canonicalization
- field ordering
- 数値表現
- timestamp 表現
- language tag の正規化
- 欠落 field と空値の区別
- relation、lineage、identityClaims の順序規則

仕様の正本は「意味的に同じ JSON」ではなく、実装間で一致する **canonical bytes** とします。

### 2.2 Identity の三層分離

次の概念を明確に分けます。

| 種類 | 役割 |
|---|---|
| Object ID | 個々に publish された object を識別する |
| Semantic Identity Key | 内容上の同一性・重複候補を照合する |
| Lineage Identity | revision をまたいだ同一系列を辿る |

例:

```json
{
  "id": "lb:object:...",
  "identityKey": "lb:key:...",
  "lineage": {
    "root": "lb:object:...",
    "previous": "lb:object:..."
  }
}
```

これにより、同じ内容の複製、別 carrier 上の同一 object、内容が更新された revision、同じ知識系列に属する異なる表現を区別できます。

### 2.3 署名対象の固定

公開鍵署名を相互運用可能にするには、署名対象となるバイト列を protocol level で固定します。

```text
canonical object bytes
  -> cryptographic digest
  -> domain-separated signature payload
  -> signature
```

例:

```text
LINGONBERRY_OBJECT_V1 || digest
```

定義対象:

- 対応署名アルゴリズム
- 鍵 ID の形式
- 鍵ローテーションと失効
- 複数署名と組織署名
- 代理署名と AI agent の署名
- author、publisher、transformer、attestor、relay の役割分離

署名は「送信者」と「知識の著者」を同一視しない設計にします。

### 2.4 Conformance Test Suite

文章仕様だけでなく、公式の適合試験を protocol の主要成果物として整備します。

```text
conformance/
├── canonicalization/
├── validation/
├── identity/
├── signatures/
├── revisions/
├── tombstones/
├── archive-replay/
├── carrier-http/
├── synchronization/
└── malformed-input/
```

各 test vector は、少なくとも input、expected canonical bytes、expected identity key、expected result を持ちます。

主要試験項目:

- field order、Unicode、timestamp の表記差
- optional field と default の扱い
- 不正署名と identity claim 不一致
- rawRef 欠落
- 循環 lineage
- 未知 extension field
- archive ordering の違い
- 異なる relay から得た同一 object
- 旧実装が将来 version を受信した場合

Rust と TypeScript など、最低 2 つの独立実装が同じ test vector を通過することを仕様安定化の条件とします。

### 2.5 Relay 間同期

複数ノードが効率的に同じ知識集合へ収束できる同期仕様を定義します。

候補:

- cursor-based synchronization
- time-range synchronization
- hash inventory
- Merkle tree / Merkle DAG
- Bloom filter
- set reconciliation
- content-addressed archive chunk
- partial replication

想定 node profile:

- full archive node
- domain-specific node
- recent-only relay
- metadata-only indexer
- attachment storage node
- local community node

capability manifest 例:

```json
{
  "retention": "full",
  "acceptedProfiles": ["..."],
  "maxObjectSize": 1048576,
  "supportsAttachments": true,
  "syncMethods": ["cursor", "merkle-v1"],
  "historyFrom": "2026-01-01T00:00:00Z"
}
```

---

## 3. 知識プロトコルとしての意味論強化

### 3.1 事実ではなく検証可能な主張を保存する

Lingonberry core は内容を真実として認定しません。

```text
Actor A asserts X at time T
Evidence B supports X
Actor C disputes X
Review D retracts X
```

relation vocabulary の候補:

- `supports`
- `contradicts`
- `qualifies`
- `retracts`
- `reviews`
- `replicates`
- `fails_to_replicate`

protocol が保証するのは真偽ではなく、主張者、時刻、来歴、関係、完全性です。

### 3.2 Trust を単一スコアにしない

信頼は次の独立した観点として扱います。

- authenticity
- integrity
- provenance completeness
- source reputation
- evidence quality
- review status
- recency
- contextual applicability

protocol core は署名検証、改変検出、provenance、relation、lineage を扱い、評価は application profile、local policy、community trust graph、institution policy、user preference に委ねます。

### 3.3 Revision、retraction、tombstone の分離

| 操作 | 意味 |
|---|---|
| revision | 内容の更新版を追加する |
| supersession | 新版を優先版として示す |
| retraction | 主張を撤回する |
| semantic tombstone | 通常表示から除外すべき状態を示す |
| storage suppression | 特定 node が配信・保持しない |
| legal removal | 法令や権利侵害に基づく運用上の削除 |
| cryptographic erasure | 暗号鍵破棄により読めなくする |

append-only を維持しつつ、実運用の削除要請に対応できるようにします。

---

## 4. 永続参照と Attachment

### 4.1 rawRef の content-addressed 化

rawRef は URL だけに依存させず、content integrity と location を分離します。

```json
{
  "rawRef": {
    "digest": "sha256:...",
    "mediaType": "application/json",
    "size": 1234,
    "locations": [
      "https://relay-a.example/objects/...",
      "lb-archive:..."
    ]
  }
}
```

> URL は所在を示し、digest は内容を示す。

これにより、移設、複製、監査、再取得、改変検出が容易になります。

### 4.2 Attachment の分離

画像、PDF、音声、データセットなどは Knowledge Object 本体へ埋め込まず、content-addressed blob として参照します。

```json
{
  "attachments": [
    {
      "digest": "sha256:...",
      "mediaType": "application/pdf",
      "size": 245678,
      "title": "調査報告書",
      "locations": []
    }
  ]
}
```

将来拡張:

- chunking
- resumable download
- mirror discovery
- license metadata
- encryption metadata
- malware scan metadata
- thumbnail
- retention policy

Knowledge Object 本体は小さく保ち、metadata-only relay を可能にします。

---

## 5. Application Profile と Extension

### 5.1 Profile の自己記述化

application profile 自身を署名付き Knowledge Object として配布できるようにします。

```json
{
  "type": "application-profile",
  "body": {
    "name": "Toitoi Profile",
    "version": "1.0.0",
    "extends": [],
    "schemas": [],
    "vocabularies": [],
    "requiredCapabilities": []
  }
}
```

profile の revision、fork、署名、互換性宣言を Lingonberry 上で管理します。中央 registry は必須にせず、複数の signed catalog を任意に運用できる形とします。

### 5.2 Extension namespace

field 名の衝突を避けるため、extension namespace を導入します。

```json
{
  "extensions": {
    "https://example.org/profiles/research/v1": {
      "confidence": 0.8
    }
  }
}
```

未知 extension を受け取った実装は、原則として object 全体を reject せず、extension を変更・欠落させずに再配信できる必要があります。semantic interpretation は任意とします。

---

## 6. API と Protocol Core の境界

```text
Lingonberry Core
├── Object Model
├── Canonicalization
├── Identity and Signatures
├── Revision and Tombstone
├── Carrier Semantics
└── Replication Semantics

Standard APIs
├── Publish API
├── Retrieval API
├── Query API
├── Sync API
└── Capability API
```

core で標準化しやすい query:

- canonical ID lookup
- exact type filter
- createdAt range
- author key filter
- relation traversal
- lineage traversal
- pagination
- capability discovery

全文検索、ranking、embedding search は実装差が大きいため、結果の完全一致を core requirement にしません。

---

## 7. Security Model

公開 relay は敵対的入力を前提とします。

### 7.1 脅威

- oversized object
- publish flooding
- deep nesting
- excessive relations
- cyclic lineage
- canonicalization bomb
- compression bomb
- expensive signature verification
- duplicate object flooding
- timestamp manipulation
- identity collision
- replay spam
- malicious profile / capability manifest
- malicious archive import
- attachment malware

### 7.2 防御

- maximum object size
- maximum nesting depth
- maximum relation / attachment count
- signature verification budget
- per-key rate limit / quota
- canonicalization timeout
- archive import sandbox
- explicit protocol error codes
- safe default limits

安全制限は各実装の暗黙的な運用知識だけにせず、推奨値と拒否理由を仕様化します。

---

## 8. Governance

仕様変更を管理する proposal 制度を導入します。

```text
LBP-0001 Core Object Model
LBP-0002 Canonicalization
LBP-0003 Identity and Signatures
LBP-0004 HTTP Carrier
LBP-0005 Archive Format
LBP-0006 Relay Synchronization
LBP-0100 Toitoi Application Profile
```

各 proposal に含める項目:

- Status: Draft / Experimental / Stable / Deprecated
- Version
- Authors
- Motivation
- Specification
- Compatibility impact
- Security considerations
- Test vectors
- Migration path

推奨ルール:

- core の Stable 化には複数実装を必要とする
- Stable 仕様では破壊的変更を避ける
- Experimental extension は独立に検証できる
- security update 手続きを別途定義する
- deprecated 仕様には移行期間を設ける

---

## 9. 最初に完成させる垂直ユースケース

Toitoi application profile を使い、次を end-to-end で動かします。

```text
inquiry を publish
  -> observation を関連付ける
  -> evidence を追加する
  -> AI が synthesis を生成する
  -> 人間が annotation / revision を追加する
  -> 別 relay へ複製する
  -> archive から replay する
  -> provenance と lineage を viewer で確認する
```

必要な構成:

- CLI または authoring client
- 2 台以上の relay
- storage node
- indexer
- archive export/import
- provenance viewer
- revision / relation graph viewer

単なる分散保存デモではなく、知識が関連付けられ、改訂され、検証され、成長することを示すデモにします。

---

## 10. 推奨ロードマップ

### Phase A: 仕様基礎

1. canonical byte representation の固定
2. 暗号学的 identity key への移行
3. object / semantic / lineage identity の分離
4. signature payload と verification の固定
5. security threat model の作成

### Phase B: 相互運用性

1. conformance test suite
2. Rust と TypeScript の独立実装
3. archive replay golden tests
4. protocol error code
5. compatibility matrix

### Phase C: 分散同期

1. relay discovery
2. cursor sync
3. hash inventory または Merkle-based sync
4. partial replication
5. signed capability manifest
6. retention policy

### Phase D: 知識表現

1. claim / evidence / contradiction model
2. provenance viewer
3. revision / relation graph
4. AI-generated object provenance
5. Toitoi application profile の安定化

### Phase E: Ecosystem

1. Lingonberry Proposal 制度
2. profile authoring guide
3. SDK
4. public test network
5. interoperability testing event
6. v1.0 compatibility policy

---

## 11. 成功条件

### Protocol correctness

- 同じ input からすべての適合実装が同じ canonical bytes を生成する
- identity key と署名検証結果が一致する
- archive replay で同じ canonical state を再構築できる

### Interoperability

- 独立した複数実装が object を交換できる
- relay 間同期で同じ object set へ収束できる
- 未知 extension を失わず中継できる

### Verifiability

- 誰が、いつ、何を主張したか確認できる
- provenance、rawRef、lineage を辿れる
- 派生 object と原資料の関係を検証できる

### Operational resilience

- node 障害から archive replay で復旧できる
- 敵対的入力を安全に拒否できる
- node policy と protocol semantics が混同されていない

### Domain independence

- Toitoi を自然に実装できる
- Toitoi 固有語彙を core に持ち込まない
- 他分野の profile を同じ core 上で定義できる

---

## 12. 結論

Lingonberry の最も重要な価値は、情報を単に分散保存することではありません。

> 誰が、いつ、どの根拠と来歴で知識を提示し、その知識がどのように改訂・翻訳・反論・統合されたかを、分散環境で検証可能にすること。

この価値を実現するためには、次の 4 点を最優先で固める必要があります。

1. canonical bytes、identity、signature の暗号学的仕様
2. 複数実装で共有する conformance test suite
3. relay 間の差分同期と部分レプリケーション
4. trust、revision、deletion、policy の責務境界

これらが揃えば、Lingonberry は良い構想にとどまらず、長期運用可能な分散知識基盤へ発展できます。
