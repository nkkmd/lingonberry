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
