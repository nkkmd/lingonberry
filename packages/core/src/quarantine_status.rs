use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_protocol::JsonValue;

use super::QuarantineStore;
use crate::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuarantineStatus {
    total: usize,
    pending: usize,
    promoted: usize,
    dismissed: usize,
    oldest_pending_at: Option<String>,
    latest_received_at: Option<String>,
    latest_promoted_at: Option<String>,
    latest_dismissed_at: Option<String>,
    reason_code_counts: BTreeMap<String, usize>,
}

impl QuarantineStore {
    pub fn status_json(&self) -> Result<JsonValue, StoreError> {
        let status = quarantine_status(self)?;
        Ok(quarantine_status_json(&status))
    }

    pub fn metrics_text(&self) -> Result<String, StoreError> {
        let status = quarantine_status(self)?;
        let now_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| crate::store_error("LB_QUARANTINE_IO", error.to_string()))?
            .as_secs();
        Ok(quarantine_metrics_text(&status, now_seconds))
    }
}

fn quarantine_status(store: &QuarantineStore) -> Result<QuarantineStatus, StoreError> {
    let records = store.list_all()?;
    let resolutions = store.list_resolutions()?;
    let dismissals = store.list_dismissals(None)?;

    let record_ids: BTreeSet<&str> = records.iter().map(|record| record.id.as_str()).collect();
    let promoted_ids: BTreeSet<&str> = resolutions
        .iter()
        .filter_map(|resolution| {
            record_ids
                .contains(resolution.quarantine_id.as_str())
                .then_some(resolution.quarantine_id.as_str())
        })
        .collect();
    let dismissed_ids: BTreeSet<&str> = dismissals
        .iter()
        .filter_map(|dismissal| {
            (record_ids.contains(dismissal.quarantine_id.as_str())
                && !promoted_ids.contains(dismissal.quarantine_id.as_str()))
            .then_some(dismissal.quarantine_id.as_str())
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
        .filter(|record| {
            !promoted_ids.contains(record.id.as_str())
                && !dismissed_ids.contains(record.id.as_str())
        })
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
    let latest_dismissed_at = dismissals
        .iter()
        .filter(|dismissal| dismissed_ids.contains(dismissal.quarantine_id.as_str()))
        .map(|dismissal| dismissal.dismissed_at.clone())
        .max();

    let total = records.len();
    let promoted = promoted_ids.len();
    let dismissed = dismissed_ids.len();

    Ok(QuarantineStatus {
        total,
        pending: total.saturating_sub(promoted).saturating_sub(dismissed),
        promoted,
        dismissed,
        oldest_pending_at,
        latest_received_at,
        latest_promoted_at,
        latest_dismissed_at,
        reason_code_counts,
    })
}

fn quarantine_status_json(status: &QuarantineStatus) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "dismissed".to_string(),
            JsonValue::Number(status.dismissed.to_string()),
        ),
        (
            "latestDismissedAt".to_string(),
            optional_string_json(&status.latest_dismissed_at),
        ),
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

fn quarantine_metrics_text(status: &QuarantineStatus, now_seconds: u64) -> String {
    let oldest_pending_age = status
        .oldest_pending_at
        .as_deref()
        .and_then(timestamp_seconds)
        .map(|received| now_seconds.saturating_sub(received))
        .unwrap_or(0);

    let mut output = String::from(
        "# HELP lingonberry_quarantine_records Current quarantine records by persistent lifecycle state.\n\
# TYPE lingonberry_quarantine_records gauge\n",
    );
    output.push_str(&format!(
        "lingonberry_quarantine_records{{state=\"total\"}} {}\n",
        status.total
    ));
    output.push_str(&format!(
        "lingonberry_quarantine_records{{state=\"pending\"}} {}\n",
        status.pending
    ));
    output.push_str(&format!(
        "lingonberry_quarantine_records{{state=\"promoted\"}} {}\n",
        status.promoted
    ));
    output.push_str(&format!(
        "lingonberry_quarantine_records{{state=\"dismissed\"}} {}\n",
        status.dismissed
    ));
    output.push_str(
        "# HELP lingonberry_quarantine_oldest_pending_age_seconds Age of the oldest pending quarantine record.\n\
# TYPE lingonberry_quarantine_oldest_pending_age_seconds gauge\n",
    );
    output.push_str(&format!(
        "lingonberry_quarantine_oldest_pending_age_seconds {}\n",
        oldest_pending_age
    ));
    output.push_str(
        "# HELP lingonberry_quarantine_reason_code_records Quarantine records grouped by bounded reason code.\n\
# TYPE lingonberry_quarantine_reason_code_records gauge\n",
    );
    for (reason_code, count) in &status.reason_code_counts {
        output.push_str(&format!(
            "lingonberry_quarantine_reason_code_records{{reason_code=\"{}\"}} {}\n",
            escape_metric_label(reason_code),
            count
        ));
    }
    output
}

fn timestamp_seconds(value: &str) -> Option<u64> {
    value.strip_suffix('Z')?.split('.').next()?.parse().ok()
}

fn escape_metric_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
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
    use crate::OPERATOR_DISMISSED_REASON_CODE;
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
        assert_eq!(status.dismissed, 0);
        assert_eq!(status.oldest_pending_at, None);
        assert_eq!(status.latest_received_at, None);
        assert_eq!(status.latest_promoted_at, None);
        assert_eq!(status.latest_dismissed_at, None);
        assert!(status.reason_code_counts.is_empty());
    }

    #[test]
    fn status_tracks_pending_promoted_dismissed_and_reason_codes() {
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
        store
            .dismiss(
                &third.id,
                "operator",
                OPERATOR_DISMISSED_REASON_CODE,
                "duplicate",
            )
            .unwrap();

        let status = quarantine_status(&store).unwrap();
        assert_eq!(status.total, 3);
        assert_eq!(status.pending, 1);
        assert_eq!(status.promoted, 1);
        assert_eq!(status.dismissed, 1);
        assert_eq!(status.oldest_pending_at, Some(second.received_at));
        assert!(status.latest_promoted_at.is_some());
        assert!(status.latest_dismissed_at.is_some());
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
        assert_eq!(status.dismissed, 0);
        assert_eq!(status.latest_promoted_at, Some("2.000000000Z".to_string()));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn metrics_report_dismissed_count_age_and_escaped_reason_codes() {
        let status = QuarantineStatus {
            total: 3,
            pending: 1,
            promoted: 1,
            dismissed: 1,
            oldest_pending_at: Some("100.000000000Z".to_string()),
            latest_received_at: None,
            latest_promoted_at: None,
            latest_dismissed_at: None,
            reason_code_counts: BTreeMap::from([("LB_\"TEST\\CODE".to_string(), 2)]),
        };
        let metrics = quarantine_metrics_text(&status, 145);
        assert!(metrics.contains("lingonberry_quarantine_records{state=\"pending\"} 1"));
        assert!(metrics.contains("lingonberry_quarantine_records{state=\"dismissed\"} 1"));
        assert!(metrics.contains("lingonberry_quarantine_oldest_pending_age_seconds 45"));
        assert!(metrics.contains("reason_code=\"LB_\\\"TEST\\\\CODE\"} 2"));
    }

    #[test]
    fn corrupt_ledger_is_reported() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(store.dismissals_path(), "not-json\n").unwrap();
        let error = quarantine_status(&store).unwrap_err();
        assert_eq!(error.code, "LB_QUARANTINE_CORRUPT");
        let _ = fs::remove_dir_all(dir);
    }
}