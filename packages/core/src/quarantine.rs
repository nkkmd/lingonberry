use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{store_error, StoreError};

#[derive(Debug, Clone)]
pub struct QuarantineRecord {
    pub id: String,
    pub received_at: String,
    pub reason_code: String,
    pub reasons: Vec<String>,
    pub request_json: String,
}

#[derive(Debug, Clone)]
pub struct QuarantineStore {
    path: PathBuf,
}

impl QuarantineStore {
    pub fn new(state_dir: impl AsRef<Path>) -> Self {
        Self {
            path: state_dir.as_ref().join("quarantine.jsonl"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(
        &self,
        request_json: &str,
        reason_code: &str,
        reasons: &[String],
    ) -> Result<QuarantineRecord, StoreError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        let id = format!("lb:q:{}-{}", now.as_secs(), now.subsec_nanos());
        let received_at = format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos());
        let record = QuarantineRecord {
            id,
            received_at,
            reason_code: reason_code.to_string(),
            reasons: reasons.to_vec(),
            request_json: request_json.to_string(),
        };
        self.append_record(&record)?;
        Ok(record)
    }

    pub fn list(&self) -> Result<Vec<QuarantineRecord>, StoreError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(&self.path)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        let mut records = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
            if line.trim().is_empty() {
                continue;
            }
            records.push(parse_record(&line)?);
        }
        Ok(records)
    }

    pub fn get(&self, id: &str) -> Result<Option<QuarantineRecord>, StoreError> {
        Ok(self.list()?.into_iter().find(|record| record.id == id))
    }

    fn append_record(&self, record: &QuarantineRecord) -> Result<(), StoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        writeln!(file, "{}", to_canonical_json(&record_json(record)))
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))
    }
}

fn record_json(record: &QuarantineRecord) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("id".to_string(), JsonValue::String(record.id.clone())),
        (
            "receivedAt".to_string(),
            JsonValue::String(record.received_at.clone()),
        ),
        (
            "reasonCode".to_string(),
            JsonValue::String(record.reason_code.clone()),
        ),
        (
            "reasons".to_string(),
            JsonValue::Array(
                record
                    .reasons
                    .iter()
                    .cloned()
                    .map(JsonValue::String)
                    .collect(),
            ),
        ),
        (
            "requestJson".to_string(),
            JsonValue::String(record.request_json.clone()),
        ),
    ]))
}

fn parse_record(line: &str) -> Result<QuarantineRecord, StoreError> {
    let value = parse_json(line)
        .map_err(|error| store_error("LB_QUARANTINE_CORRUPT", error.to_string()))?;
    let JsonValue::Object(map) = value else {
        return Err(store_error(
            "LB_QUARANTINE_CORRUPT",
            "record is not an object",
        ));
    };
    let string = |name: &str| match map.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(store_error(
            "LB_QUARANTINE_CORRUPT",
            format!("record missing {name}"),
        )),
    };
    let reasons = match map.get("reasons") {
        Some(JsonValue::Array(items)) => items
            .iter()
            .map(|item| match item {
                JsonValue::String(value) => Ok(value.clone()),
                _ => Err(store_error(
                    "LB_QUARANTINE_CORRUPT",
                    "reason is not a string",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => {
            return Err(store_error(
                "LB_QUARANTINE_CORRUPT",
                "record missing reasons",
            ))
        }
    };
    Ok(QuarantineRecord {
        id: string("id")?,
        received_at: string("receivedAt")?,
        reason_code: string("reasonCode")?,
        reasons,
        request_json: string("requestJson")?,
    })
}

pub fn quarantine_record_json(record: &QuarantineRecord) -> JsonValue {
    record_json(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_lists_and_gets_records() {
        let dir = std::env::temp_dir().join(format!(
            "lingonberry-quarantine-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = QuarantineStore::new(&dir);
        let record = store
            .append(
                "{\"object\":{}}",
                "LB_IDENTITY_DEFERRED",
                &["future rule".to_string()],
            )
            .unwrap();
        assert_eq!(store.list().unwrap().len(), 1);
        assert_eq!(
            store.get(&record.id).unwrap().unwrap().reason_code,
            "LB_IDENTITY_DEFERRED"
        );
        let _ = fs::remove_dir_all(dir);
    }
}
