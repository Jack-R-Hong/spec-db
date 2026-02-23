pub mod indexer;
pub mod query;
pub mod schema;

use tantivy::query::Query;

use spec_db_core::{SearchEngine, SpecDbError, SpecDoc, SpecId};

pub use indexer::SearchIndex;
pub use query::SearchHit;
pub use schema::SearchSchemaFields;

impl SearchIndex {
    pub fn get_spec(&self, id: &SpecId) -> Result<Option<SpecDoc>, SpecDbError> {
        let _span = tracing::info_span!("spec_db.search.get_spec", spec_id = %id).entered();
        query::get_spec_by_id(&self.searcher(), self.fields(), id)
    }
}

impl SearchEngine for SearchIndex {
    fn index_spec(&mut self, doc: &SpecDoc) -> Result<(), SpecDbError> {
        self.add_doc(doc)?;
        self.commit()
    }

    fn remove_spec(&mut self, id: &SpecId) -> Result<(), SpecDbError> {
        self.remove_doc(id)?;
        self.commit()
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SpecId>, SpecDbError> {
        let _span = tracing::info_span!(
            "spec_db.search.query",
            query_len = query.len(),
            limit,
            tag_count = 0usize
        )
        .entered();

        let text_query = query::build_text_query(self.index(), self.fields(), query)?;
        let hits = query::execute_search(&self.searcher(), self.fields(), &*text_query, limit)?;
        tracing::info!(hit_count = hits.len(), "search completed");
        Ok(hits.into_iter().map(|hit| hit.id).collect())
    }

    fn search_with_tags(
        &self,
        query: &str,
        tags: &[String],
        limit: usize,
    ) -> Result<Vec<SpecId>, SpecDbError> {
        let _span = tracing::info_span!(
            "spec_db.search.query",
            query_len = query.len(),
            limit,
            tag_count = tags.len()
        )
        .entered();

        let text_query = query::build_text_query(self.index(), self.fields(), query)?;
        let combined_query: Box<dyn Query> = if tags.is_empty() {
            text_query
        } else {
            Box::new(query::combine_text_with_tags(text_query, self.fields(), tags))
        };

        let hits = query::execute_search(&self.searcher(), self.fields(), &*combined_query, limit)?;
        tracing::info!(hit_count = hits.len(), "search completed");
        Ok(hits.into_iter().map(|hit| hit.id).collect())
    }
}
