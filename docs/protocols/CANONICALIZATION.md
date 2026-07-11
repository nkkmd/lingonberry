# Canonicalization

**Status: draft** | **Last updated: 2026-07-11**

## 1. 目的

この文書は、Lingonberry protocol object から決定的な canonical JSON bytes を生成する規則を定義します。

canonicalization は次の基盤です。

- archive replay
- semantic identity key
- publish request signature
- object deduplication
- conformance testing
- 異なる言語で実装された node 間の相互運用

同じ JSON value を入力した適合実装は、同じ UTF-8 byte sequence を生成しなければなりません。

## 2. Rule version

初期規則は次の名前で参照します。

```text
lb.canonical.json.v1
```

この version は、現在の Rust および JavaScript 参照実装の既存挙動を仕様として固定するものです。

## 3. Canonicalization pipeline

```text
JSON input
  -> parse
  -> recursively order object members
  -> serialize without insignificant whitespace
  -> UTF-8 encode
  -> canonical bytes
```

### 3.1 Parse

入力は RFC 8259 JSON として解析します。

不正な JSON、重複 member、実装が安全に表現できない number は、canonicalization より前に reject して構いません。重複 member の扱いを実装依存にしないため、publisher は重複 member を含む object を生成してはいけません。

### 3.2 Object member ordering

すべての JSON object について、member name を Unicode scalar value の辞書順で昇順に並べます。

この処理は root object だけでなく、nested object にも再帰的に適用します。

例:

```json
{"z":1,"a":{"z":2,"a":3}}
```

は次になります。

```json
{"a":{"a":3,"z":2},"z":1}
```

### 3.3 Array ordering

array element の順序は変更しません。

array の各 element が object の場合、その object の member ordering だけを再帰的に適用します。

### 3.4 Strings

string value と member name は UTF-8 として出力します。

次の character は JSON escape を使用します。

- quotation mark
- reverse solidus
- U+0000 から U+001F の control characters

`lb.canonical.json.v1` では Unicode normalization を行いません。NFC と NFD は異なる byte sequence として扱います。この点は将来の rule version で変更できますが、既存 version の意味を変更してはいけません。

### 3.5 Numbers

number は JSON number として出力し、文字列へ変換しません。

`lb.canonical.json.v1` の複数言語実装で安全に一致させるため、protocol schema は identity や signature の対象となる field で、実装間に表現差が生じる極端な数値を避けるべきです。

将来、number normalization を強化する場合は新しい canonicalization rule version を定義します。

### 3.6 Booleans and null

次の lowercase token を使用します。

```text
true
false
null
```

### 3.7 Whitespace

object member、array element、colon、comma の前後に空白を追加しません。

canonical JSON の末尾に改行を追加しません。

### 3.8 Missing values and empty values

欠落 field と、明示的な `null`、空 object、空 array、空 string は区別します。

canonicalization は optional field を追加・削除しません。default value の補完が必要な場合は、canonicalization の前段となる versioned normalization rule で処理します。

## 4. Canonical bytes API

参照実装は、少なくとも次の操作を公開または内部的に一意に実行できる必要があります。

```text
canonical_json(value) -> string
canonical_bytes(value) -> UTF-8 bytes
```

現行参照実装では次が対応します。

| 実装 | 操作 |
|---|---|
| Rust | `normalize_json` + `to_canonical_json` |
| JavaScript | `sortKeys` + `JSON.stringify` |

## 5. Identity and signing

identity key または署名 payload を生成するときは、対象となる JSON value を先に canonicalize します。

```text
semantic basis
  -> lb.canonical.json.v1
  -> canonical UTF-8 bytes
  -> hash or signature
```

canonicalization rule version は、identity rule version および signature rule version から参照できるようにします。

## 6. Compatibility

`lb.canonical.json.v1` の出力を変更してはいけません。

規則変更が必要な場合は、たとえば次のような新しい version を作ります。

```text
lb.canonical.json.v2
```

node capability は、対応する canonicalization rule version を宣言できるべきです。

## 7. Conformance fixtures

共通 fixture は次に置きます。

```text
conformance/canonicalization/
```

各 test case は、少なくとも input JSON と expected canonical JSON を持ちます。

適合実装は次を確認します。

1. input を parse できる
2. expected と完全に同じ canonical string を生成する
3. canonical string の UTF-8 bytes が一致する
4. canonical output を再度 canonicalize しても結果が変わらない

## 8. 初期 test case

`object-key-order` は次を検証します。

- root object member ordering
- nested object member ordering
- array ordering preservation
- array 内 object の member ordering
- non-ASCII UTF-8 string
- whitespace removal
- idempotence

## 9. 非目標

この version では次を行いません。

- semantic field の追加や削除
- timestamp の timezone 変換
- language tag の case normalization
- Unicode NFC normalization
- relation や label array の semantic sorting
- cryptographic hashing algorithm の変更

これらは normalization、identity、application profile の別仕様で扱います。
