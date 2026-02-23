use std::path::{Path, PathBuf};

use serde_json::json;
use spec_db_core::{SpecDbError, SpecDoc, SpecId};
use tantivy::doc;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, Searcher, Term};

use crate::schema::SearchSchemaFields;

const INDEX_WRITER_HEAP_SIZE_BYTES: usize = 50_000_000;
const SEARCH_METADATA_FILE: &str = "sync_metadata.json";

pub struct SearchIndex {
    index_dir: PathBuf,
    index: Index,
    reader: IndexReader,
    writer: IndexWriter,
    fields: SearchSchemaFields,
}

impl SearchIndex {
    pub fn open_or_create(index_dir: &Path) -> Result<Self, SpecDbError> {
        let fields = crate::schema::build_schema();
        std::fs::create_dir_all(index_dir)
            .map_err(|e| SpecDbError::SearchError(format!("failed to create index dir: {e}")))?;

        let meta_path = index_dir.join("meta.json");
        let index = if meta_path.exists() {
            Index::open_in_dir(index_dir).map_err(|e| SpecDbError::SearchError(e.to_string()))?
        } else {
            Index::create_in_dir(index_dir, fields.schema.clone())
                .map_err(|e| SpecDbError::SearchError(e.to_string()))?
        };

        let writer = index
            .writer(INDEX_WRITER_HEAP_SIZE_BYTES)
            .map_err(|e| SpecDbError::SearchError(e.to_string()))?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| SpecDbError::SearchError(e.to_string()))?;

        Ok(Self { index_dir: index_dir.to_path_buf(), index, reader, writer, fields })
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn reader(&self) -> &IndexReader {
        &self.reader
    }

    pub fn searcher(&self) -> Searcher {
        self.reader.searcher()
    }

    pub fn fields(&self) -> &SearchSchemaFields {
        &self.fields
    }

    pub fn add_doc(&mut self, doc: &SpecDoc) -> Result<(), SpecDbError> {
        let _span = tracing::info_span!("spec_db.search.add_doc", spec_id = %doc.id).entered();
        self.remove_doc(&doc.id)?;

        let depends_on: Vec<&str> = doc.depends_on.iter().map(AsRef::as_ref).collect();
        let meta = json!({
            "version": doc.version,
            "depends_on": depends_on,
            "owner": doc.owner,
            "created": doc.created,
        });

        let mut tantivy_doc = tantivy::doc!(
            self.fields.id => doc.id.as_ref(),
            self.fields.title => doc.title.as_str(),
            self.fields.body => doc.body.as_str(),
            self.fields.meta => meta,
        );

        for tag in &doc.tags {
            tantivy_doc.add_text(self.fields.tags, tag);
        }

        self.writer
            .add_document(tantivy_doc)
            .map_err(|e| SpecDbError::SearchError(e.to_string()))?;
        Ok(())
    }

    pub fn remove_doc(&mut self, id: &SpecId) -> Result<(), SpecDbError> {
        let _span = tracing::info_span!("spec_db.search.remove_doc", spec_id = %id).entered();
        let id_term = Term::from_field_text(self.fields.id, id.as_ref());
        self.writer.delete_term(id_term);
        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), SpecDbError> {
        let _span = tracing::info_span!("spec_db.search.commit").entered();
        self.writer.commit().map_err(|e| SpecDbError::SearchError(e.to_string()))?;
        self.reader.reload().map_err(|e| SpecDbError::SearchError(e.to_string()))?;
        Ok(())
    }

    pub fn doc_count(&self) -> Result<u64, SpecDbError> {
        let searcher = self.reader.searcher();
        Ok(searcher.num_docs())
    }

    pub fn sync_metadata(&self) -> Result<Option<(String, usize)>, SpecDbError> {
        let metadata_path = self.index_dir.join(SEARCH_METADATA_FILE);
        if !metadata_path.exists() {
            return Ok(None);
        }

        let metadata_bytes = std::fs::read(&metadata_path).map_err(|e| {
            SpecDbError::SearchError(format!(
                "failed to read sync metadata at {}: {e}",
                metadata_path.display()
            ))
        })?;

        let metadata: serde_json::Value = serde_json::from_slice(&metadata_bytes).map_err(|e| {
            SpecDbError::SearchError(format!(
                "failed to parse sync metadata at {}: {e}",
                metadata_path.display()
            ))
        })?;

        let sha = metadata
            .get("last_sync_sha")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                SpecDbError::SearchError(format!(
                    "sync metadata missing string last_sync_sha at {}",
                    metadata_path.display()
                ))
            })?
            .to_string();

        let doc_count_u64 =
            metadata.get("doc_count").and_then(serde_json::Value::as_u64).ok_or_else(|| {
                SpecDbError::SearchError(format!(
                    "sync metadata missing numeric doc_count at {}",
                    metadata_path.display()
                ))
            })?;

        let doc_count = usize::try_from(doc_count_u64).map_err(|_| {
            SpecDbError::SearchError(format!(
                "sync metadata doc_count too large at {}",
                metadata_path.display()
            ))
        })?;

        Ok(Some((sha, doc_count)))
    }
}
