use pulldown_cmark::{Event, MetadataBlockKind, Options, Parser, Tag, TagEnd};
use serde::Deserialize;
use spec_db_core::{SpecDbError, SpecDoc, SpecId};

use crate::validate::{validate_frontmatter, validate_spec_id};

#[derive(Debug, Deserialize)]
pub struct RawFrontmatter {
    pub id: String,
    pub title: String,
    pub version: u32,
    pub tags: Option<Vec<String>>,
    pub depends_on: Option<Vec<String>>,
    pub owner: Option<String>,
    pub created: Option<String>,
}

pub fn parse_spec(markdown: &str) -> Result<SpecDoc, SpecDbError> {
    let _span = tracing::info_span!("spec_db.ingest.parse").entered();

    let mut in_yaml_block = false;
    let mut found_yaml_block = false;
    let mut yaml_text = String::new();

    let parser = Parser::new_ext(markdown, Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    for event in parser {
        match event {
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                in_yaml_block = true;
                found_yaml_block = true;
            }
            Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                in_yaml_block = false;
            }
            Event::Text(text) if in_yaml_block => {
                yaml_text.push_str(&text);
            }
            _ => {}
        }
    }

    if !found_yaml_block {
        return Err(SpecDbError::IngestError("missing frontmatter".to_string()));
    }

    let body_text = extract_body_text(markdown)?;

    let raw = serde_yml::from_str::<RawFrontmatter>(&yaml_text)
        .map_err(|err| SpecDbError::IngestError(err.to_string()))?;

    validate_frontmatter(&raw)?;
    let id = validate_spec_id(&raw.id)?;

    let depends_on = raw
        .depends_on
        .map(|deps| deps.into_iter().map(SpecId::try_new).collect())
        .transpose()?
        .unwrap_or_default();

    Ok(SpecDoc {
        id,
        title: raw.title,
        version: raw.version,
        tags: raw.tags.unwrap_or_default(),
        depends_on,
        owner: raw.owner,
        created: raw.created.unwrap_or_default(),
        body: body_text,
    })
}

fn extract_body_text(markdown: &str) -> Result<String, SpecDbError> {
    let mut delimiter_count = 0usize;
    let mut offset = 0usize;

    for line in markdown.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.trim() == "---" {
            delimiter_count += 1;
            if delimiter_count == 2 {
                let body_start = offset + line.len();
                return Ok(markdown[body_start..].trim().to_string());
            }
        }
        offset += line.len();
    }

    if delimiter_count == 1 {
        return Ok(String::new());
    }

    Err(SpecDbError::IngestError("missing frontmatter".to_string()))
}
