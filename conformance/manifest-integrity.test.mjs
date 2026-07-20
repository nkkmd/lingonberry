import assert from 'node:assert/strict';
import { access, readFile } from 'node:fs/promises';
import test from 'node:test';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const manifest = JSON.parse(await readFile(resolve(root, 'manifest.v1.json'), 'utf8'));
const allowedSuites = new Set(['producer', 'consumer', 'internal']);
const fileFields = ['input', 'expected', 'target', 'alternateInput'];

test('manifest has unique stable case identifiers', () => {
  assert.equal(manifest.manifestVersion, 'lb.conformance.manifest.v1');
  assert.ok(Array.isArray(manifest.cases));
  const ids = manifest.cases.map((testCase) => testCase.id);
  assert.equal(new Set(ids).size, ids.length);
  for (const id of ids) {
    assert.match(id, /^[a-z0-9][a-z0-9.-]+$/);
  }
});

test('manifest cases declare suites, kinds, and rule versions', () => {
  for (const testCase of manifest.cases) {
    assert.ok(allowedSuites.has(testCase.suite), `${testCase.id}: invalid suite`);
    assert.equal(typeof testCase.kind, 'string', `${testCase.id}: missing kind`);
    assert.equal(typeof testCase.ruleVersion, 'string', `${testCase.id}: missing ruleVersion`);
  }
});

test('all referenced fixture files exist', async () => {
  for (const testCase of manifest.cases) {
    for (const field of fileFields) {
      if (testCase[field]) {
        await access(resolve(root, testCase[field]));
      }
    }
  }
});
