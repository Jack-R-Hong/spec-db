use crate::error::SpecDbError;
use crate::types::{CausalEdge, SpecDoc, SpecId, SpecNode};

pub trait SearchEngine {
    fn index_spec(&mut self, doc: &SpecDoc) -> Result<(), SpecDbError>;
    fn remove_spec(&mut self, id: &SpecId) -> Result<(), SpecDbError>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SpecId>, SpecDbError>;
    fn search_with_tags(
        &self,
        query: &str,
        tags: &[String],
        limit: usize,
    ) -> Result<Vec<SpecId>, SpecDbError>;
}

pub trait CausalGraph {
    fn upsert_node(&mut self, node: SpecNode) -> Result<(), SpecDbError>;
    fn remove_node(&mut self, id: &SpecId) -> Result<(), SpecDbError>;
    fn get_node(&self, id: &SpecId) -> Result<Option<SpecNode>, SpecDbError>;
    fn add_edge(&mut self, edge: CausalEdge) -> Result<(), SpecDbError>;
    fn remove_edge(&mut self, source: &SpecId, target: &SpecId) -> Result<(), SpecDbError>;
    fn trace_impact(&self, id: &SpecId, depth: Option<usize>) -> Result<Vec<SpecId>, SpecDbError>;
    fn find_dependencies(
        &self,
        id: &SpecId,
        depth: Option<usize>,
    ) -> Result<Vec<SpecId>, SpecDbError>;
    fn edges_from(&self, id: &SpecId) -> Result<Vec<CausalEdge>, SpecDbError>;
    fn edges_to(&self, id: &SpecId) -> Result<Vec<CausalEdge>, SpecDbError>;
}

pub trait SpecStore {
    fn put(&mut self, doc: SpecDoc) -> Result<(), SpecDbError>;
    fn get(&self, id: &SpecId) -> Result<Option<SpecDoc>, SpecDbError>;
    fn remove(&mut self, id: &SpecId) -> Result<(), SpecDbError>;
    fn list_ids(&self) -> Result<Vec<SpecId>, SpecDbError>;
    fn get_metadata(&self, id: &SpecId) -> Result<Option<SpecNode>, SpecDbError>;
}
