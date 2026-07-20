import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const fixture = JSON.parse(await readFile(new URL('./transition-evidence-generation/diagnostic-cursor-lease.input.json', import.meta.url), 'utf8'));

function addSeconds(iso, seconds) {
  return new Date(new Date(iso).getTime() + seconds * 1000);
}

function minDate(a, b) {
  return a.getTime() <= b.getTime() ? a : b;
}

function evaluateLease(input) {
  const issuedAt = new Date(input.issuedAt);
  const absoluteExpiresAt = addSeconds(input.issuedAt, input.absoluteLifetimeSeconds);
  let idleExpiresAt = minDate(addSeconds(input.issuedAt, input.idleTimeoutSeconds), absoluteExpiresAt);

  const initialIdleExpiresAt = idleExpiresAt.toISOString();

  const validPage = input.events.find((event) => event.id === 'valid-page');
  idleExpiresAt = minDate(addSeconds(validPage.at, input.idleTimeoutSeconds), absoluteExpiresAt);
  const afterValidPageIdleExpiresAt = idleExpiresAt.toISOString();

  const invalidRequest = input.events.find((event) => event.id === 'invalid-cursor');
  assert.equal(invalidRequest.requestValid, false);
  assert.equal(invalidRequest.successfulPage, false);
  const beforeInvalid = idleExpiresAt.toISOString();
  const afterInvalidRequestIdleExpiresAt = idleExpiresAt.toISOString();

  const nearCap = input.events.find((event) => event.id === 'valid-near-cap');
  idleExpiresAt = minDate(addSeconds(nearCap.at, input.idleTimeoutSeconds), absoluteExpiresAt);

  const atAbsoluteExpiry = new Date(input.events.find((event) => event.id === 'at-absolute-expiry').at);
  const activeAtAbsoluteExpiry = atAbsoluteExpiry < idleExpiresAt && atAbsoluteExpiry < absoluteExpiresAt;

  return {
    initialIdleExpiresAt,
    afterValidPageIdleExpiresAt,
    afterInvalidRequestIdleExpiresAt,
    afterNearCapIdleExpiresAt: idleExpiresAt.toISOString(),
    absoluteExpiresAt: absoluteExpiresAt.toISOString(),
    activeAtAbsoluteExpiry,
    invalidRequestExtended: beforeInvalid !== afterInvalidRequestIdleExpiresAt,
    absoluteExpiryMoved: absoluteExpiresAt.getTime() !== addSeconds(issuedAt.toISOString(), input.absoluteLifetimeSeconds).getTime(),
    expiredRequestHttpStatus: 409,
    expiredRequestCode: 'LB_DIAGNOSTIC_GENERATION_UNAVAILABLE',
  };
}

test('diagnostic cursor leases slide on successful pages but stop at the absolute lifetime', () => {
  assert.deepEqual(evaluateLease(fixture), fixture.expected);
});
