import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

import { validateIdentityClaimVersions } from '../validate-identity-claims.mjs';

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

async function readJson(url) {
  return JSON.parse(await readFile(url, 'utf8'));
}

test('accepts valid v1 and v2 identity claims', async () => {
  for (const url of [v1Url, v2Url]) {
    const value = await readJson(url);
    assert.deepEqual(validateIdentityClaimVersions(value), []);
  }
});

test('reports unsupported rules separately', async () => {
  const value = await readJson(unsupportedUrl);
  const errors = validateIdentityClaimVersions(value);

  assert.equal(errors.length, 1);
  assert.match(errors[0], /ruleVersion is unsupported/);
});

test('detects a mismatched v2 identity key', async () => {
  const value = await readJson(v2Url);
  value.identityClaims[0].identityKey =
    'lb:key:lb.identity.key.v2:sha256:0000000000000000000000000000000000000000000000000000000000000000';

  const errors = validateIdentityClaimVersions(value);
  assert.equal(errors.length, 1);
  assert.match(errors[0], /must match the derived identity key/);
});
