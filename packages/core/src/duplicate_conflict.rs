use lingonberry_protocol::{to_canonical_json, JsonValue};

pub const DUPLICATE_CONFLICT_CONTRACT_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateConflictClassification {
    New,
    ExactDuplicate,
    CanonicalIdConflict,
    CarrierIdentityConflict,
    CrossIdentityConflict,
}

impl DuplicateConflictClassification {
    pub fn code(self) -> &'static str {
        match self {
            Self::New => "LB_OBJECT_NEW",
            Self::ExactDuplicate => "LB_OBJECT_DUPLICATE",
            Self::CanonicalIdConflict
            | Self::CarrierIdentityConflict
            | Self::CrossIdentityConflict => "LB_OBJECT_CONFLICT",
        }
    }

    pub fn is_conflict(self) -> bool {
        matches!(
            self,
            Self::CanonicalIdConflict
                | Self::CarrierIdentityConflict
                | Self::CrossIdentityConflict
        )
    }
}

pub struct ExistingObjectIdentity<'a> {
    pub canonical_id: &'a str,
    pub carrier_identity: &'a str,
    pub object: &'a JsonValue,
}

pub struct IncomingObjectIdentity<'a> {
    pub canonical_id: &'a str,
    pub carrier_identity: &'a str,
    pub canonical_json: &'a str,
}

pub fn classify_duplicate_or_conflict(
    existing_by_canonical_id: Option<ExistingObjectIdentity<'_>>,
    existing_by_carrier_identity: Option<ExistingObjectIdentity<'_>>,
    incoming: IncomingObjectIdentity<'_>,
) -> DuplicateConflictClassification {
    if let Some(existing) = existing_by_carrier_identity {
        if existing.canonical_id != incoming.canonical_id {
            return DuplicateConflictClassification::CrossIdentityConflict;
        }
        if to_canonical_json(existing.object) != incoming.canonical_json {
            return DuplicateConflictClassification::CarrierIdentityConflict;
        }
        return DuplicateConflictClassification::ExactDuplicate;
    }

    if let Some(existing) = existing_by_canonical_id {
        if existing.carrier_identity != incoming.carrier_identity {
            return DuplicateConflictClassification::CrossIdentityConflict;
        }
        if to_canonical_json(existing.object) != incoming.canonical_json {
            return DuplicateConflictClassification::CanonicalIdConflict;
        }
        return DuplicateConflictClassification::ExactDuplicate;
    }

    DuplicateConflictClassification::New
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingonberry_protocol::parse_json;

    fn object(text: &str) -> JsonValue {
        parse_json(&format!(r#"{{"id":"lb:obj:1","body":{{"text":"{text}"}}}}"#))
            .expect("fixture parses")
    }

    #[test]
    fn classifies_new_object() {
        let incoming_object = object("same");
        let canonical_json = to_canonical_json(&incoming_object);
        assert_eq!(
            classify_duplicate_or_conflict(
                None,
                None,
                IncomingObjectIdentity {
                    canonical_id: "lb:obj:1",
                    carrier_identity: "carrier:1",
                    canonical_json: &canonical_json,
                },
            ),
            DuplicateConflictClassification::New
        );
    }

    #[test]
    fn classifies_exact_duplicate() {
        let existing = object("same");
        let canonical_json = to_canonical_json(&existing);
        assert_eq!(
            classify_duplicate_or_conflict(
                None,
                Some(ExistingObjectIdentity {
                    canonical_id: "lb:obj:1",
                    carrier_identity: "carrier:1",
                    object: &existing,
                }),
                IncomingObjectIdentity {
                    canonical_id: "lb:obj:1",
                    carrier_identity: "carrier:1",
                    canonical_json: &canonical_json,
                },
            ),
            DuplicateConflictClassification::ExactDuplicate
        );
    }

    #[test]
    fn classifies_same_canonical_id_with_different_content_as_conflict() {
        let existing = object("old");
        let incoming = object("new");
        let canonical_json = to_canonical_json(&incoming);
        assert_eq!(
            classify_duplicate_or_conflict(
                Some(ExistingObjectIdentity {
                    canonical_id: "lb:obj:1",
                    carrier_identity: "carrier:1",
                    object: &existing,
                }),
                None,
                IncomingObjectIdentity {
                    canonical_id: "lb:obj:1",
                    carrier_identity: "carrier:1",
                    canonical_json: &canonical_json,
                },
            ),
            DuplicateConflictClassification::CanonicalIdConflict
        );
    }

    #[test]
    fn classifies_same_carrier_identity_with_different_content_as_conflict() {
        let existing = object("old");
        let incoming = object("new");
        let canonical_json = to_canonical_json(&incoming);
        assert_eq!(
            classify_duplicate_or_conflict(
                None,
                Some(ExistingObjectIdentity {
                    canonical_id: "lb:obj:1",
                    carrier_identity: "carrier:1",
                    object: &existing,
                }),
                IncomingObjectIdentity {
                    canonical_id: "lb:obj:1",
                    carrier_identity: "carrier:1",
                    canonical_json: &canonical_json,
                },
            ),
            DuplicateConflictClassification::CarrierIdentityConflict
        );
    }

    #[test]
    fn rejects_cross_identity_aliasing_even_when_content_matches() {
        let existing = object("same");
        let canonical_json = to_canonical_json(&existing);
        assert_eq!(
            classify_duplicate_or_conflict(
                None,
                Some(ExistingObjectIdentity {
                    canonical_id: "lb:obj:other",
                    carrier_identity: "carrier:1",
                    object: &existing,
                }),
                IncomingObjectIdentity {
                    canonical_id: "lb:obj:1",
                    carrier_identity: "carrier:1",
                    canonical_json: &canonical_json,
                },
            ),
            DuplicateConflictClassification::CrossIdentityConflict
        );
    }
}
