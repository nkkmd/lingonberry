import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const fixture = JSON.parse(await readFile(new URL('./transition-evidence-generation/minimal-supported-set.input.json', import.meta.url), 'utf8'));
const unusableFixture = JSON.parse(await readFile(new URL('./transition-evidence-generation/classified-unusable-set.input.json', import.meta.url), 'utf8'));
const staleFixture = JSON.parse(await readFile(new URL('./transition-evidence-generation/last-known-good-stale.input.json', import.meta.url), 'utf8'));
const staleReadFixture = JSON.parse(await readFile(new URL('./transition-evidence-generation/stale-read-api.input.json', import.meta.url), 'utf8'));
const diagnosticFixture = JSON.parse(await readFile(new URL('./transition-evidence-generation/stable-diagnostics.input.json', import.meta.url), 'utf8'));
const kindOrder = new Map([['target',0],['transition',1],['delegation',2],['revocation',3]]);
const classifications = new Set(['supported','unsupported','corrupt','unreadable']);
const diagnosticReasonClassifications = new Map([
  ['LB_EVIDENCE_RULE_UNSUPPORTED','unsupported'],
  ['LB_EVIDENCE_PARSE_FAILED','corrupt'],
  ['LB_EVIDENCE_VALIDATION_FAILED','corrupt'],
  ['LB_EVIDENCE_DIGEST_MISMATCH','corrupt'],
  ['LB_EVIDENCE_SIGNATURE_INVALID','corrupt'],
  ['LB_EVIDENCE_BYTES_UNREADABLE','unreadable'],
  ['LB_EVIDENCE_INVENTORY_CONFLICT','corrupt'],
]);
const diagnosticRequiredFields = new Set(['kind','evidenceId','classification','reasonCode']);
const diagnosticOptionalFields = new Set(['ruleVersion','digest']);

function sortKeys(value) {
  if (Array.isArray(value)) return value.map(sortKeys);
  if (value !== null && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map((key) => [key, sortKeys(value[key])]));
  }
  return value;
}

function canonicalJson(value) {
  return JSON.stringify(sortKeys(value));
}

function evidenceGeneration(input) {
  const seen = new Map();
  for (const item of input.evidence) {
    assert.ok(kindOrder.has(item.kind));
    assert.ok(classifications.has(item.classification));
    assert.match(item.digest, /^sha256:[0-9a-f]{64}$/);
    const key = `${item.kind}\0${item.id}`;
    const prior = seen.get(key);
    if (prior) {
      assert.deepEqual(item, prior, 'same evidence id must not resolve to conflicting content or classification');
      continue;
    }
    seen.set(key, item);
  }
  const evidence = [...seen.values()].sort((a, b) =>
    kindOrder.get(a.kind) - kindOrder.get(b.kind)
      || Buffer.compare(Buffer.from(a.id, 'ascii'), Buffer.from(b.id, 'ascii'))
      || Buffer.compare(Buffer.from(a.classification, 'ascii'), Buffer.from(b.classification, 'ascii'))
      || Buffer.compare(Buffer.from(a.digest, 'ascii'), Buffer.from(b.digest, 'ascii'))
  );
  const basis = {ruleVersion:'lb.transition.evidence-generation.v1',targetId:input.targetId,evidence};
  return `evidence:sha256:${createHash('sha256').update(canonicalJson(basis), 'utf8').digest('hex')}`;
}

function snapshotEffect(input) {
  const incomplete = input.evidence.some((item) => item.classification !== 'supported');
  return {
    snapshotClassification: incomplete ? 'incomplete' : 'complete',
    authorityClassification: incomplete ? 'unknown' : 'evaluated',
    applyToEffectiveView: !incomplete,
  };
}

function preserveLastKnownGood(input) {
  assert.equal(input.observationCheckpoint.snapshotClassification, 'incomplete');
  return {
    effectiveView: {
      ...input.semanticCheckpoint,
      freshness: 'stale',
    },
    evidenceObservation: {
      generation: input.observationCheckpoint.generation,
      snapshotClassification: 'incomplete',
      applyToEffectiveView: false,
    },
    semanticCheckpointAdvanced: false,
    observationCheckpointAdvanced: true,
  };
}

function staleReadResponse(input) {
  assert.equal(input.request.method, 'GET');
  assert.match(input.request.route, /^\/v1\/effective-objects\/lb:obj:/);
  assert.equal(input.observationCheckpoint.snapshotClassification, 'incomplete');
  return {
    httpStatus: 200,
    freshness: 'stale',
    bodyAuthoritative: true,
    semanticGeneration: input.semanticCheckpoint.generation,
    observationGeneration: input.observationCheckpoint.generation,
    diagnosticCount: input.observationCheckpoint.diagnostics.length,
  };
}

function validateAndOrderDiagnostics(input) {
  const seen = new Map();
  for (const diagnostic of input.diagnostics) {
    assert.ok(kindOrder.has(diagnostic.kind));
    assert.notEqual(diagnostic.classification, 'supported');
    assert.equal(diagnosticReasonClassifications.get(diagnostic.reasonCode), diagnostic.classification);
    if (diagnostic.digest !== undefined) assert.match(diagnostic.digest, /^sha256:[0-9a-f]{64}$/);
    const fields = Object.keys(diagnostic);
    for (const required of diagnosticRequiredFields) assert.ok(fields.includes(required));
    for (const field of fields) assert.ok(diagnosticRequiredFields.has(field) || diagnosticOptionalFields.has(field));
    for (const forbidden of input.forbiddenFields) assert.equal(Object.hasOwn(diagnostic, forbidden), false);
    const key = `${diagnostic.kind}\0${diagnostic.evidenceId}`;
    const prior = seen.get(key);
    if (prior) {
      assert.deepEqual(diagnostic, prior, 'conflicting public diagnostics must not be silently selected');
      continue;
    }
    seen.set(key, diagnostic);
  }
  const diagnostics = [...seen.values()].sort((a, b) =>
    kindOrder.get(a.kind) - kindOrder.get(b.kind)
      || Buffer.compare(Buffer.from(a.evidenceId, 'ascii'), Buffer.from(b.evidenceId, 'ascii'))
      || Buffer.compare(Buffer.from(a.classification, 'ascii'), Buffer.from(b.classification, 'ascii'))
      || Buffer.compare(Buffer.from(a.reasonCode, 'ascii'), Buffer.from(b.reasonCode, 'ascii'))
  );
  return {
    valid: true,
    orderedEvidenceIds: diagnostics.map((item) => item.evidenceId),
    publicFieldSets: diagnostics.map((item) => Object.keys(item).sort()),
  };
}

test('target evidence generation is deterministic and order independent', () => {
  assert.equal(evidenceGeneration(fixture), fixture.expectedGeneration);
  assert.equal(evidenceGeneration({...fixture,evidence:[...fixture.evidence].reverse()}), fixture.expectedGeneration);
});

test('exact duplicate evidence carriers do not change the generation', () => {
  assert.equal(evidenceGeneration({...fixture,evidence:[...fixture.evidence, fixture.evidence[0]]}), fixture.expectedGeneration);
});

test('same evidence id with conflicting digest is rejected', () => {
  const conflicting = structuredClone(fixture);
  conflicting.evidence.push({...fixture.evidence[0],digest:`sha256:${'f'.repeat(64)}`});
  assert.throws(() => evidenceGeneration(conflicting));
});

test('classified unusable evidence participates in deterministic generation and fails semantic effect closed', () => {
  const generation = evidenceGeneration(unusableFixture);
  assert.match(generation, /^evidence:sha256:[0-9a-f]{64}$/);
  assert.equal(evidenceGeneration({...unusableFixture,evidence:[...unusableFixture.evidence].reverse()}), generation);
  assert.deepEqual(snapshotEffect(unusableFixture), unusableFixture.expected);
});

test('repairing an unusable marker changes generation', () => {
  const repaired = structuredClone(unusableFixture);
  repaired.evidence[1].classification = 'supported';
  assert.notEqual(evidenceGeneration(repaired), evidenceGeneration(unusableFixture));
});

test('incomplete current observation preserves and marks the last-known-good semantic view stale', () => {
  assert.deepEqual(preserveLastKnownGood(staleFixture), staleFixture.expected);
});

test('read API returns stale last-known-good state with authoritative body diagnostics', () => {
  assert.deepEqual(staleReadResponse(staleReadFixture), staleReadFixture.expected);
});

test('public diagnostics expose only stable protocol fields and deterministic reason codes', () => {
  assert.deepEqual(validateAndOrderDiagnostics(diagnosticFixture), diagnosticFixture.expected);
});

test('public diagnostics reject implementation-specific fields and mismatched reason classifications', () => {
  const leaked = structuredClone(diagnosticFixture);
  leaked.diagnostics[0].storagePath = '/srv/relay/private/evidence.bin';
  assert.throws(() => validateAndOrderDiagnostics(leaked));

  const mismatched = structuredClone(diagnosticFixture);
  mismatched.diagnostics[0].reasonCode = 'LB_EVIDENCE_PARSE_FAILED';
  assert.throws(() => validateAndOrderDiagnostics(mismatched));
});
