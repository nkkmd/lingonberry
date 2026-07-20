import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const fixture = JSON.parse(await readFile(new URL('./transition-evidence-generation/diagnostic-read-guard-heartbeat.input.json', import.meta.url), 'utf8'));

test('read guard heartbeat requires progress and respects the absolute expiry', () => {
  const absoluteExpiresAt = fixture.guardIssuedAtSeconds + fixture.guardAbsoluteSeconds;
  let expiresAt = Math.min(fixture.guardIssuedAtSeconds + fixture.guardIdleSeconds, absoluteExpiresAt);
  let progressToken = 0;

  for (const event of fixture.events) {
    const active = event.atSeconds < expiresAt && event.atSeconds < absoluteExpiresAt;
    const progressed = event.progressToken > progressToken;
    const extend = active && event.validIdentity && progressed;
    if (extend) {
      progressToken = event.progressToken;
      expiresAt = Math.min(event.atSeconds + fixture.guardIdleSeconds, absoluteExpiresAt);
    }
    assert.equal(extend, event.expectedExtended);
    assert.equal(expiresAt, event.expectedExpiresAtSeconds);
  }

  assert.deepEqual({
    absoluteExpiresAtSeconds: absoluteExpiresAt,
    finalExpiresAtSeconds: expiresAt,
    partialPageAllowed: false,
    timerOnlyHeartbeatAllowed: false,
    duplicateProgressAllowed: false,
    failureCode: 'LB_DIAGNOSTIC_PAGE_READ_FAILED',
  }, fixture.expected);
});
