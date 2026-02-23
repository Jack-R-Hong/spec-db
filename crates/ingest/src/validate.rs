use spec_db_core::{SpecDbError, SpecId};

use crate::parser::RawFrontmatter;

pub fn validate_frontmatter(raw: &RawFrontmatter) -> Result<(), SpecDbError> {
    let _span = tracing::info_span!("spec_db.ingest.validate").entered();

    if raw.id.trim().is_empty() {
        return Err(SpecDbError::IngestError("missing required field: id".to_string()));
    }

    if raw.title.trim().is_empty() {
        return Err(SpecDbError::IngestError("missing required field: title".to_string()));
    }

    Ok(())
}

pub fn validate_spec_id(id: &str) -> Result<SpecId, SpecDbError> {
    let _span = tracing::info_span!("spec_db.ingest.validate").entered();
    SpecId::try_new(id)
}
