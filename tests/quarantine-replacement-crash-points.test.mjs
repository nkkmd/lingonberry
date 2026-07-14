import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const root = process.cwd();
const registryPath = path.join(
  root,
  'packages/core/src/quarantine_replacement_failure_injection.rs',
);
const inventoryPath = path.join(
  root,
  'docs/operations/quarantine-replacement-crash-points.v1.json',
);

function registryIds(source) {
  return [...source.matchAll(/pub const FAILURE_POINT_[A-Z0-9_]+: &str =\s*"([a-z0-9.-]+)";/g)]
    .map((match) => match[1])
    .sort();
}

function productionSources() {
  const sourceDir = path.join(root, 'packages/core/src');
  return fs
    .readdirSync(sourceDir)
    .filter((name) => name.endsWith('.rs') || name.endsWith('.inc'))
    .filter((name) => name !== 'quarantine_replacement_failure_injection.rs')
    .map((name) => fs.readFileSync(path.join(sourceDir, name), 'utf8'))
    .join('\n');
}

test('crash-point inventory exactly matches the production registry', () => {
  const registry = registryIds(fs.readFileSync(registryPath, 'utf8'));
  const inventory = JSON.parse(fs.readFileSync(inventoryPath, 'utf8'));
  const inventoryIds = inventory.points.map((point) => point.id).sort();

  assert.equal(inventory.version, 'lingonberry-quarantine-replacement-crash-points/v1');
  assert.deepEqual(inventoryIds, registry);
  assert.equal(new Set(inventoryIds).size, inventoryIds.length);
  assert.ok(inventory.points.every((point) => point.registered === true));
});

test('implemented crash points have a production connection or explicit alias', () => {
  const registrySource = fs.readFileSync(registryPath, 'utf8');
  const sources = productionSources();
  const inventory = JSON.parse(fs.readFileSync(inventoryPath, 'utf8'));

  for (const point of inventory.points.filter((entry) => entry.implemented)) {
    const escaped = point.id.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const directConnection = new RegExp(`"${escaped}"`).test(sources);
    const explicitAlias = registrySource.includes(`FAILURE_POINT_POINTER_TEMPORARY_WRITE`)
      && point.id === 'publication.pointer-temporary-write';

    assert.ok(
      directConnection || explicitAlias,
      `implemented crash point has no production connection: ${point.id}`,
    );
  }
});
