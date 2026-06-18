export function toCanonicalView(object, metadata = {}) {
  return {
    canonicalId: object.id,
    object,
    ...metadata,
  };
}

export function toCanonicalGetView(record, metadata = {}) {
  return {
    canonicalId: record.canonicalId ?? record.canonical_id,
    carrierIdentity: record.carrierIdentity ?? record.carrier_identity,
    storedAt: record.storedAt ?? record.stored_at,
    object: record.object,
    ...metadata,
  };
}

export function toCanonicalListView(records, metadata = {}) {
  return {
    count: records.length,
    objects: records.map((record) => toCanonicalGetView(record)),
    ...metadata,
  };
}

export function toCanonicalGraphView(fragment, metadata = {}) {
  return {
    canonicalId: fragment.canonicalId ?? fragment.canonical_id,
    relations: fragment.relations ?? {
      outbound: fragment.relationsOutbound ?? fragment.relations_outbound ?? [],
      inbound: fragment.relationsInbound ?? fragment.relations_inbound ?? [],
      relatedIds: fragment.relatedIds ?? fragment.related_ids ?? [],
    },
    lineage: fragment.lineage ?? {
      outbound: fragment.lineageOutbound ?? fragment.lineage_outbound ?? [],
      inbound: fragment.lineageInbound ?? fragment.lineage_inbound ?? [],
      relatedIds: fragment.relatedIds ?? fragment.related_ids ?? [],
    },
    provenance: fragment.provenance ?? {
      sources: fragment.provenanceSources ?? fragment.provenance_sources ?? [],
    },
    ...metadata,
  };
}

export function toCanonicalSearchView(query, records, metadata = {}) {
  return {
    query,
    count: records.length,
    objects: records.map((record) => toCanonicalGetView(record)),
    ...metadata,
  };
}
