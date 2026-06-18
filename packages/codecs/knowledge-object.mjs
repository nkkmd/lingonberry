import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

function isObject(value) {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function validateDateTime(value, path, errors) {
  if (typeof value !== 'string' || Number.isNaN(Date.parse(value))) {
    errors.push(`${path} must be a valid date-time string`);
  }
}

function validateLanguageTag(value, path, errors) {
  if (typeof value !== 'string' || !/^[A-Za-z]{2,8}(-[A-Za-z0-9]{1,8})*$/.test(value)) {
    errors.push(`${path} must be a BCP47-style language tag`);
  }
}

export async function readJsonFile(pathname) {
  const raw = await readFile(resolve(process.cwd(), pathname), 'utf8');
  return JSON.parse(raw);
}

export function detectShape(value) {
  if (isObject(value) && isObject(value.object) && isObject(value.publisher)) {
    return 'publish-request';
  }
  return 'knowledge-object';
}

export function validateKnowledgeObject(value) {
  const errors = [];

  if (!isObject(value)) {
    return ['knowledge object must be an object'];
  }

  const required = ['id', 'schemaVersion', 'type', 'createdAt', 'body', 'provenance', 'rawRef'];
  for (const key of required) {
    if (!(key in value)) {
      errors.push(`missing required field: ${key}`);
    }
  }

  if (typeof value.id !== 'string' || !/^lb:obj:[^\s]+$/.test(value.id)) {
    errors.push('id must match ^lb:obj:[^\\s]+$');
  }

  if (value.schemaVersion !== '0.1.0') {
    errors.push('schemaVersion must be 0.1.0');
  }

  if (!['inquiry', 'observation', 'claim', 'evidence', 'annotation', 'synthesis', 'translation', 'reference', 'concept'].includes(value.type)) {
    errors.push('type must be one of the supported knowledge object types');
  }

  validateDateTime(value.createdAt, 'createdAt', errors);

  if (!isObject(value.body)) {
    errors.push('body must be an object');
  } else {
    if (typeof value.body.text !== 'string' || value.body.text.length < 1) {
      errors.push('body.text must be a non-empty string');
    }
    validateLanguageTag(value.body.language, 'body.language', errors);
    const bodyKeys = Object.keys(value.body);
    if (bodyKeys.some((key) => !['text', 'language'].includes(key))) {
      errors.push('body must not contain additional properties');
    }
  }

  if (!isObject(value.provenance)) {
    errors.push('provenance must be an object');
  } else {
    if (!Array.isArray(value.provenance.sources) || value.provenance.sources.length < 1) {
      errors.push('provenance.sources must be a non-empty array');
    } else {
      for (const [index, source] of value.provenance.sources.entries()) {
        if (!isObject(source)) {
          errors.push(`provenance.sources[${index}] must be an object`);
          continue;
        }
        if (typeof source.protocol !== 'string' || source.protocol.length < 1) {
          errors.push(`provenance.sources[${index}].protocol must be a non-empty string`);
        }
        if (typeof source.sourceId !== 'string' || source.sourceId.length < 1) {
          errors.push(`provenance.sources[${index}].sourceId must be a non-empty string`);
        }
        if ('authorId' in source && (typeof source.authorId !== 'string' || source.authorId.length < 1)) {
          errors.push(`provenance.sources[${index}].authorId must be a non-empty string when present`);
        }
        if ('observedAt' in source) {
          validateDateTime(source.observedAt, `provenance.sources[${index}].observedAt`, errors);
        }
        const allowed = ['protocol', 'sourceId', 'authorId', 'observedAt'];
        if (Object.keys(source).some((key) => !allowed.includes(key))) {
          errors.push(`provenance.sources[${index}] must not contain additional properties`);
        }
      }
    }
    const allowed = ['sources'];
    if (Object.keys(value.provenance).some((key) => !allowed.includes(key))) {
      errors.push('provenance must not contain additional properties');
    }
  }

  if (!isObject(value.rawRef)) {
    errors.push('rawRef must be an object');
  } else {
    if (typeof value.rawRef.protocol !== 'string' || value.rawRef.protocol.length < 1) {
      errors.push('rawRef.protocol must be a non-empty string');
    }
    if (typeof value.rawRef.sourceId !== 'string' || value.rawRef.sourceId.length < 1) {
      errors.push('rawRef.sourceId must be a non-empty string');
    }
    if ('locator' in value.rawRef && (typeof value.rawRef.locator !== 'string' || value.rawRef.locator.length < 1)) {
      errors.push('rawRef.locator must be a non-empty string when present');
    }
    if ('payloadHash' in value.rawRef && (typeof value.rawRef.payloadHash !== 'string' || value.rawRef.payloadHash.length < 1)) {
      errors.push('rawRef.payloadHash must be a non-empty string when present');
    }
    const allowed = ['protocol', 'sourceId', 'locator', 'payloadHash'];
    if (Object.keys(value.rawRef).some((key) => !allowed.includes(key))) {
      errors.push('rawRef must not contain additional properties');
    }
  }

  const allowedRoot = new Set([
    'id',
    'schemaVersion',
    'type',
    'createdAt',
    'body',
    'contexts',
    'relations',
    'status',
    'lineage',
    'provenance',
    'rawRef',
    'identityClaims',
    'attachments',
    'labels',
    'meta',
  ]);
  for (const key of Object.keys(value)) {
    if (!allowedRoot.has(key)) {
      errors.push(`unknown root field: ${key}`);
    }
  }

  return errors;
}

export function validatePublishRequest(value) {
  const errors = [];

  if (!isObject(value)) {
    return ['publish request must be an object'];
  }

  if (!isObject(value.object)) {
    errors.push('object must be an object');
  } else {
    errors.push(...validateKnowledgeObject(value.object).map((error) => `object.${error}`));
  }

  if (!isObject(value.publisher)) {
    errors.push('publisher must be an object');
  } else {
    if (typeof value.publisher.publicKey !== 'string' || !/^[0-9a-f]{64}$/.test(value.publisher.publicKey)) {
      errors.push('publisher.publicKey must be a 64-character lowercase hex string');
    }
    if (typeof value.publisher.signature !== 'string' || value.publisher.signature.length < 1) {
      errors.push('publisher.signature must be a non-empty string');
    }
    const allowed = ['publicKey', 'signature'];
    if (Object.keys(value.publisher).some((key) => !allowed.includes(key))) {
      errors.push('publisher must not contain additional properties');
    }
  }

  const allowedRoot = new Set(['object', 'publisher']);
  for (const key of Object.keys(value)) {
    if (!allowedRoot.has(key)) {
      errors.push(`unknown root field: ${key}`);
    }
  }

  return errors;
}

export function sortKeys(value) {
  if (Array.isArray(value)) {
    return value.map(sortKeys);
  }
  if (!isObject(value)) {
    return value;
  }
  return Object.fromEntries(Object.keys(value).sort().map((key) => [key, sortKeys(value[key])]));
}

export function normalizeKnowledgeObject(object) {
  return sortKeys(object);
}

export function finalizeKnowledgeObject(object) {
  const normalized = normalizeKnowledgeObject(object);
  return {
    canonicalId: normalized.id,
    object: normalized,
  };
}
