use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use super::QuarantineStore;
use crate::{
    acquire_quarantine_lock, read_managed_ledger_lines, resolve_quarantine_active_path,
    store_error, StoreError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineAnnotation {
    pub id: String,
    pub quarantine_id: String,
    pub annotated_at: String,
    pub operator: String,
    pub note: String,
}

impl QuarantineStore {
    pub fn annotations_path(&self) -> Result<PathBuf, StoreError> {
        resolve_quarantine_active_path(self.state_dir(), "quarantine-annotations.jsonl")
    }

    pub fn append_annotation(
        &self,
        quarantine_id: &str,
        operator: &str,
        note: &str,
    ) -> Result<QuarantineAnnotation, StoreError> {
        let _lock = acquire_quarantine_lock(self.state_dir(), "quarantine-annotate")?;
        if self.get(quarantine_id)?.is_none() {
            return Err(store_error(
                "LB_QUARANTINE_NOT_FOUND",
                format!("quarantine record not found: {quarantine_id}"),
            ));
        }
        let operator = operator.trim();
        let note = note.trim();
        if operator.is_empty() || note.is_empty() {
            return Err(store_error(
                "LB_QUARANTINE_ANNOTATION",
                "operator and note must not be empty",
            ));
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        let annotation = QuarantineAnnotation {
            id: format!("lb:qa:{}-{}", now.as_secs(), now.subsec_nanos()),
            quarantine_id: quarantine_id.to_string(),
            annotated_at: format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos()),
            operator: operator.to_string(),
            note: note.to_string(),
        };
        append_annotation_line(&self.annotations_path()?, &annotation)?;
        Ok(annotation)
    }

    pub fn list_annotations(
        &self,
        quarantine_id: Option<&str>,
    ) -> Result<Vec<QuarantineAnnotation>, StoreError> {
        let mut annotations = Vec::new();
        for line in read_managed_ledger_lines(self.state_dir(), "quarantine-annotations.jsonl")? {
            let annotation = parse_annotation(&line)?;
            if quarantine_id
                .map(|id| annotation.quarantine_id == id)
                .unwrap_or(true)
            {
                annotations.push(annotation);
            }
        }
        Ok(annotations)
    }
}

pub fn quarantine_annotation_json(annotation: &QuarantineAnnotation) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("id".to_string(), JsonValue::String(annotation.id.clone())),
        (
            "quarantineId".to_string(),
            JsonValue::String(annotation.quarantine_id.clone()),
        ),
        (
            "annotatedAt".to_string(),
            JsonValue::String(annotation.annotated_at.clone()),
        ),
        (
            "operator".to_string(),
            JsonValue::String(annotation.operator.clone()),
        ),
        (
            "note".to_string(),
            JsonValue::String(annotation.note.clone()),
        ),
    ]))
}

fn append_annotation_line(
    path: &std::path::Path,
    annotation: &QuarantineAnnotation,
) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    writeln!(
        file,
        "{}",
        to_canonical_json(&quarantine_annotation_json(annotation))
    )
    .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))
}

fn parse_annotation(line: &str) -> Result<QuarantineAnnotation, StoreError> {
    let map = match parse_json(line)
        .map_err(|error| store_error("LB_QUARANTINE_CORRUPT", error.to_string()))?
    {
        JsonValue::Object(map) => map,
        _ => {
            return Err(store_error(
                "LB_QUARANTINE_CORRUPT",
                "annotation is not an object",
            ))
        }
    };
    Ok(QuarantineAnnotation {
        id: required_string(&map, "id")?,
        quarantine_id: required_string(&map, "quarantineId")?,
        annotated_at: required_string(&map, "annotatedAt")?,
        operator: required_string(&map, "operator")?,
        note: required_string(&map, "note")?,
    })
}

fn required_string(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<String, StoreError> {
    match map.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(store_error(
            "LB_QUARANTINE_CORRUPT",
            format!("annotation missing {name}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_quarantine_ledger_index, rotate_quarantine_ledger};

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-annotations-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn reads_archived_and_active_annotations() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        let record = store
            .append("{\"object\":{}}", "LB_IDENTITY_DEFERRED", &[])
            .unwrap();
        store
            .append_annotation(&record.id, "one", "archived")
            .unwrap();
        build_quarantine_ledger_index(&dir).unwrap();
        rotate_quarantine_ledger(&dir, "quarantine-annotations.jsonl").unwrap();
        store
            .append_annotation(&record.id, "two", "active")
            .unwrap();
        let annotations = store.list_annotations(Some(&record.id)).unwrap();
        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0].operator, "one");
        assert_eq!(annotations[1].operator, "two");
        let _ = fs::remove_dir_all(dir);
    }
}
