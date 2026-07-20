#!/usr/bin/env node

import assert from 'node:assert/strict';
import { createHash, createPublicKey, verify } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const manifest = JSON.parse(await readFile(resolve(root, 'manifest.v1.json'), 'utf8'));
const semanticFields = ['type','createdAt','body','contexts','relations','status','lineage','attachments','labels'];
const transitionFields = ['objectType','transitionType','targetId','replacementId','supersedesTransitionIds','issuedAt','reason'];
const protocolIdPattern = /^lb:(?:obj|transition|key):[A-Za-z0-9._~:-]+$/;
const objectIdPattern = /^lb:obj:[A-Za-z0-9._~:-]+$/;
const transitionIdPattern = /^lb:transition:[A-Za-z0-9._~:-]+$/;
const keyIdPattern = /^lb:key:[A-Za-z0-9._~:-]+$/;
function byteLength(value) { return Buffer.byteLength(value, 'utf8'); }
function isObjectId(value) { return typeof value === 'string' && objectIdPattern.test(value) && byteLength(value) <= 255; }
function isTransitionId(value) { return typeof value === 'string' && transitionIdPattern.test(value) && byteLength(value) <= 255; }
function isKeyId(value) { return typeof value === 'string' && keyIdPattern.test(value) && byteLength(value) <= 512; }
function isProtocolId(value) {
  if (typeof value !== 'string' || !protocolIdPattern.test(value)) return false;
  if (value.startsWith('lb:obj:')) return isObjectId(value);
  if (value.startsWith('lb:transition:')) return isTransitionId(value);
  return isKeyId(value);
}
function sortKeys(value) { if (Array.isArray(value)) return value.map(sortKeys); if (value !== null && typeof value === 'object') return Object.fromEntries(Object.keys(value).sort().map((key) => [key, sortKeys(value[key])])); return value; }
function canonicalJson(value) { return JSON.stringify(sortKeys(value)); }
function selectedBasis(value, fields) { const basis = {}; for (const field of fields) if (Object.hasOwn(value, field)) basis[field] = value[field]; return basis; }
function semanticBasis(value) { return selectedBasis(value, semanticFields); }
function transitionBasis(value) { const basis = selectedBasis(value, transitionFields); if (Array.isArray(basis.supersedesTransitionIds)) basis.supersedesTransitionIds = [...basis.supersedesTransitionIds].sort(); return basis; }
function fnv1a64(input) { let digest = 0xcbf29ce484222325n; for (const byte of Buffer.from(input, 'utf8')) { digest ^= BigInt(byte); digest = (digest * 0x100000001b3n) & 0xffffffffffffffffn; } return digest.toString(16).padStart(16, '0'); }
function identityKeyV1(value) { return `lb:key:lb.identity.key.v1:fnv1a64:${fnv1a64(canonicalJson(semanticBasis(value)))}`; }
function identityKeyV2(value) { return `lb:key:lb.identity.key.v2:sha256:${createHash('sha256').update(canonicalJson(semanticBasis(value)), 'utf8').digest('hex')}`; }
function transitionIdentity(value) { return `lb:key:lb.transition.identity.v1:sha256:${createHash('sha256').update(canonicalJson(transitionBasis(value)), 'utf8').digest('hex')}`; }
function classifyTimestamp(value) { if (typeof value !== 'string') return 'invalid'; return /^\d{4}-(0[1-9]|1[0-2])-([0-2]\d|3[01])T([01]\d|2[0-3]):[0-5]\d:(?:[0-5]\d|60)(?:\.\d+)?Z$/.test(value) ? 'valid' : 'invalid'; }
function classifyTransition(value) {
  if (value?.objectType !== 'transition' || value?.schemaVersion !== '0.1.0') return 'invalid';
  if (!isTransitionId(value.id) || !isObjectId(value.targetId)) return 'invalid';
  if (value.supersedesTransitionIds !== undefined) {
    if (!Array.isArray(value.supersedesTransitionIds) || value.supersedesTransitionIds.length === 0) return 'invalid';
    if (new Set(value.supersedesTransitionIds).size !== value.supersedesTransitionIds.length) return 'invalid';
    if (value.supersedesTransitionIds.some((id) => !isTransitionId(id) || id === value.id)) return 'invalid';
  }
  if (classifyTimestamp(value.issuedAt) !== 'valid' || !Array.isArray(value.provenance?.sources) || value.provenance.sources.length === 0) return 'invalid';
  if (typeof value.rawRef?.protocol !== 'string' || typeof value.rawRef?.sourceId !== 'string') return 'invalid';
  if (value.transitionType === 'replace') return isObjectId(value.replacementId) && value.replacementId !== value.targetId ? 'valid' : 'invalid';
  if (value.transitionType === 'withdraw') return Object.hasOwn(value, 'replacementId') ? 'invalid' : 'valid';
  return 'invalid';
}
function classifyTransitionAuthority(input) {
  const retain = true;
  if (input.targetPublisherKey == null) return {classification:'unknown',basis:'target-publisher-unknown',retain,applyToEffectiveView:false};
  if (input.transitionPublisherKey === input.targetPublisherKey) return {classification:'authorized',basis:'original-publisher',retain,applyToEffectiveView:true};
  let incomplete = false;
  for (const delegation of input.delegations ?? []) {
    if (delegation.verified !== true) { incomplete = true; continue; }
    if (delegation.issuerKey !== input.targetPublisherKey || delegation.delegateKey !== input.transitionPublisherKey) continue;
    if (!Array.isArray(delegation.scopes) || !delegation.scopes.includes('transition')) continue;
    if (classifyTimestamp(delegation.validFrom) !== 'valid' || classifyTimestamp(delegation.validUntil) !== 'valid') { incomplete = true; continue; }
    if (delegation.revokedAt && classifyTimestamp(delegation.revokedAt) !== 'valid') { incomplete = true; continue; }
    if (input.issuedAt < delegation.validFrom || input.issuedAt > delegation.validUntil) continue;
    if (delegation.revokedAt && delegation.revokedAt <= input.issuedAt) continue;
    return {classification:'authorized',basis:'delegated-publisher',retain,applyToEffectiveView:true};
  }
  if (incomplete) return {classification:'unknown',basis:'authority-evidence-incomplete',retain,applyToEffectiveView:false};
  return {classification:'unauthorized',basis:'no-applicable-authority',retain,applyToEffectiveView:false};
}
function projectTransitions(input) {
  const authorized = input.transitions.filter((item) => item.authority === 'authorized');
  const byId = new Map(authorized.map((item) => [item.id, item]));
  const superseded = new Set();
  const edges = new Map();
  for (const item of authorized) {
    if (!isTransitionId(item.id) || !isObjectId(item.targetId)) return {classification:'invalid-transition-graph'};
    const parents = item.supersedesTransitionIds ?? [];
    if (!Array.isArray(parents) || new Set(parents).size !== parents.length) return {classification:'invalid-transition-graph'};
    edges.set(item.id, parents);
    for (const parentId of parents) {
      const prior = byId.get(parentId);
      if (!isTransitionId(parentId) || !prior || prior.targetId !== input.targetId || item.targetId !== input.targetId || prior.id === item.id) return {classification:'invalid-transition-graph'};
      superseded.add(prior.id);
    }
  }
  const visiting = new Set(); const visited = new Set();
  function hasCycle(id) { if (visiting.has(id)) return true; if (visited.has(id)) return false; visiting.add(id); for (const parentId of edges.get(id) ?? []) if (hasCycle(parentId)) return true; visiting.delete(id); visited.add(id); return false; }
  for (const id of byId.keys()) if (hasCycle(id)) return {classification:'invalid-transition-graph'};
  const heads = authorized.filter((item) => item.targetId === input.targetId && !superseded.has(item.id));
  if (heads.length === 0) return {classification:'active-original'};
  if (heads.length > 1) return {classification:'ambiguous',headTransitionIds:heads.map((item) => item.id).sort()};
  const head = heads[0];
  if (head.transitionType === 'replace') return {classification:'replaced',effectiveTransitionId:head.id,replacementId:head.replacementId};
  return {classification:'withdrawn',effectiveTransitionId:head.id};
}
function httpPublishSignatureTarget(request) { const target = structuredClone(request); assert.equal(typeof target.publisher, 'object'); assert.notEqual(target.publisher, null); assert.equal(typeof target.publisher.signature, 'string'); delete target.publisher.signature; return canonicalJson(target); }
function classifyHttpPublishSignature(request, target) { if (!/^[0-9a-f]{64}$/.test(request.publisher.publicKey) || !/^[0-9a-f]{128}$/.test(request.publisher.signature)) return 'malformed'; try { const prefix = Buffer.from('302a300506032b6570032100', 'hex'); const key = createPublicKey({key:Buffer.concat([prefix,Buffer.from(request.publisher.publicKey,'hex')]),format:'der',type:'spki'}); return verify(null, Buffer.from(target,'utf8'), key, Buffer.from(request.publisher.signature,'hex')) ? 'valid' : 'invalid'; } catch { return 'malformed'; } }
function fnv1a64Lines(lines) { let digest = 0xcbf29ce484222325n; for (const line of lines) for (const byte of Buffer.concat([Buffer.from(line,'utf8'),Buffer.from([0x0a])])) { digest ^= BigInt(byte); digest = (digest * 0x100000001b3n) & 0xffffffffffffffffn; } return `fnv1a64:${digest.toString(16).padStart(16,'0')}`; }
function indexGenerationDigest(input) { const canonicalId = input.object.id; assert.equal(typeof canonicalId, 'string'); const recordFingerprint = fnv1a64Lines([input.carrierIdentity,input.storedAt,canonicalJson(input.object)]); const idDigest = fnv1a64Lines([canonicalId]); const contentDigest = fnv1a64Lines([`${canonicalId}\0${recordFingerprint}`]); return {ruleVersion:'lb.index.generation.v1',recordFingerprint,recordCount:1,idDigest,contentDigest,generation:`idx:${idDigest}`}; }
function buildBoundaryId(testCase) {
  const prefix = testCase.kind === 'object' ? 'lb:obj:' : testCase.kind === 'transition' ? 'lb:transition:' : 'lb:key:';
  assert.ok(testCase.totalBytes > prefix.length);
  return prefix + 'a'.repeat(testCase.totalBytes - prefix.length);
}
async function read(relativePath) { return readFile(resolve(root, relativePath), 'utf8'); }
const results = [];
for (const testCase of manifest.cases) {
  try {
    if (testCase.kind === 'canonicalization') { const input = JSON.parse(await read(testCase.input)); const expected = await read(testCase.expected); const actual = canonicalJson(input); assert.equal(actual, expected); assert.equal(canonicalJson(JSON.parse(actual)), actual); }
    else if (testCase.kind === 'protocol-id') { const input = JSON.parse(await read(testCase.input)); const classification = input.values.every((value) => isProtocolId(value)) ? 'valid' : 'invalid'; assert.equal(classification, input.expectedClassification); }
    else if (testCase.kind === 'protocol-id-boundaries') { const input = JSON.parse(await read(testCase.input)); for (const boundary of input.cases) { const value = buildBoundaryId(boundary); const classification = isProtocolId(value) ? 'valid' : 'invalid'; assert.equal(classification, boundary.expectedClassification); assert.equal(byteLength(value), boundary.totalBytes); } }
    else if (testCase.kind === 'identity-key') assert.equal(identityKeyV2(JSON.parse(await read(testCase.input))), (await read(testCase.expected)).trimEnd());
    else if (testCase.kind === 'identity-key-equivalence') assert.equal(identityKeyV2(JSON.parse(await read(testCase.input))), identityKeyV2(JSON.parse(await read(testCase.alternateInput))));
    else if (testCase.kind === 'http-publish-signature') { const input = JSON.parse(await read(testCase.input)); const target = httpPublishSignatureTarget(input); assert.equal(classifyHttpPublishSignature(input,target), testCase.expectedVerification); if (testCase.target) assert.equal(target, await read(testCase.target)); if (testCase.expected) { const expected = JSON.parse(await read(testCase.expected)); assert.equal(createHash('sha256').update(target,'utf8').digest('hex'), expected.targetSha256); assert.equal(input.publisher.publicKey, expected.publicKey); assert.equal(input.publisher.signature, expected.signature); } }
    else if (testCase.kind === 'index-generation-digest') assert.deepEqual(indexGenerationDigest(JSON.parse(await read(testCase.input))), JSON.parse(await read(testCase.expected)));
    else if (testCase.kind === 'timestamp') { const input = JSON.parse(await read(testCase.input)); assert.equal(classifyTimestamp(input.createdAt), input.expectedClassification); if (input.expectedCanonicalValue) assert.equal(JSON.parse(canonicalJson({createdAt:input.createdAt})).createdAt, input.expectedCanonicalValue); }
    else if (testCase.kind === 'legacy-identity-v1') { const input = JSON.parse(await read(testCase.input)); const claim = input.identityClaims?.[0]; assert.equal(claim?.ruleVersion, 'lb.identity.key.v1'); assert.equal(claim?.canonicalId, input.id); assert.equal(claim?.identityKey, identityKeyV1(input)); assert.equal(testCase.expectedCompatibility, 'compatible'); }
    else if (testCase.kind === 'transition-object') { const input = JSON.parse(await read(testCase.input)); assert.equal(classifyTransition(input), testCase.expectedClassification); if (testCase.expected) assert.equal(transitionIdentity(input), (await read(testCase.expected)).trimEnd()); }
    else if (testCase.kind === 'transition-identity-equivalence') { const input = JSON.parse(await read(testCase.input)); const alternate = JSON.parse(await read(testCase.alternateInput)); assert.equal(classifyTransition(input), 'valid'); assert.equal(classifyTransition(alternate), 'valid'); assert.equal(transitionIdentity(input), transitionIdentity(alternate)); }
    else if (testCase.kind === 'transition-authority') { const input = JSON.parse(await read(testCase.input)); assert.deepEqual(classifyTransitionAuthority(input), input.expected); }
    else if (testCase.kind === 'transition-supersession') { const input = JSON.parse(await read(testCase.input)); assert.deepEqual(projectTransitions(input), input.expected); }
    else throw new Error(`unsupported conformance case kind: ${testCase.kind}`);
    results.push({id:testCase.id,suite:testCase.suite,status:'pass'});
  } catch (error) { results.push({id:testCase.id,suite:testCase.suite,status:'fail',error:error.message}); }
}
const suites = Object.fromEntries(['producer','consumer','internal'].map((suite) => { const selected = results.filter((result) => result.suite === suite); return [suite,{status:selected.some((result) => result.status === 'fail') ? 'fail' : 'pass',cases:selected.length}]; }));
process.stdout.write(`${JSON.stringify({manifestVersion:manifest.manifestVersion,suites,results}, null, 2)}\n`);
if (results.some((result) => result.status === 'fail')) process.exitCode = 1;
