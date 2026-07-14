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
    permanently_rejected: usize,
    oldest_pending_at: Option<String>,
    latest_received_at: Option<String>,
    latest_promoted_at: Option<String>,
    latest_dismissed_at: Option<String>,
    latest_permanently_rejected_at: Option<String>,
    reason_code_counts: BTreeMap<String, usize>,
}

impl QuarantineStore {
    pub fn status_json(&self) -> Result<JsonValue, StoreError> {
        Ok(quarantine_status_json(&quarantine_status(self)?))
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
    let permanent_rejections = store.list_permanent_rejections(None)?;

    let record_ids: BTreeSet<&str> = records.iter().map(|record| record.id.as_str()).collect();
    let promoted_ids: BTreeSet<&str> = resolutions
        .iter()
        .filter_map(|event| {
            record_ids
                .contains(event.quarantine_id.as_str())
                .then_some(event.quarantine_id.as_str())
        })
        .collect();
    let dismissed_ids: BTreeSet<&str> = dismissals
        .iter()
        .filter_map(|event| {
            (record_ids.contains(event.quarantine_id.as_str())
                && !promoted_ids.contains(event.quarantine_id.as_str()))
            .then_some(event.quarantine_id.as_str())
        })
        .collect();
    let permanently_rejected_ids: BTreeSet<&str> = permanent_rejections
        .iter()
        .filter_map(|event| {
            (record_ids.contains(event.quarantine_id.as_str())
                && !promoted_ids.contains(event.quarantine_id.as_str())
                && !dismissed_ids.contains(event.quarantine_id.as_str()))
            .then_some(event.quarantine_id.as_str())
        })
        .collect();

    let mut reason_code_counts = BTreeMap::new();
    for record in &records {
        *reason_code_counts
            .entry(record.reason_code.clone())
            .or_insert(0) += 1;
    }

    let is_pending = |id: &str| {
        !promoted_ids.contains(id)
            && !dismissed_ids.contains(id)
            && !permanently_rejected_ids.contains(id)
    };
    let oldest_pending_at = records
        .iter()
        .filter(|record| is_pending(record.id.as_str()))
        .map(|record| record.received_at.clone())
        .min();
    let latest_received_at = records
        .iter()
        .map(|record| record.received_at.clone())
        .max();
    let latest_promoted_at = resolutions
        .iter()
        .filter(|event| promoted_ids.contains(event.quarantine_id.as_str()))
        .map(|event| event.resolved_at.clone())
        .max();
    let latest_dismissed_at = dismissals
        .iter()
        .filter(|event| dismissed_ids.contains(event.quarantine_id.as_str()))
        .map(|event| event.dismissed_at.clone())
        .max();
    let latest_permanently_rejected_at = permanent_rejections
        .iter()
        .filter(|event| permanently_rejected_ids.contains(event.quarantine_id.as_str()))
        .map(|event| event.rejected_at.clone())
        .max();

    let total = records.len();
    let promoted = promoted_ids.len();
    let dismissed = dismissed_ids.len();
    let permanently_rejected = permanently_rejected_ids.len();

    Ok(QuarantineStatus {
        total,
        pending: total
            .saturating_sub(promoted)
            .saturating_sub(dismissed)
            .saturating_sub(permanently_rejected),
        promoted,
        dismissed,
        permanently_rejected,
        oldest_pending_at,
        latest_received_at,
        latest_promoted_at,
        latest_dismissed_at,
        latest_permanently_rejected_at,
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
            "latestPermanentlyRejectedAt".to_string(),
            optional_string_json(&status.latest_permanently_rejected_at),
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
            "permanentlyRejected".to_string(),
            JsonValue::Number(status.permanently_rejected.to_string()),
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
    for (state, count) in [
        ("total", status.total),
        ("pending", status.pending),
        ("promoted", status.promoted),
        ("dismissed", status.dismissed),
        ("permanently_rejected", status.permanently_rejected),
    ] {
        output.push_str(&format!(
            "lingonberry_quarantine_records{{state=\"{state}\"}} {count}\n"
        ));
    }
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
    use crate::{OPERATOR_DISMISSED_REASON_CODE, OPERATOR_PERMANENTLY_REJECTED_REASON_CODE};
    use std::fs;
    use std::path::PathBuf;

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
        let status = quarantine_status(&QuarantineStore::new(temp_dir())).unwrap();
        assert_eq!(status.total, 0);
        assert_eq!(status.pending, 0);
        assert_eq!(status.permanently_rejected, 0);
        assert_eq!(status.latest_permanently_rejected_at, None);
    }

    #[test]
    fn tracks_all_persistent_states() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        let promoted = store.append("{\"object\":{}}", "LB_A", &[]).unwrap();
        let pending = store.append("{\"object\":{}}", "LB_B", &[]).unwrap();
        let dismissed = store.append("{\"object\":{}}", "LB_C", &[]).unwrap();
        let rejected = store.append("{\"object\":{}}", "LB_D", &[]).unwrap();
        store
            .append_resolution(&promoted.id, "lb:obj:first", false)
            .unwrap();
        store
            .dismiss(
                &dismissed.id,
                "operator",
                OPERATOR_DISMISSED_REASON_CODE,
                "duplicate",
            )
            .unwrap();
        store
            .permanently_reject(
                &rejected.id,
                "operator",
                OPERATOR_PERMANENTLY_REJECTED_REASON_CODE,
                "prohibited",
            )
            .unwrap();
        let status = quarantine_status(&store).unwrap();
        assert_eq!(status.total, 4);
        assert_eq!(status.pending, 1);
        assert_eq!(status.promoted, 1);
        assert_eq!(status.dismissed, 1);
        assert_eq!(status.permanently_rejected, 1);
        assert_eq!(status.oldest_pending_at, Some(pending.received_at));
        assert!(status.latest_permanently_rejected_at.is_some());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn metrics_include_permanent_rejection_gauge() {
        let status = QuarantineStatus {
            total: 4,
            pending: 1,
            promoted: 1,
            dismissed: 1,
            permanently_rejected: 1,
            oldest_pending_at: Some("100.000000000Z".to_string()),
            latest_received_at: None,
            latest_promoted_at: None,
            latest_dismissed_at: None,
            latest_permanently_rejected_at: None,
            reason_code_counts: BTreeMap::new(),
        };
        let metrics = quarantine_metrics_text(&status, 145);
        assert!(
            metrics.contains("lingonberry_quarantine_records{state=\"permanently_rejected\"} 1")
        );
        assert!(metrics.contains("lingonberry_quarantine_oldest_pending_age_seconds 45"));
    }

    #[test]
    fn corrupt_rejection_ledger_is_reported() {
        let dir = temp_dir();
        let store = QuarantineStore::new(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(store.permanent_rejections_path().unwrap(), "not-json\n").unwrap();
        assert_eq!(
            quarantine_status(&store).unwrap_err().code,
            "LB_QUARANTINE_CORRUPT"
        );
        let _ = fs::remove_dir_all(dir);
    }
}
