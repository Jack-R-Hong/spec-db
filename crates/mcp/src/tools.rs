use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::{Value, json};
use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{
    CausalEdge, CausalGraph, EdgeOrigin, EdgeType, SpecDbError, SpecId, TrustLevel,
};
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

#[derive(Deserialize)]
pub struct AddCausalLinkInput {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub edge_type: Option<String>,
}

#[derive(Deserialize)]
pub struct EdgeActionInput {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub edge_type: Option<String>,
}

#[derive(Clone)]
pub struct ToolHandler {
    pub repo_path: PathBuf,
    pub specs_root: String,
    pub tantivy_dir: PathBuf,
    pub fjall_dir: PathBuf,
    pub ai_default_trust: f64,
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

    pub fn add_causal_link(&self, input: AddCausalLinkInput) -> Result<Value, SpecDbError> {
        let source = parse_spec_id("source", &input.source)?;
        let target = parse_spec_id("target", &input.target)?;

        if source == target {
            return Err(tool_error(
                "validation_error",
                "Self-referencing edges are not allowed",
                Value::Null,
            ));
        }

        let edge_type_raw = input.edge_type.unwrap_or_else(|| "depends_on".to_owned());
        let edge_type = EdgeType::from_str(edge_type_raw.as_str()).map_err(|_| {
            tool_error(
                "validation_error",
                "Invalid edge type",
                json!({
                    "edge_type": edge_type_raw,
                    "allowed": ["depends_on", "constrains", "implements"],
                }),
            )
        })?;

        let mut graph = self.open_graph()?;
        if graph.get_node(&source)?.is_none() {
            return Err(tool_error(
                "not_found",
                "Source spec not found",
                json!({ "id": source.to_string() }),
            ));
        }

        if graph.get_node(&target)?.is_none() {
            return Err(tool_error(
                "not_found",
                "Target spec not found",
                json!({ "id": target.to_string() }),
            ));
        }

        if let Some(existing) =
            graph.edges_from(&source)?.into_iter().find(|edge| edge.target == target)
        {
            return Err(tool_error(
                "conflict",
                "Edge already exists",
                json!({
                    "source": source.to_string(),
                    "target": target.to_string(),
                    "existing_edge_type": existing.edge_type.to_string(),
                    "edge_type": edge_type.to_string(),
                }),
            ));
        }

        if let Some(cycle) = graph.validate_no_cycle(&source, &target)? {
            return Err(tool_error(
                "csm_validation_failed",
                "Proposed edge creates a causal cycle",
                json!({ "cycle": cycle }),
            ));
        }

        let edge = CausalEdge {
            source: source.clone(),
            target: target.clone(),
            edge_type,
            trust: TrustLevel::new(self.ai_default_trust),
            origin: EdgeOrigin::Ai,
            created_at: Some(now_iso8601()),
        };

        graph.add_edge(edge.clone())?;

        let lattice_dir = self.repo_path.join(".lattice");
        let all_edges = graph.all_edges()?;
        spec_db_causal::export::export_ai_edges(&all_edges, &lattice_dir)?;

        Ok(json!({
            "status": "ok",
            "message": "causal link added",
            "edge": edge_to_json(edge),
        }))
    }

    pub fn promote_edge(&self, input: EdgeActionInput) -> Result<Value, SpecDbError> {
        let source = parse_spec_id("source", &input.source)?;
        let target = parse_spec_id("target", &input.target)?;
        let edge_type_raw = input.edge_type.unwrap_or_else(|| "depends_on".to_owned());
        let _edge_type = EdgeType::from_str(edge_type_raw.as_str()).map_err(|_| {
            tool_error(
                "validation_error",
                "Invalid edge type",
                json!({ "edge_type": edge_type_raw }),
            )
        })?;

        let mut graph = self.open_graph()?;

        let existing = graph.edges_from(&source)?.into_iter().find(|e| e.target == target);

        let edge = existing.ok_or_else(|| {
            tool_error(
                "not_found",
                "Edge not found",
                json!({ "source": source.to_string(), "target": target.to_string(), "edge_type": edge_type_raw }),
            )
        })?;

        if edge.origin == EdgeOrigin::Human {
            return Err(tool_error(
                "validation_error",
                "Edge is already human-curated",
                json!({ "source": source.to_string(), "target": target.to_string() }),
            ));
        }

        graph.update_edge_origin(&source, &target, EdgeOrigin::Human, TrustLevel::human())?;

        let lattice_dir = self.repo_path.join(".lattice");
        let all_edges = graph.all_edges()?;
        spec_db_causal::export::export_ai_edges(&all_edges, &lattice_dir)?;

        Ok(json!({
            "status": "ok",
            "message": "edge promoted to human-curated",
            "edge": {
                "from": source.to_string(),
                "to": target.to_string(),
                "edge_type": edge.edge_type.to_string(),
                "trust": 1.0,
                "origin": "human",
            }
        }))
    }

    pub fn reject_edge(&self, input: EdgeActionInput) -> Result<Value, SpecDbError> {
        let source = parse_spec_id("source", &input.source)?;
        let target = parse_spec_id("target", &input.target)?;
        let edge_type_raw = input.edge_type.unwrap_or_else(|| "depends_on".to_owned());
        let _edge_type = EdgeType::from_str(edge_type_raw.as_str()).map_err(|_| {
            tool_error(
                "validation_error",
                "Invalid edge type",
                json!({ "edge_type": edge_type_raw }),
            )
        })?;

        let mut graph = self.open_graph()?;

        let existing = graph.edges_from(&source)?.into_iter().find(|e| e.target == target);

        if existing.is_none() {
            return Err(tool_error(
                "not_found",
                "Edge not found",
                json!({ "source": source.to_string(), "target": target.to_string(), "edge_type": edge_type_raw }),
            ));
        }

        graph.remove_edge(&source, &target)?;

        let lattice_dir = self.repo_path.join(".lattice");
        let all_edges = graph.all_edges()?;
        spec_db_causal::export::export_ai_edges(&all_edges, &lattice_dir)?;

        Ok(json!({
            "status": "ok",
            "message": "edge rejected and removed",
            "edge": {
                "from": source.to_string(),
                "to": target.to_string(),
                "edge_type": edge_type_raw,
            }
        }))
    }

    fn open_graph(&self) -> Result<CausalEngine, SpecDbError> {
        let store = Arc::new(FjallStore::open(&self.fjall_dir)?);
        CausalEngine::from_store(store)
    }
}

fn parse_spec_id(field: &str, raw: &str) -> Result<SpecId, SpecDbError> {
    SpecId::try_new(raw).map_err(|_| {
        tool_error(
            "validation_error",
            "Invalid spec id",
            json!({
                "field": field,
                "id": raw,
            }),
        )
    })
}

fn now_iso8601() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let (year, month, day) = epoch_days_to_ymd(days as i64);
    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

fn epoch_days_to_ymd(mut days: i64) -> (i32, u32, u32) {
    days += 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = (days - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m, d)
}

fn tool_error(error_type: &str, message: &str, context: Value) -> SpecDbError {
    let payload = json!({
        "error_type": error_type,
        "message": message,
        "context": context,
    });
    SpecDbError::IngestError(format!("mcp_error::{}", payload))
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
        "type": edge.edge_type.to_string(),
        "edge_type": edge.edge_type.to_string(),
        "trust": edge.trust.value(),
        "origin": edge.origin.to_string(),
    })
}
