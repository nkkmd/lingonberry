import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const fixture = JSON.parse(await readFile(
  new URL('./transition-evidence-generation/diagnostic-retention-hybrid.input.json', import.meta.url),
  'utf8',
));

function classifyHybridRetention(input) {
  assert.equal(input.policy.maximumRecentGenerations, 8);
  assert.equal(input.policy.maximumAgeSeconds, 86400);

  const now = Date.parse(input.now);
  const nonProtected = input.snapshots
    .filter((snapshot) => !snapshot.currentObservation && !snapshot.semanticCheckpoint && !snapshot.activeCursorLease)
    .sort((a, b) => Date.parse(b.observedAt) - Date.parse(a.observedAt)
      || Buffer.compare(Buffer.from(a.generation, 'ascii'), Buffer.from(b.generation, 'ascii')));

  const recentPolicyRetained = new Set(
    nonProtected
      .filter((snapshot, index) => {
        const ageSeconds = (now - Date.parse(snapshot.observedAt)) / 1000;
        return index < input.policy.maximumRecentGenerations
          && ageSeconds <= input.policy.maximumAgeSeconds;
      })
      .map((snapshot) => snapshot.generation),
  );

  const protectedOverrideGenerations = input.snapshots
    .filter((snapshot) => snapshot.currentObservation || snapshot.semanticCheckpoint || snapshot.activeCursorLease)
    .map((snapshot) => snapshot.generation);

  const collectibleGenerations = nonProtected
    .filter((snapshot) => !recentPolicyRetained.has(snapshot.generation))
    .map((snapshot) => snapshot.generation);

  return {
    recentPolicyRetainedGenerations: [...recentPolicyRetained],
    protectedOverrideGenerations,
    collectibleGenerations,
    countBound: input.policy.maximumRecentGenerations,
    ageBoundSeconds: input.policy.maximumAgeSeconds,
    requiresBothRecentConditions: true,
    canonicalEvidenceDeleted: false,
  };
}

test('recent diagnostic retention requires both count and age bounds while protected generations override policy', () => {
  assert.deepEqual(classifyHybridRetention(fixture), fixture.expected);
});

test('a generation exactly on the age boundary is age-eligible but still collectible when outside the count bound', () => {
  const boundary = fixture.snapshots.find((snapshot) => snapshot.generation.endsWith('g-boundary'));
  assert.equal((Date.parse(fixture.now) - Date.parse(boundary.observedAt)) / 1000, fixture.policy.maximumAgeSeconds);
  assert.ok(fixture.expected.collectibleGenerations.includes(boundary.generation));
});
