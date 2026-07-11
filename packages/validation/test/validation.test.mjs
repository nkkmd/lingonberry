import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

import {
  IdentityValidationStatus,
  validateKnowledgeObjectFull,
} from '../validation.mjs';

const v1Url = new URL(
  '../../../fixtures/knowledge-object/with-identity-claim.json',
  import.meta.url,
);
const v2Url = new URL(
  '../../../conformance/identity-claims/valid-v2.json',
  import.meta.url,
);
const unsupportedUrl = new URL(
  '../../../conformance/identity-claims/unsupported-rule.json',
  import.meta.url,
);
const mismatchUrl = new URL(
  '../../../fixtures/knowledge-object/invalid-identity-claim-mismatch.json',
  import.meta.url,
);

async function readJson(url) {
  return JSON.parse(await readFile(url, 'utf8'));
}

test('validates v1 and v2 claims through one facade', async () => {
  for (const url of [v1Url, v2Url]) {
    const report = validateKnowledgeObjectFull(await readJson(url));
    assert.equal(report.valid, true);
    assert.equal(report.identityStatus, IdentityValidationStatus.VALID);
  }
});

test('separates unsupported rules from mismatches', async () => {
  const report = validateKnowledgeObjectFull(await readJson(unsupportedUrl));

  assert.equal(report.valid, false);
  assert.equal(report.identityStatus, IdentityValidationStatus.UNSUPPORTED);
  assert.deepEqual(report.identityErrors, []);
  assert.equal(report.unsupportedIdentityRules.length, 1);
});

test('reports identity mismatches as invalid', async () => {
  const report = validateKnowledgeObjectFull(await readJson(mismatchUrl));

  assert.equal(report.valid, false);
  assert.equal(report.identityStatus, IdentityValidationStatus.INVALID);
  assert.ok(report.identityErrors.length > 0);
});
