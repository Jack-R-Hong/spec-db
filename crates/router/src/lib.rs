pub mod classifier;
pub mod composer;

use std::collections::HashSet;

use composer::{
    SearchHitData, compose_causal_only, compose_empty, compose_fallback, compose_hybrid,
    compose_search_only,
};
use spec_db_core::{CausalGraph, SearchEngine, SpecDbError, SpecId};

pub use classifier::{IntentClassifier, QueryIntent, classify};
pub use composer::{ComposedHit, ComposedQueryResult};

const DEFAULT_SEARCH_LIMIT: usize = 10;

pub struct QueryRouter<S: SearchEngine, C: CausalGraph> {
    search: S,
    graph: C,
}

impl<S: SearchEngine, C: CausalGraph> QueryRouter<S, C> {
    pub fn new(search: S, graph: C) -> Self {
        Self { search, graph }
    }

    pub fn query(&self, natural_language: &str) -> Result<ComposedQueryResult, SpecDbError> {
        let _span = tracing::info_span!("spec_db.router.query", query_len = natural_language.len())
            .entered();

        let intent = classify(natural_language);

        let result = match intent {
            QueryIntent::Search => self.execute_search(natural_language)?,
            QueryIntent::Causal => self.execute_causal(natural_language)?,
            QueryIntent::Hybrid => self.execute_hybrid(natural_language)?,
        };

        if result.search_results.is_empty() && result.causal_context.is_empty() {
            Ok(compose_empty())
        } else {
            Ok(result)
        }
    }

    fn execute_search(&self, query: &str) -> Result<ComposedQueryResult, SpecDbError> {
        let ids = self.search.search(query, DEFAULT_SEARCH_LIMIT)?;
        if ids.is_empty() {
            let fallback = self.causal_from_query_or_empty(query)?;
            if fallback.is_empty() {
                return Ok(compose_empty());
            }
            return Ok(compose_fallback(fallback));
        }

        Ok(compose_search_only(ids.into_iter().map(SearchHitData::from).collect()))
    }

    fn execute_causal(&self, query: &str) -> Result<ComposedQueryResult, SpecDbError> {
        let causal_ids = self.causal_from_query_or_empty(query)?;
        Ok(compose_causal_only(causal_ids))
    }

    fn execute_hybrid(&self, query: &str) -> Result<ComposedQueryResult, SpecDbError> {
        let search_ids = self.search.search(query, DEFAULT_SEARCH_LIMIT)?;
        let search_hits: Vec<SearchHitData> =
            search_ids.iter().cloned().map(SearchHitData::from).collect();

        let causal_ids = if let Some(target) = extract_spec_id(query) {
            self.collect_causal_context(&target)?
        } else {
            let mut combined = Vec::new();
            for id in &search_ids {
                combined.extend(self.collect_causal_context(id)?);
            }
            dedupe_spec_ids(combined)
        };

        if search_hits.is_empty() && causal_ids.is_empty() {
            Ok(compose_empty())
        } else if search_hits.is_empty() {
            Ok(compose_fallback(causal_ids))
        } else {
            Ok(compose_hybrid(search_hits, causal_ids))
        }
    }

    fn causal_from_query_or_empty(&self, query: &str) -> Result<Vec<SpecId>, SpecDbError> {
        match extract_spec_id(query) {
            Some(target) => self.collect_causal_context(&target),
            None => Ok(Vec::new()),
        }
    }

    fn collect_causal_context(&self, target: &SpecId) -> Result<Vec<SpecId>, SpecDbError> {
        let mut combined = self.graph.trace_impact(target, Some(2))?;
        combined.extend(self.graph.find_dependencies(target, Some(2))?);
        Ok(dedupe_spec_ids(combined))
    }
}

fn extract_spec_id(query: &str) -> Option<SpecId> {
    for token in query.split_whitespace() {
        let clean =
            token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != ':' && c != '-');
        if clean.contains("spec::") {
            let lowered = clean.to_ascii_lowercase();
            if let Ok(id) = SpecId::try_new(lowered) {
                return Some(id);
            }
        }
    }
    None
}

fn dedupe_spec_ids(ids: Vec<SpecId>) -> Vec<SpecId> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for id in ids {
        if seen.insert(id.as_ref().to_owned()) {
            out.push(id);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::extract_spec_id;

    #[test]
    fn extract_spec_id_from_sentence() {
        let id = extract_spec_id("what depends on spec::auth::login?").expect("expected id");
        assert_eq!(id.as_ref(), "spec::auth::login");
    }

    #[test]
    fn extract_spec_id_none_when_absent() {
        assert!(extract_spec_id("rate limiting api").is_none());
    }
}
