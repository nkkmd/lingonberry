import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const fixture = JSON.parse(await readFile(new URL('./transition-evidence-generation/minimal-supported-set.input.json', import.meta.url), 'utf8'));
const kindOrder = new Map([['target',0],['transition',1],['delegation',2],['revocation',3]]);

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
    assert.equal(item.classification, 'supported');
    assert.match(item.digest, /^sha256:[0-9a-f]{64}$/);
    const key = `${item.kind}\0${item.id}`;
    const prior = seen.get(key);
    if (prior) {
      assert.deepEqual(item, prior, 'same evidence id must not resolve to conflicting content');
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
