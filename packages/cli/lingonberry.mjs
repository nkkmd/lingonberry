#!/usr/bin/env node
import {
  detectShape,
  finalizeKnowledgeObject,
  readJsonFile,
  validateKnowledgeObject,
  validatePublishRequest,
} from '../codecs/knowledge-object.mjs';
import {
  appendKnowledgeObject,
  getKnowledgeObjectRecord,
  listStoredIds,
} from '../core/object-store.mjs';
import { toCanonicalView } from '../api/canonical-view.mjs';

function fail(message, details = []) {
  const suffix = details.length > 0 ? `\n- ${details.join('\n- ')}` : '';
  throw new Error(`${message}${suffix}`);
}

async function handleValidate(pathname) {
  const value = await readJsonFile(pathname);
  const errors = detectShape(value) === 'publish-request'
    ? validatePublishRequest(value)
    : validateKnowledgeObject(value);
  if (errors.length > 0) {
    fail('validation failed', errors);
  }
  console.log(JSON.stringify({ ok: true }, null, 2));
}

async function handlePublish(pathname) {
  const value = await readJsonFile(pathname);
  const errors = validatePublishRequest(value);
  if (errors.length > 0) {
    fail('validation failed', errors);
  }
  const finalized = finalizeKnowledgeObject(value.object);
  const stored = await appendKnowledgeObject(finalized.object);
  console.log(JSON.stringify({
    ...finalized,
    storedAt: stored.storedAt,
    duplicate: Boolean(stored.duplicate),
  }, null, 2));
}

async function handleGet(id) {
  const record = await getKnowledgeObjectRecord(id);
  if (!record) {
    fail(`object not found: ${id}`);
  }
  console.log(JSON.stringify(toCanonicalView(record.object, { storedAt: record.storedAt }), null, 2));
}

async function handleList() {
  const ids = await listStoredIds();
  console.log(JSON.stringify({ ids }, null, 2));
}

async function main(argv) {
  const [command, pathname] = argv;
  if (!command) {
    fail('usage: lingonberry <validate|publish|get|list> <json-file|id>');
  }

  if (command === 'validate') {
    if (!pathname) {
      fail('usage: lingonberry validate <json-file>');
    }
    await handleValidate(pathname);
    return;
  }

  if (command === 'publish') {
    if (!pathname) {
      fail('usage: lingonberry publish <json-file>');
    }
    await handlePublish(pathname);
    return;
  }

  if (command === 'get') {
    if (!pathname) {
      fail('usage: lingonberry get <canonical-id>');
    }
    await handleGet(pathname);
    return;
  }

  if (command === 'list') {
    await handleList();
    return;
  }

  fail(`unknown command: ${command}`);
}

main(process.argv.slice(2)).catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
