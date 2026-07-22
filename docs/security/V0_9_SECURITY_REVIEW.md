# v0.9.0 Security Review

**Status: active review** | **Release target: v0.9.0** | **Last updated: 2026-07-22**

## 1. Purpose

この文書は、Lingonberry v0.9.0のsecurity reviewにおける対象、trust boundary、abuse case、verification method、finding dispositionを固定する。レビューは「既知の正常入力で動作すること」ではなく、敵対的入力、矛盾状態、部分的I/O、process interruptionの下でもfail closedとdurability boundaryが維持されることを確認する。

## 2. Security invariants

1. validation未通過objectをcanonical storageへ保存しない。
2. identity、signature、digest verificationを迂回できない。
3. duplicateとconflictを混同せず、conflictで既存objectを上書きしない。
4. unknown、corrupt、contradictory、partial stateを成功扱いしない。
5. ordinary startupからimplicit migration、repair、destructive operationを実行しない。
6. backup、restore、replacement、cleanup、migrationは対象とevidenceを厳密にbindingする。
7. active state／data directory、non-empty target、symlink targetへrestoreしない。
8. canonical storageを正本とし、indexをsemantic authorityにしない。
9. authorizationは対象解決、validation、precondition確認より前倒しで副作用を許可しない。
10. attacker-controlled値をsecret、filesystem path、unbounded metric labelとして露出しない。

## 3. Trust boundaries

| Boundary | Untrusted input | Protected asset | Required behavior |
|---|---|---|---|
| Relay ingress | HTTP body、headers、object envelope | canonical storage、CPU／memory | size／depth制限、deterministic validation、fail closed |
| Identity verification | public key、signature、digest対象 | authenticity contract | canonical bytesとのexact binding、bypass不可 |
| Filesystem operations | configured path、archive entry、workspace state | data directory、backup、host filesystem | traversal／symlink拒否、same-filesystem前提の検証 |
| Admin operations | token、role、request ordering | quarantine、replacement、cleanup、restore | authentication後にauthorization、対象とproofのbinding |
| Journal／recovery | partial／malformed journal | last-known-good state | contradictory state拒否、idempotent resume／rollback |
| Index reader | malformed／stale segment | query correctness、availability | storage authority維持、panic／OOB／unbounded allocation禁止 |
| Diagnostics | config、error、object metadata | secrets、privacy、availability | redaction、bounded cardinality、safe error detail |

## 4. Review matrix

### 4.1 Path traversal

- archive entry、object-derived filename、configured directory、temporary workspaceを対象にする。
- absolute path、`..`、mixed separator、prefix collision、normalization差異を試験する。
- canonicalized pathが許可root配下であることを、mutation直前にも確認する。
- reject時にtarget外へfile／directoryを作成していないことを確認する。

### 4.2 Symlink handling

- state、data、backup、restore target、workspace、archive destinationでsymlinkを試験する。
- dangling symlink、relative symlink、parent component symlink、raceによる置換を含める。
- `metadata`と`symlink_metadata`の取り違え、canonicalize後のTOCTOUを監査する。

### 4.3 Oversized／deeply nested input

- request body、JSON、tag／relation array、string、archive、journal、index segmentを対象にする。
- byte size、collection count、nesting depth、decoded expansionに独立した上限を要求する。
- rejectionはbounded time／memoryで完了し、partial canonical writeを残さない。

### 4.4 Malformed serialization

- invalid UTF-8、duplicate key、unknown field、number edge、trailing data、truncationを試験する。
- parser間でacceptance差異がないことを確認する。
- parse failureをinternal errorやsuccessへ変換しない。

### 4.5 Signature verification bypass

- canonicalization差異、field reorder、Unicode normalization、digest substitution、algorithm confusionを試験する。
- identifier、digest、signatureが同一canonical representationへexact bindingされることを確認する。
- legacy／unknown algorithmを明示的に拒否する。

### 4.6 Authorization ordering

- unauthenticated、wrong role、expired credential、対象不存在、invalid proofの組合せを試験する。
- unauthorized requestがexistence、state、path、timing detailを過剰に漏らさない。
- authorization failure前後にmutation、workspace creation、journal appendを行わない。

### 4.7 Information leakage

- effective configuration、logs、diagnostics、HTTP error、metricsを対象にする。
- token、secret、full filesystem path、raw object content、high-cardinality identifierを出力しない。
- operator向けdetailとremote client向けdetailを分離する。

### 4.8 TOCTOU

- plan／apply、inspect／migrate、verify／commit、prepare／publish、authorize／cleanupを対象にする。
- immutable evidence、subject binding、precondition revalidationをmutation直前に要求する。
- path／inode／generation／digestの差し替えを検出して停止する。

### 4.9 Disk-full／I/O failure

- create、write、flush、fsync、rename、directory fsync、deleteの各failure pointを試験する。
- durable commit前にsuccessを報告しない。
- partial file、partial journal、staging directoryからdeterministicにresume／rollback／rejectできる。
- I/O errorをnot-foundやsuccessful cleanupとして扱わない。

## 5. Finding record

各findingは次を必須項目とする。

```text
ID
component / boundary
severity
status
attack precondition
impact
reproduction
root cause
fix
regression test
compatibility impact
owner
disposition
```

Statusは`open`、`fix-in-progress`、`fixed-pending-verification`、`closed`、`accepted-medium-or-low`のいずれかとする。Critical／Highにrisk acceptanceは認めない。

## 6. Verification evidence

- targeted unit／integration test
- property testまたはfuzz regression corpus
- crash／I/O failure injection result
- code review記録
- CI run URLまたはrun ID
- manual reviewの場合は対象commitとreviewer、実施日

## 7. Initial review inventory

優先して監査する領域:

1. protocol parse／canonical serialization／identifier／digest
2. validationとidentity verification ordering
3. relay request size／error mapping／admin authorization
4. storage migration／backup／restore path handling
5. replacement／cleanup proof、workspace、TOCTOU boundary
6. journal parser／recovery classifier
7. index segment／checkpoint reader
8. operator diagnostics、logs、metrics redaction

## 8. Release gate

v0.9.0 release時点で次を満たす。

- Critical／High findingがゼロ
- 修正済みfindingにregression testがある
- Medium findingはknown issue、回避策、修正targetが明記される
- review対象commitがrelease candidate commitと一致する
- security-sensitive contract変更がprotocol／API／operations文書へ反映される
