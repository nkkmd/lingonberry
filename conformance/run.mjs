#!/usr/bin/env node

import assert from 'node:assert/strict';
import { createHash, createPublicKey, verify } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const manifest = JSON.parse(await readFile(resolve(root, 'manifest.v1.json'), 'utf8'));
const semanticFieldsV2 = ['type','createdAt','body','contexts','relations','status','lineage','attachments','labels'];

function sortKeys(value) {
  if (Array.isArray(value)) return value.map(sortKeys);
  if (value !== null && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map((key) => [key, sortKeys(value[key])]));
  }
  return value;
}
function canonicalJson(value) { return JSON.stringify(sortKeys(value)); }
function identityKeyV2(value) {
  const basis = {};
  for (const field of semanticFieldsV2) if (Object.hasOwn(value, field)) basis[field] = value[field];
  return `lb:key:lb.identity.key.v2:sha256:${createHash('sha256').update(canonicalJson(basis), 'utf8').digest('hex')}`;
}
function httpPublishSignatureTarget(request) {
  const target = structuredClone(request);
  assert.equal(typeof target.publisher, 'object');
  assert.notEqual(target.publisher, null);
  assert.equal(typeof target.publisher.signature, 'string');
  delete target.publisher.signature;
  return canonicalJson(target);
}
function classifyHttpPublishSignature(request, target) {
  if (!/^[0-9a-f]{64}$/.test(request.publisher.publicKey) || !/^[0-9a-f]{128}$/.test(request.publisher.signature)) {
    return 'malformed';
  }
  try {
    const spkiPrefix = Buffer.from('302a300506032b6570032100', 'hex');
    const key = createPublicKey({
      key: Buffer.concat([spkiPrefix, Buffer.from(request.publisher.publicKey, 'hex')]),
      format: 'der', type: 'spki',
    });
    return verify(null, Buffer.from(target, 'utf8'), key, Buffer.from(request.publisher.signature, 'hex')) ? 'valid' : 'invalid';
  } catch {
    return 'malformed';
  }
}
function fnv1a64Lines(lines) {
  let digest = 0xcbf29ce484222325n;
  for (const line of lines) {
    for (const byte of Buffer.concat([Buffer.from(line, 'utf8'), Buffer.from([0x0a])])) {
      digest ^= BigInt(byte);
      digest = (digest * 0x100000001b3n) & 0xffffffffffffffffn;
    }
  }
  return `fnv1a64:${digest.toString(16).padStart(16, '0')}`;
}
function indexGenerationDigest(input) {
  const canonicalId = input.object.id;
  assert.equal(typeof canonicalId, 'string');
  const recordFingerprint = fnv1a64Lines([input.carrierIdentity, input.storedAt, canonicalJson(input.object)]);
  const idDigest = fnv1a64Lines([canonicalId]);
  const contentDigest = fnv1a64Lines([`${canonicalId}\0${recordFingerprint}`]);
  return {ruleVersion:'lb.index.generation.v1',recordFingerprint,recordCount:1,idDigest,contentDigest,generation:`idx:${idDigest}`};
}
async function read(relativePath) { return readFile(resolve(root, relativePath), 'utf8'); }

const results = [];
for (const testCase of manifest.cases) {
  try {
    if (testCase.kind === 'canonicalization') {
      const input = JSON.parse(await read(testCase.input));
      const expected = await read(testCase.expected);
      const actual = canonicalJson(input);
      assert.equal(actual, expected);
      assert.equal(canonicalJson(JSON.parse(actual)), actual);
    } else if (testCase.kind === 'identity-key') {
      assert.equal(identityKeyV2(JSON.parse(await read(testCase.input))), (await read(testCase.expected)).trimEnd());
    } else if (testCase.kind === 'identity-key-equivalence') {
      assert.equal(identityKeyV2(JSON.parse(await read(testCase.input))), identityKeyV2(JSON.parse(await read(testCase.alternateInput))));
    } else if (testCase.kind === 'http-publish-signature') {
      const input = JSON.parse(await read(testCase.input));
      const target = httpPublishSignatureTarget(input);
      assert.equal(classifyHttpPublishSignature(input, target), testCase.expectedVerification);
      if (testCase.target) assert.equal(target, await read(testCase.target));
      if (testCase.expected) {
        const expected = JSON.parse(await read(testCase.expected));
        assert.equal(createHash('sha256').update(target, 'utf8').digest('hex'), expected.targetSha256);
        assert.equal(input.publisher.publicKey, expected.publicKey);
        assert.equal(input.publisher.signature, expected.signature);
        assert.equal(expected.verification, testCase.expectedVerification);
      }
    } else if (testCase.kind === 'index-generation-digest') {
      assert.deepEqual(indexGenerationDigest(JSON.parse(await read(testCase.input))), JSON.parse(await read(testCase.expected)));
    } else {
      throw new Error(`unsupported conformance case kind: ${testCase.kind}`);
    }
    results.push({id:testCase.id,status:'pass'});
  } catch (error) {
    results.push({id:testCase.id,status:'fail',error:error.message});
  }
}
process.stdout.write(`${JSON.stringify({manifestVersion:manifest.manifestVersion,results}, null, 2)}\n`);
if (results.some((result) => result.status === 'fail')) process.exitCode = 1;
