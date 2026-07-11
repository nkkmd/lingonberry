import { createHash } from 'node:crypto';

import { sortKeys } from '../codecs/knowledge-object.mjs';

export const IDENTITY_KEY_RULE_VERSION_V2 = 'lb.identity.key.v2';
export const CANONICALIZATION_RULE_VERSION_V1 = 'lb.canonical.json.v1';

const SEMANTIC_FIELDS = [
  'type',
  'createdAt',
  'body',
  'contexts',
  'relations',
  'status',
  'lineage',
  'attachments',
  'labels',
];

export function identityKeyBasis(value) {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    return {};
  }

  const basis = {};
  for (const field of SEMANTIC_FIELDS) {
    if (field in value) {
      basis[field] = value[field];
    }
  }
  return sortKeys(basis);
}

export function deriveIdentityKeyV2(value) {
  const basis = identityKeyBasis(value);
  const canonicalJson = JSON.stringify(sortKeys(basis));
  const digest = createHash('sha256').update(canonicalJson, 'utf8').digest('hex');

  return `lb:key:${IDENTITY_KEY_RULE_VERSION_V2}:sha256:${digest}`;
}
