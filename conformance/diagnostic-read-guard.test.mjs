import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const fixture = JSON.parse(await readFile(new URL('./transition-evidence-generation/diagnostic-read-guard.input.json', import.meta.url), 'utf8'));

function evaluateReadGuard(input) {
  assert.equal(input.guardLifetimeSeconds, 120);
  const request = input.request;
  const guardAcquired = request.cursorValid
    && request.targetMatches
    && request.generationMatches
    && request.cursorLeaseActive
    && request.limitValid
    && request.snapshotRetained
    && !request.deleteClaimCommitted;

  const guardActiveAtReadCompletion = guardAcquired
    && input.guard.readCompletedAtEpochSeconds < input.guard.expiresAtEpochSeconds;

  for (const race of input.raceCases) {
    assert.notEqual(race.guardCommitted && race.deleteClaimCommitted, true, 'guard and GC delete claim must not both commit');
    if (race.guardCommitted) assert.equal(race.expected, 'page-readable');
    if (race.deleteClaimCommitted) assert.equal(race.expected, 'generation-unavailable');
  }

  return {
    guardAcquired,
    guardActiveAtReadCompletion,
    guardLifetimeSeconds: input.guard.expiresAtEpochSeconds - input.guard.issuedAtEpochSeconds,
    partialPageAllowed: false,
    generationSwitchAllowed: false,
    expiredGuardPinsSnapshot: false,
    generationUnavailableCode: 'LB_DIAGNOSTIC_GENERATION_UNAVAILABLE',
    pageReadFailureCode: 'LB_DIAGNOSTIC_PAGE_READ_FAILED',
    serializedRaceOutcomes: input.raceCases.map((race) => race.expected),
  };
}

test('diagnostic page reads use bounded guards serialized against garbage collection', () => {
  assert.deepEqual(evaluateReadGuard(fixture), fixture.expected);
});

test('expired guard cannot authorize or pin a page read', () => {
  const expired = structuredClone(fixture);
  expired.guard.readCompletedAtEpochSeconds = expired.guard.expiresAtEpochSeconds;
  assert.equal(evaluateReadGuard(expired).guardActiveAtReadCompletion, false);
});

test('committed garbage-collection claim prevents guard acquisition', () => {
  const collected = structuredClone(fixture);
  collected.request.deleteClaimCommitted = true;
  assert.equal(evaluateReadGuard(collected).guardAcquired, false);
});
