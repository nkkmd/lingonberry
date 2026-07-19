use lingonberry_core::{
    publish_ingestion_result_json, PublishIngestionResult, PublishIngestionStatus,
};
use lingonberry_protocol::JsonValue;

#[derive(Debug, Clone)]
pub struct IngestionHttpResponse {
    pub status_code: u16,
    pub status_text: &'static str,
    pub body: JsonValue,
}

pub fn ingestion_http_response(result: &PublishIngestionResult) -> IngestionHttpResponse {
    let (status_code, status_text) = match result.status {
        PublishIngestionStatus::Stored => (201, "Created"),
        PublishIngestionStatus::Duplicate => (200, "OK"),
        PublishIngestionStatus::Deferred => (202, "Accepted"),
        PublishIngestionStatus::Rejected => rejection_http_status(&result.code),
        PublishIngestionStatus::Conflict => (409, "Conflict"),
        PublishIngestionStatus::Failed => (500, "Internal Server Error"),
    };

    IngestionHttpResponse {
        status_code,
        status_text,
        body: publish_ingestion_result_json(result),
    }
}

pub fn ingestion_cli_error(result: &PublishIngestionResult) -> Option<String> {
    match result.status {
        PublishIngestionStatus::Stored
        | PublishIngestionStatus::Duplicate
        | PublishIngestionStatus::Deferred => None,
        PublishIngestionStatus::Rejected
        | PublishIngestionStatus::Conflict
        | PublishIngestionStatus::Failed => {
            let detail = if result.errors.is_empty() {
                String::new()
            } else {
                format!(": {}", result.errors.join("; "))
            };
            Some(format!("{}{}", result.code, detail))
        }
    }
}

fn rejection_http_status(code: &str) -> (u16, &'static str) {
    match code {
        "LB_UNSUPPORTED_IDENTITY_RULE" => (422, "Unprocessable Entity"),
        _ => (400, "Bad Request"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingonberry_core::PUBLISH_INGESTION_CONTRACT_VERSION;

    fn result(status: PublishIngestionStatus, code: &str) -> PublishIngestionResult {
        PublishIngestionResult {
            contract_version: PUBLISH_INGESTION_CONTRACT_VERSION,
            status,
            code: code.to_string(),
            errors: vec!["detail".to_string()],
            canonical_id: None,
            identity_key: None,
            carrier_identity: None,
            stored_at: None,
            object: None,
            quarantine_id: None,
        }
    }

    #[test]
    fn stored_and_duplicate_have_distinct_success_statuses() {
        assert_eq!(
            ingestion_http_response(&result(PublishIngestionStatus::Stored, "LB_OBJECT_STORED"))
                .status_code,
            201
        );
        assert_eq!(
            ingestion_http_response(&result(
                PublishIngestionStatus::Duplicate,
                "LB_OBJECT_DUPLICATE"
            ))
            .status_code,
            200
        );
    }

    #[test]
    fn deferred_is_accepted_without_cli_error() {
        let result = result(
            PublishIngestionStatus::Deferred,
            "LB_IDENTITY_CLAIM_DEFERRED",
        );
        assert_eq!(ingestion_http_response(&result).status_code, 202);
        assert_eq!(ingestion_cli_error(&result), None);
    }

    #[test]
    fn rejection_conflict_and_failure_are_machine_readable_errors() {
        let rejected = result(PublishIngestionStatus::Rejected, "LB_VALIDATION_FAILED");
        let conflict = result(PublishIngestionStatus::Conflict, "LB_OBJECT_CONFLICT");
        let failed = result(PublishIngestionStatus::Failed, "LB_SQLITE_EXEC");

        assert_eq!(ingestion_http_response(&rejected).status_code, 400);
        assert_eq!(ingestion_http_response(&conflict).status_code, 409);
        assert_eq!(ingestion_http_response(&failed).status_code, 500);
        assert_eq!(
            ingestion_cli_error(&conflict),
            Some("LB_OBJECT_CONFLICT: detail".to_string())
        );
    }

    #[test]
    fn unsupported_identity_rule_is_unprocessable() {
        let result = result(
            PublishIngestionStatus::Rejected,
            "LB_UNSUPPORTED_IDENTITY_RULE",
        );
        let response = ingestion_http_response(&result);
        assert_eq!(response.status_code, 422);
        assert_eq!(response.status_text, "Unprocessable Entity");
    }
}
