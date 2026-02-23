use serde::Serialize;
use spec_db_core::SpecId;

#[derive(Debug, Clone)]
pub struct SearchHitData {
    pub id: String,
    pub title: String,
    pub score: f32,
    pub snippet: String,
}

impl From<SpecId> for SearchHitData {
    fn from(value: SpecId) -> Self {
        Self { id: value.to_string(), title: String::new(), score: 0.0, snippet: String::new() }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ComposedHit {
    pub id: String,
    pub title: String,
    pub score: f32,
    pub snippet: String,
    pub causal_edges: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComposedQueryResult {
    pub intent: String,
    pub search_results: Vec<ComposedHit>,
    pub causal_context: Vec<String>,
    pub message: String,
}

pub fn compose_search_only(hits: Vec<SearchHitData>) -> ComposedQueryResult {
    ComposedQueryResult {
        intent: "search".to_owned(),
        search_results: to_composed_hits(hits, &[]),
        causal_context: Vec::new(),
        message: "Search results returned.".to_owned(),
    }
}

pub fn compose_causal_only(ids: Vec<SpecId>) -> ComposedQueryResult {
    let causal_context = ids.into_iter().map(|id| id.to_string()).collect::<Vec<_>>();
    let message = if causal_context.is_empty() {
        "No causal context found for this query.".to_owned()
    } else {
        "Causal context returned.".to_owned()
    };

    ComposedQueryResult {
        intent: "causal".to_owned(),
        search_results: Vec::new(),
        causal_context,
        message,
    }
}

pub fn compose_hybrid(hits: Vec<SearchHitData>, causal_ids: Vec<SpecId>) -> ComposedQueryResult {
    let causal_context = causal_ids.into_iter().map(|id| id.to_string()).collect::<Vec<_>>();

    ComposedQueryResult {
        intent: "hybrid".to_owned(),
        search_results: to_composed_hits(hits, &causal_context),
        causal_context,
        message: "Combined search results and causal context.".to_owned(),
    }
}

pub fn compose_empty() -> ComposedQueryResult {
    ComposedQueryResult {
        intent: "empty".to_owned(),
        search_results: Vec::new(),
        causal_context: Vec::new(),
        message: "No search or causal results found for this query.".to_owned(),
    }
}

pub fn compose_fallback(causal_ids: Vec<SpecId>) -> ComposedQueryResult {
    let causal_context = causal_ids.into_iter().map(|id| id.to_string()).collect::<Vec<_>>();
    let message = if causal_context.is_empty() {
        "No direct search matches were found and no causal fallback context is available."
            .to_owned()
    } else {
        "No direct search matches were found; returning causal fallback context.".to_owned()
    };

    ComposedQueryResult {
        intent: "search".to_owned(),
        search_results: Vec::new(),
        causal_context,
        message,
    }
}

fn to_composed_hits(hits: Vec<SearchHitData>, causal_edges: &[String]) -> Vec<ComposedHit> {
    hits.into_iter()
        .map(|hit| ComposedHit {
            id: hit.id,
            title: hit.title,
            score: hit.score,
            snippet: hit.snippet,
            causal_edges: causal_edges.to_vec(),
        })
        .collect()
}
