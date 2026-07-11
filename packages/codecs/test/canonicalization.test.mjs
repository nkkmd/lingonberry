import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

import { sortKeys } from '../knowledge-object.mjs';

const inputUrl = new URL(
  '../../../conformance/canonicalization/object-key-order.input.json',
  import.meta.url,
);
const expectedUrl = new URL(
  '../../../conformance/canonicalization/object-key-order.expected.json',
  import.meta.url,
);

async function loadFixture() {
  const [inputRaw, expected] = await Promise.all([
    readFile(inputUrl, 'utf8'),
    readFile(expectedUrl, 'utf8'),
  ]);
  return {
    input: JSON.parse(inputRaw),
    expected,
  };
}

function canonicalize(value) {
  return JSON.stringify(sortKeys(value));
}

test('canonicalization matches shared fixture', async () => {
  const { input, expected } = await loadFixture();
  const canonical = canonicalize(input);

  assert.deepEqual(Buffer.from(canonical, 'utf8'), Buffer.from(expected, 'utf8'));
});

test('canonicalization is idempotent', async () => {
  const { input } = await loadFixture();
  const first = canonicalize(input);
  const second = canonicalize(JSON.parse(first));

  assert.equal(first, second);
});
