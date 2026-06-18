export function toCanonicalView(object, metadata = {}) {
  return {
    canonicalId: object.id,
    object,
    ...metadata,
  };
}
