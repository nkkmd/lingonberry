#!/usr/bin/env node

import {
  generateKeyPairSync,
  sign,
} from 'node:crypto';
import { pathToFileURL } from 'node:url';

export const CANONICALIZATION_RULE_VERSION = 'lb.canonical.json.v1';
export const SIGNATURE_RULE_VERSION = 'lb.http.publish.signature.v1';

export function sortKeys(value) {
  if (Array.isArray(value)) return value.map(sortKeys);
  if (value !== null && typeof value === 'object') {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, sortKeys(value[key])]),
    );
  }
  return value;
}

export function canonicalJson(value) {
  return JSON.stringify(sortKeys(value));
}

function rawPublicKeyHex(publicKey) {
  const der = publicKey.export({ format: 'der', type: 'spki' });
  return der.subarray(der.length - 32).toString('hex');
}

export function createMinimalKnowledgeObject({
  id = 'lb:obj:js-producer-0001',
  createdAt = '2026-07-20T00:00:00Z',
  text = 'Can a non-Rust producer publish this object?',
  language = 'en',
} = {}) {
  return {
    id,
    schemaVersion: '0.1.0',
    type: 'inquiry',
    createdAt,
    body: { text, language },
    provenance: {
      sources: [
        {
          protocol: 'lingonberry-js-conformance',
          sourceId: `producer:${id}`,
          observedAt: createdAt,
        },
      ],
    },
    rawRef: {
      protocol: 'lingonberry-js-conformance',
      sourceId: `producer:${id}`,
    },
  };
}

export function createSignedPublishRequest(object, keyPair = generateKeyPairSync('ed25519')) {
  const publicKey = rawPublicKeyHex(keyPair.publicKey);
  const unsigned = {
    object,
    publisher: { publicKey },
  };
  const target = canonicalJson(unsigned);
  const signature = sign(null, Buffer.from(target, 'utf8'), keyPair.privateKey).toString('hex');

  return {
    request: {
      object,
      publisher: { publicKey, signature },
    },
    target,
    metadata: {
      canonicalizationRuleVersion: CANONICALIZATION_RULE_VERSION,
      signatureRuleVersion: SIGNATURE_RULE_VERSION,
    },
  };
}

function parseArgs(argv) {
  const options = {};
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--id') options.id = argv[++index];
    else if (argument === '--created-at') options.createdAt = argv[++index];
    else if (argument === '--text') options.text = argv[++index];
    else if (argument === '--language') options.language = argv[++index];
    else throw new Error(`unknown argument: ${argument}`);
  }
  return options;
}

export function main(argv = process.argv.slice(2)) {
  const object = createMinimalKnowledgeObject(parseArgs(argv));
  const produced = createSignedPublishRequest(object);
  process.stdout.write(`${canonicalJson(produced.request)}\n`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
