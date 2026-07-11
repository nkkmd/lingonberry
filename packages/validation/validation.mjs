import { validateKnowledgeObject } from '../codecs/knowledge-object.mjs';
import { validateIdentityClaimVersions } from '../identity/validate-identity-claims.mjs';

export const IdentityValidationStatus = Object.freeze({
  VALID: 'valid',
  INVALID: 'invalid',
  UNSUPPORTED: 'unsupported',
  NOT_PRESENT: 'not-present',
});

function isLegacyIdentityRuleError(error) {
  return (
    error.startsWith('identityClaims[') &&
    (error.includes('.ruleVersion must be lb.identity.key.v1') ||
      error.includes('.identityKey must match the derived identity key'))
  );
}

export function validateKnowledgeObjectFull(value) {
  const schemaErrors = validateKnowledgeObject(value).filter(
    (error) => !isLegacyIdentityRuleError(error),
  );
  const identityResults = validateIdentityClaimVersions(value);
  const identityErrors = [];
  const unsupportedIdentityRules = [];

  for (const error of identityResults) {
    if (error.includes('.ruleVersion is unsupported:')) {
      unsupportedIdentityRules.push(error);
    } else {
      identityErrors.push(error);
    }
  }

  const hasIdentityClaims =
    typeof value === 'object' &&
    value !== null &&
    !Array.isArray(value) &&
    Array.isArray(value.identityClaims) &&
    value.identityClaims.length > 0;

  let identityStatus = IdentityValidationStatus.NOT_PRESENT;
  if (unsupportedIdentityRules.length > 0) {
    identityStatus = IdentityValidationStatus.UNSUPPORTED;
  } else if (identityErrors.length > 0) {
    identityStatus = IdentityValidationStatus.INVALID;
  } else if (hasIdentityClaims) {
    identityStatus = IdentityValidationStatus.VALID;
  }

  return {
    schemaErrors,
    identityErrors,
    unsupportedIdentityRules,
    identityStatus,
    valid:
      schemaErrors.length === 0 &&
      identityErrors.length === 0 &&
      unsupportedIdentityRules.length === 0,
  };
}
