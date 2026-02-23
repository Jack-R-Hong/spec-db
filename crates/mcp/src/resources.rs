use std::path::PathBuf;
use std::sync::Arc;

use rmcp::model::{AnnotateAble, RawResource, Resource, ResourceContents};
use serde_json::{Value, json};
use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{CausalGraph, SpecDbError, SpecId};
use spec_db_search::SearchIndex;

#[derive(Debug, PartialEq, Eq)]
pub enum ResourceUri {
    Spec(String),
    GraphOverview,
    GraphNode(String),
}

#[derive(Clone)]
pub struct ResourceHandler {
    pub tantivy_dir: PathBuf,
    pub fjall_dir: PathBuf,
}

impl ResourceHandler {
    pub fn list_resources(&self) -> Vec<Resource> {
        vec![
            RawResource::new("spec://{id}", "spec-by-id").no_annotation(),
            RawResource::new("graph://overview", "graph-overview").no_annotation(),
            RawResource::new("graph://node/{id}", "graph-node").no_annotation(),
        ]
    }

    pub fn read_resource(&self, uri: &str) -> Result<ResourceContents, SpecDbError> {
        let value = match parse_resource_uri(uri) {
            Some(ResourceUri::Spec(id)) => self.read_spec(&id)?,
            Some(ResourceUri::GraphOverview) => self.read_graph_overview()?,
            Some(ResourceUri::GraphNode(id)) => self.read_graph_node(&id)?,
            None => {
                return Err(SpecDbError::ConfigError(format!("resource not found: {uri}")));
            }
        };

        Ok(ResourceContents::text(value.to_string(), uri.to_owned()))
    }

    fn read_spec(&self, raw_id: &str) -> Result<Value, SpecDbError> {
        let id = SpecId::try_new(raw_id.to_owned())?;
        let search = SearchIndex::open_or_create(&self.tantivy_dir)?;
        let spec = search.get_spec(&id)?;
        Ok(json!({ "spec": spec }))
    }

    fn read_graph_overview(&self) -> Result<Value, SpecDbError> {
        let store = Arc::new(FjallStore::open(&self.fjall_dir)?);
        let graph = CausalEngine::from_store(store.clone())?;
        let nodes = store.iter_nodes()?;
        let edges = store.iter_edges()?;

        let mut disconnected_clusters = Vec::new();
        for node in nodes {
            let outbound = graph.edges_from(&node.id)?;
            let inbound = graph.edges_to(&node.id)?;
            if outbound.is_empty() && inbound.is_empty() {
                disconnected_clusters.push(node.id.to_string());
            }
        }

        Ok(json!({
            "total_specs": store.iter_nodes()?.len(),
            "total_edges": edges.len(),
            "disconnected_clusters": disconnected_clusters,
        }))
    }

    fn read_graph_node(&self, raw_id: &str) -> Result<Value, SpecDbError> {
        let id = SpecId::try_new(raw_id.to_owned())?;
        let graph = CausalEngine::from_store(Arc::new(FjallStore::open(&self.fjall_dir)?))?;
        if graph.get_node(&id)?.is_none() {
            return Err(SpecDbError::GraphError(format!("node not found: {id}")));
        }

        let inbound = graph
            .edges_to(&id)?
            .into_iter()
            .map(|edge| {
                json!({
                    "from": edge.source.to_string(),
                    "to": edge.target.to_string(),
                    "type": "depends_on",
                })
            })
            .collect::<Vec<_>>();

        let outbound = graph
            .edges_from(&id)?
            .into_iter()
            .map(|edge| {
                json!({
                    "from": edge.source.to_string(),
                    "to": edge.target.to_string(),
                    "type": "depends_on",
                })
            })
            .collect::<Vec<_>>();

        Ok(json!({
            "node": id.to_string(),
            "inbound": inbound,
            "outbound": outbound,
        }))
    }
}

pub fn parse_resource_uri(uri: &str) -> Option<ResourceUri> {
    if uri == "graph://overview" {
        return Some(ResourceUri::GraphOverview);
    }
    if let Some(id) = uri.strip_prefix("spec://") {
        return Some(ResourceUri::Spec(id.to_owned()));
    }
    if let Some(id) = uri.strip_prefix("graph://node/") {
        return Some(ResourceUri::GraphNode(id.to_owned()));
    }
    None
}
