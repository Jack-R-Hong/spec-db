use spec_db_core::{
    CausalEdge, CausalGraph, EdgeOrigin, SearchEngine, SpecDbError, SpecDoc, SpecId, SpecNode,
    TrustLevel,
};

use crate::parser::parse_spec;

pub struct IngestPipeline<S: SearchEngine, G: CausalGraph> {
    search: S,
    graph: G,
}

impl<S: SearchEngine, G: CausalGraph> IngestPipeline<S, G> {
    pub fn new(search: S, graph: G) -> Self {
        Self { search, graph }
    }

    pub fn add_spec(&mut self, markdown: &str) -> Result<SpecId, SpecDbError> {
        let _span = tracing::info_span!("spec_db.ingest.add_spec").entered();

        let doc: SpecDoc = parse_spec(markdown)?;
        if self.graph.get_node(&doc.id)?.is_some() {
            return Err(SpecDbError::IngestError(format!("duplicate spec ID: {}", doc.id)));
        }

        let node = SpecNode { id: doc.id.clone(), title: doc.title.clone(), version: doc.version };
        let edges: Vec<CausalEdge> = doc
            .depends_on
            .iter()
            .map(|dep| CausalEdge {
                source: doc.id.clone(),
                target: dep.clone(),
                trust: TrustLevel::human(),
                origin: EdgeOrigin::Human,
            })
            .collect();

        self.search.index_spec(&doc)?;

        if let Err(err) = self.graph.upsert_node(node) {
            let _ = self.search.remove_spec(&doc.id);
            return Err(err);
        }

        for edge in edges {
            if let Err(err) = self.graph.add_edge(edge) {
                let _ = self.graph.remove_node(&doc.id);
                let _ = self.search.remove_spec(&doc.id);
                return Err(err);
            }
        }

        Ok(doc.id)
    }

    pub fn remove_spec(&mut self, id: &SpecId) -> Result<(), SpecDbError> {
        self.graph.remove_node(id)?;
        self.search.remove_spec(id)?;
        Ok(())
    }

    pub fn search(&self) -> &S {
        &self.search
    }

    pub fn graph(&self) -> &G {
        &self.graph
    }

    pub fn graph_mut(&mut self) -> &mut G {
        &mut self.graph
    }
}
