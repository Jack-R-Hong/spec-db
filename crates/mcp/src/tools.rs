use std::path::PathBuf;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::{Value, json};
use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{CausalGraph, SpecDbError, SpecId};
use spec_db_ingest::{GitSync, IngestPipeline, StorePaths};
use spec_db_router::QueryRouter;
use spec_db_search::{SearchIndex, query};

#[derive(Deserialize)]
pub struct SearchSpecsInput {
    pub query: String,
    pub limit: Option<usize>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct GetSpecInput {
    pub id: String,
}

#[derive(Deserialize)]
pub struct TraceImpactInput {
    pub id: String,
    pub depth: Option<usize>,
}

#[derive(Deserialize)]
pub struct FindDependenciesInput {
    pub id: String,
}

#[derive(Deserialize)]
pub struct QueryInput {
    pub natural_language: String,
}

#[derive(Deserialize)]
pub struct AddSpecInput {
    pub markdown: String,
}

#[derive(Deserialize)]
pub struct SyncInput {
    pub mode: Option<String>,
}

#[derive(Clone)]
pub struct ToolHandler {
    pub repo_path: PathBuf,
    pub specs_root: String,
    pub tantivy_dir: PathBuf,
    pub fjall_dir: PathBuf,
}

impl ToolHandler {
    pub fn search_specs(&self, input: SearchSpecsInput) -> Result<Value, SpecDbError> {
        let limit = input.limit.unwrap_or(10);
        let search = SearchIndex::open_or_create(&self.tantivy_dir)?;
        let hits = if let Some(tags) = input.tags {
            execute_search_with_optional_tags(&search, &input.query, &tags, limit)?
        } else {
            execute_search_with_optional_tags(&search, &input.query, &[], limit)?
        };

        Ok(json!({
            "results": hits,
        }))
    }

    pub fn get_spec(&self, input: GetSpecInput) -> Result<Value, SpecDbError> {
        let id = SpecId::try_new(input.id)?;
        let search = SearchIndex::open_or_create(&self.tantivy_dir)?;
        let spec = search.get_spec(&id)?;
        Ok(json!({ "spec": spec }))
    }

    pub fn trace_impact(&self, input: TraceImpactInput) -> Result<Value, SpecDbError> {
        let id = SpecId::try_new(input.id)?;
        let graph = self.open_graph()?;
        let _ = graph.trace_impact(&id, input.depth)?;
        let mut edges = graph.edges_from(&id)?;
        edges.extend(graph.edges_to(&id)?);
        Ok(json!({
            "node": id.to_string(),
            "edges": edges.into_iter().map(edge_to_json).collect::<Vec<_>>(),
        }))
    }

    pub fn find_dependencies(&self, input: FindDependenciesInput) -> Result<Value, SpecDbError> {
        let id = SpecId::try_new(input.id)?;
        let graph = self.open_graph()?;
        let _ = graph.find_dependencies(&id, None)?;
        let mut edges = graph.edges_from(&id)?;
        edges.extend(graph.edges_to(&id)?);
        Ok(json!({
            "node": id.to_string(),
            "edges": edges.into_iter().map(edge_to_json).collect::<Vec<_>>(),
        }))
    }

    pub fn query(&self, input: QueryInput) -> Result<Value, SpecDbError> {
        let search = SearchIndex::open_or_create(&self.tantivy_dir)?;
        let graph = self.open_graph()?;
        let router = QueryRouter::new(search, graph);
        let result = router.query(&input.natural_language)?;
        serde_json::to_value(result)
            .map_err(|e| SpecDbError::IngestError(format!("failed to serialize query result: {e}")))
    }

    pub fn add_spec(&self, input: AddSpecInput) -> Result<Value, SpecDbError> {
        let search = SearchIndex::open_or_create(&self.tantivy_dir)?;
        let fjall_store = Arc::new(FjallStore::open(&self.fjall_dir)?);
        let engine = CausalEngine::from_store(fjall_store.clone())?;
        let mut pipeline = IngestPipeline::new(search, engine);
        let spec_id = pipeline.add_spec(&input.markdown)?;
        let doc_count = fjall_store.iter_nodes()?.len();
        fjall_store.set_doc_count(doc_count)?;

        Ok(json!({
            "status": "ok",
            "message": "spec ingested",
            "details": {
                "id": spec_id.to_string(),
                "doc_count": doc_count,
            }
        }))
    }

    pub fn sync(&self, input: SyncInput) -> Result<Value, SpecDbError> {
        let git_sync = GitSync::new(
            self.repo_path.clone(),
            self.specs_root.clone(),
            StorePaths { tantivy_dir: self.tantivy_dir.clone(), fjall_dir: self.fjall_dir.clone() },
        );

        let mode = input.mode.unwrap_or_else(|| "incremental".to_owned());
        let report =
            if mode == "full" { git_sync.full_rebuild()? } else { git_sync.incremental_sync()? };

        Ok(json!({
            "status": "ok",
            "message": "sync completed",
            "details": {
                "mode": mode,
                "specs_ingested": report.specs_ingested,
                "head_sha": report.head_sha,
            }
        }))
    }

    fn open_graph(&self) -> Result<CausalEngine, SpecDbError> {
        let store = Arc::new(FjallStore::open(&self.fjall_dir)?);
        CausalEngine::from_store(store)
    }
}

fn execute_search_with_optional_tags(
    search: &SearchIndex,
    query_text: &str,
    tags: &[String],
    limit: usize,
) -> Result<Vec<Value>, SpecDbError> {
    let text_query = query::build_text_query(search.index(), search.fields(), query_text)?;
    let merged_query = if tags.is_empty() {
        text_query
    } else {
        Box::new(query::combine_text_with_tags(text_query, search.fields(), tags))
    };

    let hits = query::execute_search(&search.searcher(), search.fields(), &*merged_query, limit)?;
    Ok(hits
        .into_iter()
        .map(|hit| {
            json!({
                "id": hit.id.to_string(),
                "title": hit.title,
                "score": hit.score,
                "snippet": hit.snippet,
            })
        })
        .collect())
}

fn edge_to_json(edge: spec_db_core::CausalEdge) -> Value {
    json!({
        "from": edge.source.to_string(),
        "to": edge.target.to_string(),
        "type": "depends_on",
    })
}
