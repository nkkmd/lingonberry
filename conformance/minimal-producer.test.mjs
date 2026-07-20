import assert from 'node:assert/strict';
import { createPublicKey, verify } from 'node:crypto';
import test from 'node:test';

import {
  canonicalJson,
  createMinimalKnowledgeObject,
  createSignedPublishRequest,
} from './minimal-producer.mjs';

function publicKeyFromRawHex(publicKeyHex) {
  const prefix = Buffer.from('302a300506032b6570032100', 'hex');
  return createPublicKey({
    key: Buffer.concat([prefix, Buffer.from(publicKeyHex, 'hex')]),
    format: 'der',
    type: 'spki',
  });
}

test('standalone producer emits a valid signed request', () => {
  const object = createMinimalKnowledgeObject({
    id: 'lb:obj:js-producer-test',
    createdAt: '2026-07-20T00:00:00Z',
  });
  const produced = createSignedPublishRequest(object);

  assert.equal(produced.request.object.id, 'lb:obj:js-producer-test');
  assert.match(produced.request.publisher.publicKey, /^[0-9a-f]{64}$/);
  assert.match(produced.request.publisher.signature, /^[0-9a-f]{128}$/);
  assert.equal(
    produced.target,
    canonicalJson({
      object: produced.request.object,
      publisher: { publicKey: produced.request.publisher.publicKey },
    }),
  );
  assert.equal(
    verify(
      null,
      Buffer.from(produced.target, 'utf8'),
      publicKeyFromRawHex(produced.request.publisher.publicKey),
      Buffer.from(produced.request.publisher.signature, 'hex'),
    ),
    true,
  );
});
