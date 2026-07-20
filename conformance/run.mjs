#!/usr/bin/env node

import assert from 'node:assert/strict';
import { createHash, createPublicKey, verify } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const manifestPath = resolve(root, 'manifest.v1.json');
const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));

const semanticFieldsV2 = [
  'type', 'createdAt', 'body', 'contexts', 'relations',
  'status', 'lineage', 'attachments', 'labels',
];

function sortKeys(value) {
  if (Array.isArray(value)) return value.map(sortKeys);
  if (value !== null && typeof value === 'object') {
    return Object.fromEntries(
      Object.keys(value).sort().map((key) => [key, sortKeys(value[key])]),
    );
  }
  return value;
}

function canonicalJson(value) {
  return JSON.stringify(sortKeys(value));
}

function identityKeyV2(value) {
  const basis = {};
  for (const field of semanticFieldsV2) {
    if (Object.hasOwn(value, field)) basis[field] = value[field];
  }
  const digest = createHash('sha256').update(canonicalJson(basis), 'utf8').digest('hex');
  return `lb:key:lb.identity.key.v2:sha256:${digest}`;
}

function httpPublishSignatureTarget(request) {
  const target = structuredClone(request);
  assert.equal(typeof target.publisher, 'object');
  assert.notEqual(target.publisher, null);
  assert.equal(typeof target.publisher.signature, 'string');
  delete target.publisher.signature;
  return canonicalJson(target);
}

function ed25519PublicKeyFromRawHex(publicKeyHex) {
  assert.match(publicKeyHex, /^[0-9a-f]{64}$/);
  const spkiPrefix = Buffer.from('302a300506032b6570032100', 'hex');
  return createPublicKey({
    key: Buffer.concat([spkiPrefix, Buffer.from(publicKeyHex, 'hex')]),
    format: 'der',
    type: 'spki',
  });
}

function fnv1a64Lines(lines) {
  const offsetBasis = 0xcbf29ce484222325n;
  const prime = 0x100000001b3n;
  const mask = 0xffffffffffffffffn;
  let digest = offsetBasis;
  for (const line of lines) {
    for (const byte of Buffer.concat([Buffer.from(line, 'utf8'), Buffer.from([0x0a])])) {
      digest ^= BigInt(byte);
      digest = (digest * prime) & mask;
    }
  }
  return `fnv1a64:${digest.toString(16).padStart(16, '0')}`;
}

function indexGenerationDigest(input) {
  const canonicalId = input.object.id;
  assert.equal(typeof canonicalId, 'string');
  const recordFingerprint = fnv1a64Lines([
    input.carrierIdentity,
    input.storedAt,
    canonicalJson(input.object),
  ]);
  const idDigest = fnv1a64Lines([canonicalId]);
  const contentDigest = fnv1a64Lines([`${canonicalId}\0${recordFingerprint}`]);
  return {
    ruleVersion: 'lb.index.generation.v1',
    recordFingerprint,
    recordCount: 1,
    idDigest,
    contentDigest,
    generation: `idx:${idDigest}`,
  };
}

async function read(relativePath) {
  return readFile(resolve(root, relativePath), 'utf8');
}

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
      const input = JSON.parse(await read(testCase.input));
      const expected = (await read(testCase.expected)).trimEnd();
      assert.equal(identityKeyV2(input), expected);
    } else if (testCase.kind === 'identity-key-equivalence') {
      const input = JSON.parse(await read(testCase.input));
      const alternate = JSON.parse(await read(testCase.alternateInput));
      assert.equal(identityKeyV2(input), identityKeyV2(alternate));
    } else if (testCase.kind === 'http-publish-signature') {
      const input = JSON.parse(await read(testCase.input));
      const actualTarget = httpPublishSignatureTarget(input);
      const actualVerification = verify(
        null,
        Buffer.from(actualTarget, 'utf8'),
        ed25519PublicKeyFromRawHex(input.publisher.publicKey),
        Buffer.from(input.publisher.signature, 'hex'),
      ) ? 'valid' : 'invalid';
      assert.equal(actualVerification, testCase.expectedVerification);
      if (testCase.target) assert.equal(actualTarget, await read(testCase.target));
      if (testCase.expected) {
        const expected = JSON.parse(await read(testCase.expected));
        assert.equal(createHash('sha256').update(actualTarget, 'utf8').digest('hex'), expected.targetSha256);
        assert.equal(input.publisher.publicKey, expected.publicKey);
        assert.equal(input.publisher.signature, expected.signature);
        assert.equal(expected.verification, testCase.expectedVerification);
      }
    } else if (testCase.kind === 'index-generation-digest') {
      const input = JSON.parse(await read(testCase.input));
      const expected = JSON.parse(await read(testCase.expected));
      assert.deepEqual(indexGenerationDigest(input), expected);
    } else {
      throw new Error(`unsupported conformance case kind: ${testCase.kind}`);
    }
    results.push({ id: testCase.id, status: 'pass' });
  } catch (error) {
    results.push({ id: testCase.id, status: 'fail', error: error.message });
  }
}

const failed = results.filter((result) => result.status === 'fail');
process.stdout.write(`${JSON.stringify({ manifestVersion: manifest.manifestVersion, results }, null, 2)}\n`);
if (failed.length > 0) process.exitCode = 1;
