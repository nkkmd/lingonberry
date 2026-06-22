use lingonberry_core::{StorageBackend, StoreError, StoredCatalogRecord};
use lingonberry_protocol::JsonValue;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct IndexedRecord {
    pub canonical_id: String,
    pub carrier_identity: String,
    pub stored_at: String,
    pub object_type: Option<String>,
    pub object: JsonValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationEdge {
    pub canonical_id: String,
    pub source: String,
    pub target: String,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineageEdge {
    pub canonical_id: String,
    pub edge_type: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProvenanceSourceEntry {
    pub canonical_id: String,
    pub protocol: String,
    pub source_id: String,
    pub author_id: Option<String>,
    pub observed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProvenanceGraphEntry {
    pub canonical_id: String,
    pub protocol: String,
    pub source_id: String,
    pub author_id: Option<String>,
    pub observed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectGraphFragment {
    pub canonical_id: String,
    pub relations_outbound: Vec<RelationEdge>,
    pub relations_inbound: Vec<RelationEdge>,
    pub lineage_outbound: Vec<LineageEdge>,
    pub lineage_inbound: Vec<LineageEdge>,
    pub provenance_sources: Vec<ProvenanceSourceEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationGraphFragment {
    pub canonical_id: String,
    pub outbound: Vec<RelationEdge>,
    pub inbound: Vec<RelationEdge>,
    pub related_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineageGraphFragment {
    pub canonical_id: String,
    pub outbound: Vec<LineageEdge>,
    pub inbound: Vec<LineageEdge>,
    pub related_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProvenanceGraphFragment {
    pub protocol: String,
    pub source_id: String,
    pub canonical_ids: Vec<String>,
    pub entries: Vec<ProvenanceGraphEntry>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct IndexSnapshot {
    records: BTreeMap<String, IndexedRecord>,
    type_index: BTreeMap<String, Vec<String>>,
    relation_edges: Vec<RelationEdge>,
    lineage_edges: Vec<LineageEdge>,
    provenance_index: BTreeMap<String, Vec<String>>,
}

impl IndexSnapshot {
    pub fn from_records(records: impl IntoIterator<Item = StoredCatalogRecord>) -> Self {
        let mut snapshot = Self::default();
        for record in records {
            snapshot.ingest_record(record);
        }
        snapshot
    }

    pub fn from_backend(backend: &impl StorageBackend) -> Result<Self, StoreError> {
        let records = backend.subscribe(None)?;
        Ok(Self::from_records(records))
    }

    pub fn rebuild_from_backend(backend: &impl StorageBackend) -> Result<Self, StoreError> {
        Self::from_backend(backend)
    }

    pub fn record(&self, canonical_id: &str) -> Option<&IndexedRecord> {
        self.records.get(canonical_id)
    }

    pub fn list_types(&self) -> Vec<String> {
        self.type_index.keys().cloned().collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn ids_for_type(&self, object_type: &str) -> Vec<String> {
        self.type_index
            .get(object_type)
            .cloned()
            .unwrap_or_default()
    }

    pub fn records_for_type(&self, object_type: &str) -> Vec<&IndexedRecord> {
        self.ids_for_type(object_type)
            .into_iter()
            .filter_map(|canonical_id| self.records.get(&canonical_id))
            .collect()
    }

    pub fn relation_edges(&self) -> &[RelationEdge] {
        &self.relation_edges
    }

    pub fn relation_graph(&self, canonical_id: &str) -> Option<RelationGraphFragment> {
        self.records.get(canonical_id)?;
        let outbound = self
            .relation_edges
            .iter()
            .filter(|edge| edge.canonical_id == canonical_id)
            .cloned()
            .collect::<Vec<_>>();
        let inbound = self
            .relation_edges
            .iter()
            .filter(|edge| edge.target == canonical_id && edge.canonical_id != canonical_id)
            .cloned()
            .collect::<Vec<_>>();
        let mut related_ids = Vec::new();
        for edge in outbound.iter().chain(inbound.iter()) {
            if edge.source != canonical_id {
                push_unique(&mut related_ids, edge.source.clone());
            }
            if edge.target != canonical_id {
                push_unique(&mut related_ids, edge.target.clone());
            }
        }
        Some(RelationGraphFragment {
            canonical_id: canonical_id.to_string(),
            outbound,
            inbound,
            related_ids,
        })
    }

    pub fn lineage_edges(&self) -> &[LineageEdge] {
        &self.lineage_edges
    }

    pub fn lineage_graph(&self, canonical_id: &str) -> Option<LineageGraphFragment> {
        self.records.get(canonical_id)?;
        let outbound = self
            .lineage_edges
            .iter()
            .filter(|edge| edge.canonical_id == canonical_id)
            .cloned()
            .collect::<Vec<_>>();
        let inbound = self
            .lineage_edges
            .iter()
            .filter(|edge| edge.target == canonical_id && edge.canonical_id != canonical_id)
            .cloned()
            .collect::<Vec<_>>();
        let mut related_ids = Vec::new();
        for edge in outbound.iter().chain(inbound.iter()) {
            if edge.target != canonical_id {
                push_unique(&mut related_ids, edge.target.clone());
            }
            if edge.canonical_id != canonical_id {
                push_unique(&mut related_ids, edge.canonical_id.clone());
            }
        }
        Some(LineageGraphFragment {
            canonical_id: canonical_id.to_string(),
            outbound,
            inbound,
            related_ids,
        })
    }

    pub fn provenance_ids_for_source(&self, protocol: &str, source_id: &str) -> Vec<String> {
        let key = provenance_key(protocol, source_id);
        self.provenance_index.get(&key).cloned().unwrap_or_default()
    }

    pub fn provenance_source_count(&self) -> usize {
        self.provenance_index.len()
    }

    pub fn provenance_graph(
        &self,
        protocol: &str,
        source_id: &str,
    ) -> Option<ProvenanceGraphFragment> {
        let canonical_ids = self.provenance_ids_for_source(protocol, source_id);
        if canonical_ids.is_empty() {
            return None;
        }

        let mut entries = Vec::new();
        for canonical_id in &canonical_ids {
            let Some(record) = self.records.get(canonical_id) else {
                continue;
            };
            for source in provenance_sources(&record.object).unwrap_or_default() {
                if source.protocol == protocol && source.source_id == source_id {
                    entries.push(ProvenanceGraphEntry {
                        canonical_id: canonical_id.clone(),
                        protocol: source.protocol,
                        source_id: source.source_id,
                        author_id: source.author_id,
                        observed_at: source.observed_at,
                    });
                }
            }
        }

        Some(ProvenanceGraphFragment {
            protocol: protocol.to_string(),
            source_id: source_id.to_string(),
            canonical_ids,
            entries,
        })
    }

    pub fn graph_fragment(&self, canonical_id: &str) -> Option<ObjectGraphFragment> {
        let _record = self.records.get(canonical_id)?;
        let relation_graph = self.relation_graph(canonical_id)?;
        let lineage_graph = self.lineage_graph(canonical_id)?;
        let lineage_outbound = lineage_graph.outbound;
        let lineage_inbound = lineage_graph.inbound;
        let provenance_sources = self.record_provenance_sources(canonical_id);
        Some(ObjectGraphFragment {
            canonical_id: canonical_id.to_string(),
            relations_outbound: relation_graph.outbound,
            relations_inbound: relation_graph.inbound,
            lineage_outbound,
            lineage_inbound,
            provenance_sources,
        })
    }

    fn record_provenance_sources(&self, canonical_id: &str) -> Vec<ProvenanceSourceEntry> {
        self.records
            .get(canonical_id)
            .and_then(|record| provenance_sources(&record.object).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|entry| ProvenanceSourceEntry {
                canonical_id: canonical_id.to_string(),
                ..entry
            })
            .collect()
    }

    fn ingest_record(&mut self, record: StoredCatalogRecord) {
        let canonical_id = record.canonical_id.clone();
        let object_type = object_type(&record.object).map(ToOwned::to_owned);
        let indexed_record = IndexedRecord {
            canonical_id: canonical_id.clone(),
            carrier_identity: record.carrier_identity,
            stored_at: record.stored_at,
            object_type: object_type.clone(),
            object: record.object,
        };

        if let Some(object_type) = object_type {
            push_unique(
                &mut self.type_index.entry(object_type).or_default(),
                canonical_id.clone(),
            );
        }

        for edge in relation_edges(&indexed_record.object, &canonical_id) {
            self.relation_edges.push(edge);
        }
        for edge in lineage_edges(&indexed_record.object, &canonical_id) {
            self.lineage_edges.push(edge);
        }
        for provenance_source in provenance_sources(&indexed_record.object).unwrap_or_default() {
            push_unique(
                self.provenance_index
                    .entry(provenance_key(
                        &provenance_source.protocol,
                        &provenance_source.source_id,
                    ))
                    .or_default(),
                canonical_id.clone(),
            );
        }

        self.records.insert(canonical_id, indexed_record);
    }
}

fn object_type(value: &JsonValue) -> Option<&str> {
    object_map(value).and_then(|map| string_field(map, "type"))
}

fn relation_edges(value: &JsonValue, canonical_id: &str) -> Vec<RelationEdge> {
    let Some(entries) = object_map(value).and_then(|map| array_field(map, "relations")) else {
        return Vec::new();
    };

    entries
        .iter()
        .filter_map(|entry| {
            let map = object_map(entry)?;
            let source = string_field(map, "source")?.to_string();
            let target = string_field(map, "target")?.to_string();
            let kind = string_field(map, "kind").map(ToOwned::to_owned);
            Some(RelationEdge {
                canonical_id: canonical_id.to_string(),
                source,
                target,
                kind,
            })
        })
        .collect()
}

fn lineage_edges(value: &JsonValue, canonical_id: &str) -> Vec<LineageEdge> {
    let Some(entries) = object_map(value).and_then(|map| array_field(map, "lineage")) else {
        return Vec::new();
    };

    entries
        .iter()
        .filter_map(|entry| {
            let map = object_map(entry)?;
            let edge_type = string_field(map, "type")?.to_string();
            let target = string_field(map, "target")?.to_string();
            Some(LineageEdge {
                canonical_id: canonical_id.to_string(),
                edge_type,
                target,
            })
        })
        .collect()
}

fn provenance_sources(value: &JsonValue) -> Result<Vec<ProvenanceSourceEntry>, ()> {
    let Some(provenance_map) = object_map(value)
        .and_then(|map| map.get("provenance"))
        .and_then(object_map)
    else {
        return Ok(Vec::new());
    };
    let Some(entries) = array_field(provenance_map, "sources") else {
        return Ok(Vec::new());
    };

    let mut output = Vec::new();
    for entry in entries {
        let Some(map) = object_map(entry) else {
            continue;
        };
        let Some(protocol) = string_field(map, "protocol") else {
            continue;
        };
        let Some(source_id) = string_field(map, "sourceId") else {
            continue;
        };
        output.push(ProvenanceSourceEntry {
            canonical_id: String::new(),
            protocol: protocol.to_string(),
            source_id: source_id.to_string(),
            author_id: string_field(map, "authorId").map(ToOwned::to_owned),
            observed_at: string_field(map, "observedAt").map(ToOwned::to_owned),
        });
    }
    Ok(output)
}

fn provenance_key(protocol: &str, source_id: &str) -> String {
    format!("{}::{}", protocol, source_id)
}

fn object_map(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map),
        _ => None,
    }
}

fn array_field<'a>(map: &'a BTreeMap<String, JsonValue>, key: &str) -> Option<&'a [JsonValue]> {
    match map.get(key) {
        Some(JsonValue::Array(items)) => Some(items.as_slice()),
        _ => None,
    }
}

fn string_field<'a>(map: &'a BTreeMap<String, JsonValue>, key: &str) -> Option<&'a str> {
    match map.get(key) {
        Some(JsonValue::String(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingonberry_core::SqliteStorageBackend;
    use lingonberry_protocol::{
        finalize_knowledge_object, parse_json, validate_knowledge_object, validate_publish_request,
    };
    use std::collections::BTreeMap;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("lingonberry-{}-{}", name, unique))
    }

    fn publish_fixture(backend: &SqliteStorageBackend, raw: &str) -> StoredCatalogRecord {
        let request = parse_json(raw).unwrap();
        assert!(validate_publish_request(&request).is_empty());
        let object = object_map(&request).unwrap().get("object").unwrap().clone();
        assert!(validate_knowledge_object(&object).is_empty());
        let finalized = finalize_knowledge_object(&object).unwrap();
        backend.append_publish_request(raw, &finalized).unwrap();
        backend.get(&finalized.canonical_id).unwrap().unwrap()
    }

    #[test]
    fn builds_type_index_from_canonical_store() {
        let backend = SqliteStorageBackend::new(temp_dir("type-index"));
        let inquiry = publish_fixture(
            &backend,
            include_str!("../../../fixtures/http-publish-request/minimal-request.json"),
        );

        let claim = {
            let mut object = inquiry.object.clone();
            if let JsonValue::Object(map) = &mut object {
                map.insert(
                    "id".to_string(),
                    JsonValue::String("lb:obj:example-0002".to_string()),
                );
                map.insert("type".to_string(), JsonValue::String("claim".to_string()));
                map.insert(
                    "body".to_string(),
                    JsonValue::Object({
                        let mut body = BTreeMap::new();
                        body.insert(
                            "text".to_string(),
                            JsonValue::String("This is a claim".to_string()),
                        );
                        body.insert("language".to_string(), JsonValue::String("en".to_string()));
                        body
                    }),
                );
                if let Some(JsonValue::Object(provenance)) = map.get_mut("provenance") {
                    if let Some(JsonValue::Array(sources)) = provenance.get_mut("sources") {
                        if let Some(JsonValue::Object(source)) = sources.first_mut() {
                            source.insert(
                                "sourceId".to_string(),
                                JsonValue::String("draft:example-0002".to_string()),
                            );
                            source.insert(
                                "observedAt".to_string(),
                                JsonValue::String("2026-06-17T00:00:00Z".to_string()),
                            );
                        }
                    }
                }
                if let Some(JsonValue::Object(raw_ref)) = map.get_mut("rawRef") {
                    raw_ref.insert(
                        "sourceId".to_string(),
                        JsonValue::String("draft:example-0002".to_string()),
                    );
                }
            }
            let finalized = finalize_knowledge_object(&object).unwrap();
            let request = JsonValue::Object({
                let mut publisher = BTreeMap::new();
                publisher.insert(
                    "publicKey".to_string(),
                    JsonValue::String(
                        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                            .to_string(),
                    ),
                );
                publisher.insert(
                    "signature".to_string(),
                    JsonValue::String("sig:example-0002".to_string()),
                );

                let mut request = BTreeMap::new();
                request.insert("object".to_string(), object.clone());
                request.insert("publisher".to_string(), JsonValue::Object(publisher));
                request
            });
            backend
                .append_publish_request(
                    &lingonberry_protocol::to_canonical_json(&request),
                    &finalized,
                )
                .unwrap();
            backend.get(&finalized.canonical_id).unwrap().unwrap()
        };

        let snapshot = IndexSnapshot::from_backend(&backend).unwrap();
        assert_eq!(
            snapshot.list_types(),
            vec!["claim".to_string(), "inquiry".to_string()]
        );
        assert_eq!(
            snapshot.ids_for_type("inquiry"),
            vec![inquiry.canonical_id.clone()]
        );
        assert_eq!(
            snapshot.ids_for_type("claim"),
            vec![claim.canonical_id.clone()]
        );
    }

    #[test]
    fn builds_graph_fragments_from_relations_lineage_and_provenance() {
        let mut object = parse_json(include_str!(
            "../../../fixtures/knowledge-object/minimal-wire-object.json"
        ))
        .unwrap();
        if let JsonValue::Object(map) = &mut object {
            map.insert(
                "relations".to_string(),
                JsonValue::Array(vec![JsonValue::Object({
                    let mut relation = BTreeMap::new();
                    relation.insert(
                        "source".to_string(),
                        JsonValue::String("lb:obj:example-0001".to_string()),
                    );
                    relation.insert(
                        "target".to_string(),
                        JsonValue::String("lb:obj:example-0002".to_string()),
                    );
                    relation.insert("kind".to_string(), JsonValue::String("cites".to_string()));
                    relation
                })]),
            );
            map.insert(
                "lineage".to_string(),
                JsonValue::Array(vec![JsonValue::Object({
                    let mut edge = BTreeMap::new();
                    edge.insert(
                        "type".to_string(),
                        JsonValue::String("derived_from".to_string()),
                    );
                    edge.insert(
                        "target".to_string(),
                        JsonValue::String("lb:obj:example-0000".to_string()),
                    );
                    edge
                })]),
            );
        }
        let finalized = finalize_knowledge_object(&object).unwrap();
        let snapshot = IndexSnapshot::from_records(vec![
            StoredCatalogRecord {
                stored_at: "2026-06-17T00:00:00Z".to_string(),
                canonical_id: finalized.canonical_id.clone(),
                carrier_identity: "carrier:example".to_string(),
                object: finalized.object.clone(),
            },
            StoredCatalogRecord {
                stored_at: "2026-06-17T00:00:01Z".to_string(),
                canonical_id: "lb:obj:example-0003".to_string(),
                carrier_identity: "carrier:example-2".to_string(),
                object: {
                    let mut inbound_object = finalized.object.clone();
                    if let JsonValue::Object(map) = &mut inbound_object {
                        map.insert(
                            "id".to_string(),
                            JsonValue::String("lb:obj:example-0003".to_string()),
                        );
                        map.insert(
                            "lineage".to_string(),
                            JsonValue::Array(vec![JsonValue::Object({
                                let mut edge = BTreeMap::new();
                                edge.insert(
                                    "type".to_string(),
                                    JsonValue::String("revises".to_string()),
                                );
                                edge.insert(
                                    "target".to_string(),
                                    JsonValue::String(finalized.canonical_id.clone()),
                                );
                                edge
                            })]),
                        );
                    }
                    inbound_object
                },
            },
        ]);

        let relation_graph = snapshot.relation_graph(&finalized.canonical_id).unwrap();
        assert_eq!(relation_graph.outbound.len(), 1);
        assert_eq!(relation_graph.inbound.len(), 0);
        assert_eq!(
            relation_graph.related_ids,
            vec!["lb:obj:example-0002".to_string()]
        );

        let lineage_graph = snapshot.lineage_graph(&finalized.canonical_id).unwrap();
        assert_eq!(lineage_graph.outbound.len(), 1);
        assert_eq!(lineage_graph.inbound.len(), 1);
        assert_eq!(
            lineage_graph.related_ids,
            vec![
                "lb:obj:example-0000".to_string(),
                "lb:obj:example-0003".to_string()
            ]
        );

        let fragment = snapshot.graph_fragment(&finalized.canonical_id).unwrap();
        assert_eq!(fragment.relations_outbound.len(), 1);
        assert_eq!(fragment.lineage_outbound.len(), 1);
        assert_eq!(fragment.provenance_sources.len(), 1);
        assert_eq!(
            snapshot.provenance_ids_for_source("lingonberry", "draft:example-0001"),
            vec![
                finalized.canonical_id.clone(),
                "lb:obj:example-0003".to_string()
            ]
        );
    }

    #[test]
    fn builds_provenance_graph_from_source_key() {
        let base = parse_json(include_str!(
            "../../../fixtures/knowledge-object/minimal-wire-object.json"
        ))
        .unwrap();
        let mut second = base.clone();
        if let JsonValue::Object(map) = &mut second {
            map.insert(
                "id".to_string(),
                JsonValue::String("lb:obj:example-0004".to_string()),
            );
            if let Some(JsonValue::Object(provenance)) = map.get_mut("provenance") {
                if let Some(JsonValue::Array(sources)) = provenance.get_mut("sources") {
                    if let Some(JsonValue::Object(source)) = sources.first_mut() {
                        source.insert("authorId".to_string(), JsonValue::String("bob".to_string()));
                        source.insert(
                            "observedAt".to_string(),
                            JsonValue::String("2026-06-17T01:00:00Z".to_string()),
                        );
                    }
                }
            }
        }

        let first = finalize_knowledge_object(&base).unwrap();
        let second = finalize_knowledge_object(&second).unwrap();
        let snapshot = IndexSnapshot::from_records(vec![
            StoredCatalogRecord {
                stored_at: "2026-06-17T00:00:00Z".to_string(),
                canonical_id: first.canonical_id.clone(),
                carrier_identity: "carrier:first".to_string(),
                object: first.object.clone(),
            },
            StoredCatalogRecord {
                stored_at: "2026-06-17T00:00:01Z".to_string(),
                canonical_id: second.canonical_id.clone(),
                carrier_identity: "carrier:second".to_string(),
                object: second.object.clone(),
            },
        ]);

        let fragment = snapshot
            .provenance_graph("lingonberry", "draft:example-0001")
            .unwrap();
        assert_eq!(
            fragment.canonical_ids,
            vec![first.canonical_id.clone(), second.canonical_id.clone()]
        );
        assert_eq!(fragment.entries.len(), 2);
        assert_eq!(fragment.entries[1].author_id.as_deref(), Some("bob"));
        assert_eq!(
            fragment.entries[1].observed_at.as_deref(),
            Some("2026-06-17T01:00:00Z")
        );
    }

    #[test]
    fn rebuild_from_backend_preserves_query_results() {
        let backend = SqliteStorageBackend::new(temp_dir("rebuild-index"));
        let first = publish_fixture(
            &backend,
            include_str!("../../../fixtures/http-publish-request/minimal-request.json"),
        );
        let snapshot = IndexSnapshot::rebuild_from_backend(&backend).unwrap();
        assert_eq!(snapshot.record_count(), 1);
        assert_eq!(snapshot.list_types(), vec!["inquiry".to_string()]);
        assert_eq!(
            snapshot.ids_for_type("inquiry"),
            vec![first.canonical_id.clone()]
        );
        assert_eq!(snapshot.relation_edges().len(), 0);
        assert_eq!(snapshot.lineage_edges().len(), 0);
        assert_eq!(snapshot.provenance_source_count(), 1);
        assert!(snapshot.record(&first.canonical_id).is_some());
    }
}
