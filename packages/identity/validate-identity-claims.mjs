import { deriveIdentityKey } from '../codecs/knowledge-object.mjs';
import { deriveIdentityKeyV2, IDENTITY_KEY_RULE_VERSION_V2 } from './identity-key.mjs';

export const IDENTITY_KEY_RULE_VERSION_V1 = 'lb.identity.key.v1';

export function deriveIdentityKeyForRule(value, ruleVersion) {
  switch (ruleVersion) {
    case IDENTITY_KEY_RULE_VERSION_V1:
      return { supported: true, identityKey: deriveIdentityKey(value) };
    case IDENTITY_KEY_RULE_VERSION_V2:
      return { supported: true, identityKey: deriveIdentityKeyV2(value) };
    default:
      return { supported: false, identityKey: null };
  }
}

export function validateIdentityClaimVersions(value) {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    return ['knowledge object must be an object'];
  }
  if (!('identityClaims' in value)) {
    return [];
  }
  if (!Array.isArray(value.identityClaims)) {
    return ['identityClaims must be an array'];
  }

  const errors = [];
  for (const [index, claim] of value.identityClaims.entries()) {
    if (typeof claim !== 'object' || claim === null || Array.isArray(claim)) {
      errors.push(`identityClaims[${index}] must be an object`);
      continue;
    }
    if (typeof claim.ruleVersion !== 'string' || claim.ruleVersion.length === 0) {
      errors.push(`identityClaims[${index}].ruleVersion must be a non-empty string`);
      continue;
    }

    const result = deriveIdentityKeyForRule(value, claim.ruleVersion);
    if (!result.supported) {
      errors.push(`identityClaims[${index}].ruleVersion is unsupported: ${claim.ruleVersion}`);
      continue;
    }
    if (typeof claim.identityKey !== 'string') {
      errors.push(`identityClaims[${index}].identityKey must be a string`);
    } else if (claim.identityKey !== result.identityKey) {
      errors.push(`identityClaims[${index}].identityKey must match the derived identity key for ${claim.ruleVersion}`);
    }
    if (typeof claim.canonicalId === 'string' && typeof value.id === 'string' && claim.canonicalId !== value.id) {
      errors.push(`identityClaims[${index}].canonicalId must match the enclosing object id`);
    }
  }

  return errors;
}
