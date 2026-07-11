import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

import { deriveIdentityKeyV2 } from '../identity-key.mjs';

const inputUrl = new URL(
  '../../../conformance/identity-key-v2/minimal-object.input.json',
  import.meta.url,
);
const alternateUrl = new URL(
  '../../../conformance/identity-key-v2/minimal-object-alternate-origin.input.json',
  import.meta.url,
);
const expectedUrl = new URL(
  '../../../conformance/identity-key-v2/minimal-object.expected.txt',
  import.meta.url,
);

async function readJson(url) {
  return JSON.parse(await readFile(url, 'utf8'));
}

test('identity key v2 matches the shared fixture', async () => {
  const [input, expected] = await Promise.all([
    readJson(inputUrl),
    readFile(expectedUrl, 'utf8'),
  ]);

  assert.equal(deriveIdentityKeyV2(input), expected);
});

test('identity key v2 excludes transport and provenance fields', async () => {
  const [first, second] = await Promise.all([
    readJson(inputUrl),
    readJson(alternateUrl),
  ]);

  assert.equal(deriveIdentityKeyV2(first), deriveIdentityKeyV2(second));
});
