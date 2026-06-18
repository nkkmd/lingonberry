import { mkdir, readFile, appendFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';

const DEFAULT_STATE_DIR = resolve(process.cwd(), '.lingonberry');

async function ensureParent(pathname) {
  await mkdir(dirname(pathname), { recursive: true });
}

export function getStorePaths(baseDir = DEFAULT_STATE_DIR) {
  const stateDir = resolve(baseDir);
  return {
    stateDir,
    logPath: resolve(stateDir, 'knowledge-object-log.jsonl'),
    latestPath: resolve(stateDir, 'knowledge-object-latest.json'),
  };
}

export async function appendKnowledgeObject(object, baseDir = DEFAULT_STATE_DIR) {
  const { logPath, latestPath } = getStorePaths(baseDir);
  await ensureParent(logPath);
  const existing = await getKnowledgeObject(object.id, baseDir);
  if (existing) {
    const existingJson = JSON.stringify(existing);
    const incomingJson = JSON.stringify(object);
    if (existingJson !== incomingJson) {
      const error = new Error(`object already exists with different content: ${object.id}`);
      error.code = 'LB_OBJECT_CONFLICT';
      throw error;
    }
    return {
      storedAt: null,
      object: existing,
      duplicate: true,
    };
  }
  const record = {
    storedAt: new Date().toISOString(),
    object,
  };
  await appendFile(logPath, `${JSON.stringify(record)}\n`, 'utf8');
  await writeFile(latestPath, `${JSON.stringify({ id: object.id, storedAt: record.storedAt }, null, 2)}\n`, 'utf8');
  return record;
}

export async function getKnowledgeObject(id, baseDir = DEFAULT_STATE_DIR) {
  const record = await getKnowledgeObjectRecord(id, baseDir);
  return record ? record.object : null;
}

export async function getKnowledgeObjectRecord(id, baseDir = DEFAULT_STATE_DIR) {
  const { logPath } = getStorePaths(baseDir);
  let raw;
  try {
    raw = await readFile(logPath, 'utf8');
  } catch (error) {
    if (error && error.code === 'ENOENT') {
      return null;
    }
    throw error;
  }

  const lines = raw.split('\n').filter((line) => line.trim().length > 0);
  for (let index = lines.length - 1; index >= 0; index -= 1) {
    const entry = JSON.parse(lines[index]);
    if (entry?.object?.id === id) {
      return entry;
    }
  }

  return null;
}

export async function listStoredIds(baseDir = DEFAULT_STATE_DIR) {
  const { logPath } = getStorePaths(baseDir);
  let raw;
  try {
    raw = await readFile(logPath, 'utf8');
  } catch (error) {
    if (error && error.code === 'ENOENT') {
      return [];
    }
    throw error;
  }

  const ids = [];
  const seen = new Set();
  for (const line of raw.split('\n')) {
    if (!line.trim()) {
      continue;
    }
    const entry = JSON.parse(line);
    const id = entry?.object?.id;
    if (typeof id === 'string' && !seen.has(id)) {
      seen.add(id);
      ids.push(id);
    }
  }
  return ids;
}
