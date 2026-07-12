use std::collections::{BTreeMap, BTreeSet};

use lingonberry_protocol::JsonValue;

use super::QuarantineStore;
use crate::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuarantineStatus {
    total: usize,
    pending: usize,
    promoted: usize,
    oldest_pending_at: Option<String>,
    latest_received_at: Option<String>,
    latest_promoted_at: Option<String>,
    reason_code_counts: BTreeMap<String, usize>,
}

impl QuarantineStore {
    pub fn status_json(&self) -> Result<JsonValue, StoreError> {
        let status = quarantine_status(self)?;
        Ok(quarantine_status_json(&status))
    }
}

fn quarantine_status(store: &QuarantineStore) -> Result<QuarantineStatus, StoreError> {
    let records = store.list()?;
    let resolutions = store.list_resolutions()?;

    let record_ids: BTreeSet<&str> = records.iter().map(|record| record.id.as_str()).collect();
    let promoted_ids: BTreeSet<&str> = resolutions
        .iter()
        .filter_map(|resolution| {
            record_ids
                .contains(resolution.quarantine_id.as_str())
                .then_some(resolution.quarantine_id.as_str())
        })
        .collect();

    let mut reason_code_counts = BTreeMap::new();
    for record in &records {
        *reason_code_counts
            .entry(record.reason_code.clone())
            .or_insert(0) += 1;
    }

    let oldest_pending_at = records
        .iter()
        .filter(|record| !promoted_ids.contains(record.id.as_str()))
        .map(|record| record.received_at.clone())
        .min();
    let latest_received_at = records
        .iter()
        .map(|record| record.received_at.clone())
        .max();
    let latest_promoted_at = resolutions
        .iter()
        .filter(|resolution| record_ids.contains(resolution.quarantine_id.as_str()))
        .map(|resolution| resolution.resolved_at.clone())
        .max();

    let total = records.len();
    let promoted = promoted_ids.len();

    Ok(QuarantineStatus {
        total,
        pending: total.saturating_sub(promoted),
        promoted,
        oldest_pending_at,
        latest_received_at,
        latest_promoted_at,
        reason_code_counts,
    })
}

fn quarantine_status_json(status: &QuarantineStatus) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "latestPromotedAt".to_string(),
            optional_string_json(&status.latest_promoted_at),
        ),
        (
            "latestReceivedAt".to_string(),
            optional_string_json(&status.latest_received_at),
        ),
        (
            "oldestPendingAt".to_string(),
            optional_string_json(&status.oldest_pending_at),
        ),
        (
            "pending".to_string(),
            JsonValue::Number(status.pending.to_string()),
        ),
        (
            "promoted".to_string(),
            JsonValue::Number(status.promoted.to_string()),
        ),
        (
            "reasonCodeCounts".to_string(),
            JsonValue::Object(
                status
                    .reason_code_counts
                    .iter()
                    .map(|(code, count)| (code.clone(), JsonValue::Number(count.to_string())))
                    .collect(),
            ),
        ),
        (
            "total".to_string(),
            JsonValue::Number(status.total.to_string()),
        ),
    ]))
}

fn optional_string_json(value: &Option<String>) -> JsonValue {
    match value {
        Some(value) => JsonValue::String(value.clone()),
        None => JsonValue::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-quarantine-status-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn empty_store_has_zero_status() {
        let dir = temp_dir();
        let status = quarantine_status(&QuarantineStore::new(&dir)).unwrap();

        assert_eq!(status.total, 0);
        assert_eq!(status.pending, 0);
        assert_eq!(status.promoted, 0);
        assert_eq!(status.oldest_pending_at, None);
        assert_eq!(status.latest_received_at, None);
        assert_eq!(status.latest_promoted_at, None);
        assert!(status.reason_code_counts.is_empty());
    }

    #[test]
    fn status_tracks_pending_promoted_and_reason_codes() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        let first = store
            .append("{\"object\":{}}", "LB_IDENTITY_DEFERRED", &[])
            .unwrap();
        let second = store
            .append("{\"object\":{}}", "LB_POLICY_DEFERRED", &[])
            .unwrap();
        let third = store
            .append("{\"object\":{}}", "LB_IDENTITY_DEFERRED", &[])
            .unwrap();

        store
            .append_resolution(&first.id, "lb:obj:first", false)
            .unwrap();

        let status = quarantine_status(&store).unwrap();
        assert_eq!(status.total, 3);
        assert_eq!(status.pending, 2);
        assert_eq!(status.promoted, 1);
        assert_eq!(status.oldest_pending_at, Some(second.received_at));
        assert_eq!(status.latest_received_at, Some(third.received_at));
        assert!(status.latest_promoted_at.is_some());
        assert_eq!(status.reason_code_counts["LB_IDENTITY_DEFERRED"], 2);
        assert_eq!(status.reason_code_counts["LB_POLICY_DEFERRED"], 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn unknown_and_duplicate_resolutions_are_not_double_counted() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        let record = store
            .append("{\"object\":{}}", "LB_IDENTITY_DEFERRED", &[])
            .unwrap();

        fs::create_dir_all(&dir).unwrap();
        fs::write(
            store.resolutions_path(),
            format!(
                "{{\"canonicalId\":\"lb:obj:one\",\"duplicate\":false,\"quarantineId\":\"{}\",\"resolvedAt\":\"1.000000000Z\",\"status\":\"promoted\"}}\n{{\"canonicalId\":\"lb:obj:two\",\"duplicate\":true,\"quarantineId\":\"{}\",\"resolvedAt\":\"2.000000000Z\",\"status\":\"promoted\"}}\n{{\"canonicalId\":\"lb:obj:unknown\",\"duplicate\":false,\"quarantineId\":\"lb:q:unknown\",\"resolvedAt\":\"3.000000000Z\",\"status\":\"promoted\"}}\n",
                record.id, record.id
            ),
        )
        .unwrap();

        let status = quarantine_status(&store).unwrap();
        assert_eq!(status.total, 1);
        assert_eq!(status.pending, 0);
        assert_eq!(status.promoted, 1);
        assert_eq!(status.latest_promoted_at, Some("2.000000000Z".to_string()));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn corrupt_ledger_is_reported() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(store.path(), "not-json\n").unwrap();

        let error = quarantine_status(&store).unwrap_err();
        assert_eq!(error.code, "LB_QUARANTINE_CORRUPT");

        let _ = fs::remove_dir_all(dir);
    }
}
