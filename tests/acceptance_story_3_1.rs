//! Acceptance tests for Story 3.1: Spec Format Definition & Markdown/YAML Parsing

use spec_db_core::{SpecDbError, SpecId};
use spec_db_ingest::parse_spec;

fn _spec_id(value: &str) -> SpecId {
    SpecId::try_new(value).unwrap()
}

/// AC1: Valid frontmatter fields are extracted into SpecDoc.
#[test]
fn ac1_valid_frontmatter_extracts_to_spec_doc() {
    let markdown = r#"---
id: "spec::auth::login"
title: "Login Flow"
version: 3
tags: ["auth", "security"]
depends_on: ["spec::auth::token"]
owner: "backend"
created: "2026-01-15"
---
# Login Flow
User authentication via credentials.
"#;

    let doc = parse_spec(markdown).unwrap();

    assert_eq!(doc.id, _spec_id("spec::auth::login"));
    assert_eq!(doc.title, "Login Flow");
    assert_eq!(doc.version, 3);
    assert_eq!(doc.tags, vec!["auth", "security"]);
    assert_eq!(doc.depends_on, vec![_spec_id("spec::auth::token")]);
    assert_eq!(doc.owner.as_deref(), Some("backend"));
    assert_eq!(doc.created, "2026-01-15");
    assert!(doc.body.contains("User authentication"));
}

/// AC2: The canonical markdown/yaml format maps to the expected SpecDoc fields.
#[test]
fn ac2_specific_format_example_maps_correctly() {
    let markdown = r#"---
id: "spec::payments::refunds"
title: "Refund Processing"
version: 11
tags: ["payments", "compliance"]
depends_on: ["spec::payments::ledger", "spec::auth::login"]
owner: "finance-platform"
created: "2026-02-01"
---
# Refund Processing
Handle partial and full refunds with audit logging.
"#;

    let doc = parse_spec(markdown).unwrap();

    assert_eq!(doc.id.as_ref(), "spec::payments::refunds");
    assert_eq!(doc.title, "Refund Processing");
    assert_eq!(doc.version, 11);
    assert_eq!(doc.tags, vec!["payments", "compliance"]);
    assert_eq!(
        doc.depends_on,
        vec![_spec_id("spec::payments::ledger"), _spec_id("spec::auth::login"),]
    );
    assert_eq!(doc.owner.as_deref(), Some("finance-platform"));
    assert_eq!(doc.created, "2026-02-01");
    assert!(doc.body.starts_with("# Refund Processing"));
}

/// AC3: Invalid SpecId patterns return IngestError.
#[test]
fn ac3_invalid_spec_id_pattern_returns_ingest_error() {
    let markdown = r#"---
id: "spec::Auth::Invalid"
title: "Bad ID"
version: 1
tags: ["auth"]
depends_on: []
owner: "backend"
created: "2026-02-23"
---
# Bad ID
body
"#;

    let err = parse_spec(markdown).unwrap_err();

    match err {
        SpecDbError::IngestError(message) => {
            assert!(message.contains("invalid SpecId"));
        }
        other => panic!("expected IngestError, got {other:?}"),
    }
}

/// AC4: Missing required fields return IngestError.
#[test]
fn ac4_missing_required_fields_return_ingest_error() {
    let markdown = r#"---
id: "spec::auth::missing-title"
title: ""
version: 1
tags: ["auth"]
depends_on: []
owner: "backend"
created: "2026-02-23"
---
# Missing Title
body
"#;

    let err = parse_spec(markdown).unwrap_err();

    match err {
        SpecDbError::IngestError(message) => {
            assert!(message.contains("missing required field: title"));
        }
        other => panic!("expected IngestError, got {other:?}"),
    }
}

/// AC5: Markdown without frontmatter returns IngestError.
#[test]
fn ac5_no_frontmatter_returns_ingest_error() {
    let markdown = "# No frontmatter\n\njust markdown";

    let err = parse_spec(markdown).unwrap_err();

    match err {
        SpecDbError::IngestError(message) => {
            assert!(message.contains("missing frontmatter"));
        }
        other => panic!("expected IngestError, got {other:?}"),
    }
}
