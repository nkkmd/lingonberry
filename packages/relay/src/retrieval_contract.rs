use lingonberry_core::{
    object_retrieval_result_json, ObjectRetrievalResult, ObjectRetrievalStatus,
};
use lingonberry_protocol::JsonValue;

#[derive(Debug, Clone)]
pub struct RetrievalHttpResponse {
    pub status_code: u16,
    pub status_text: &'static str,
    pub body: JsonValue,
}

pub fn retrieval_http_response(result: &ObjectRetrievalResult) -> RetrievalHttpResponse {
    let (status_code, status_text) = match result.status {
        ObjectRetrievalStatus::Found => (200, "OK"),
        ObjectRetrievalStatus::InvalidRequest => (400, "Bad Request"),
        ObjectRetrievalStatus::NotFound => (404, "Not Found"),
        ObjectRetrievalStatus::Failed => (500, "Internal Server Error"),
    };

    RetrievalHttpResponse {
        status_code,
        status_text,
        body: object_retrieval_result_json(result),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_retrieval_statuses_to_http_statuses() {
        let cases = [
            (ObjectRetrievalStatus::Found, 200, "OK"),
            (ObjectRetrievalStatus::InvalidRequest, 400, "Bad Request"),
            (ObjectRetrievalStatus::NotFound, 404, "Not Found"),
            (
                ObjectRetrievalStatus::Failed,
                500,
                "Internal Server Error",
            ),
        ];

        for (status, status_code, status_text) in cases {
            let result = ObjectRetrievalResult {
                contract_version: "1",
                status,
                code: "LB_TEST",
                message: "test".to_string(),
                record: None,
            };
            let response = retrieval_http_response(&result);
            assert_eq!(response.status_code, status_code);
            assert_eq!(response.status_text, status_text);
            let JsonValue::Object(body) = response.body else {
                panic!("response body must be an object");
            };
            assert_eq!(
                body.get("contractVersion"),
                Some(&JsonValue::String("1".to_string()))
            );
            assert_eq!(
                body.get("status"),
                Some(&JsonValue::String(status.as_str().to_string()))
            );
        }
    }
}
