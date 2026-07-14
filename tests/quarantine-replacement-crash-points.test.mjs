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

function registryEntries(source) {
  return [...source.matchAll(/pub const (FAILURE_POINT_[A-Z0-9_]+): &str =\s*"([a-z0-9.-]+)";/g)]
    .map((match) => ({ symbol: match[1], id: match[2] }));
}

function aliasSymbols(source) {
  const match = source.match(
    /pub const QUARANTINE_REPLACEMENT_FAILURE_POINT_ALIASES:[\s\S]*?= &\[([\s\S]*?)\n\];/,
  );
  assert.ok(match, 'failure-point alias table is missing');
  return new Set(
    [...match[1].matchAll(/FAILURE_POINT_[A-Z0-9_]+/g)].map((entry) => entry[0]),
  );
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
  const registry = registryEntries(fs.readFileSync(registryPath, 'utf8'))
    .map((entry) => entry.id)
    .sort();
  const inventory = JSON.parse(fs.readFileSync(inventoryPath, 'utf8'));
  const inventoryIds = inventory.points.map((point) => point.id).sort();

  assert.equal(inventory.version, 'lingonberry-quarantine-replacement-crash-points/v1');
  assert.deepEqual(inventoryIds, registry);
  assert.equal(new Set(inventoryIds).size, inventoryIds.length);
  assert.ok(inventory.points.every((point) => point.registered === true));
});

test('implemented crash points have a production connection or explicit alias', () => {
  const registrySource = fs.readFileSync(registryPath, 'utf8');
  const registry = new Map(registryEntries(registrySource).map((entry) => [entry.id, entry.symbol]));
  const aliases = aliasSymbols(registrySource);
  const sources = productionSources();
  const inventory = JSON.parse(fs.readFileSync(inventoryPath, 'utf8'));

  for (const point of inventory.points.filter((entry) => entry.implemented)) {
    const symbol = registry.get(point.id);
    assert.ok(symbol, `implemented crash point is missing from registry: ${point.id}`);

    const directConnection = sources.includes(symbol);
    const explicitAlias = aliases.has(symbol);

    assert.ok(
      directConnection || explicitAlias,
      `implemented crash point has no production connection: ${point.id}`,
    );
  }
});
