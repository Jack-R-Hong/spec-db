use std::collections::HashMap;
use std::sync::Arc;

use spec_db_core::{CausalEdge, CausalGraph, SpecDbError, SpecId, SpecNode};

use crate::store::FjallStore;
use crate::traversal;

pub struct NodeView {
    pub node: SpecNode,
    pub inbound_edges: Vec<CausalEdge>,
    pub outbound_edges: Vec<CausalEdge>,
}

pub struct CausalEngine {
    store: Arc<FjallStore>,
    nodes: HashMap<String, SpecNode>,
    outbound_edges: HashMap<String, Vec<CausalEdge>>,
    inbound_edges: HashMap<String, Vec<CausalEdge>>,
}

impl CausalEngine {
    pub fn from_store(store: Arc<FjallStore>) -> Result<Self, SpecDbError> {
        let mut engine = Self {
            store,
            nodes: HashMap::new(),
            outbound_edges: HashMap::new(),
            inbound_edges: HashMap::new(),
        };
        engine.load_from_store()?;
        Ok(engine)
    }

    #[tracing::instrument(name = "spec_db.graph.load", skip(self))]
    pub fn load_from_store(&mut self) -> Result<(), SpecDbError> {
        self.nodes.clear();
        self.outbound_edges.clear();
        self.inbound_edges.clear();

        let all_nodes = self.store.iter_nodes()?;
        for node in all_nodes {
            let key = node.id.as_ref().to_owned();
            self.nodes.insert(key, node);
        }

        let all_edges = self.store.iter_edges()?;
        for edge in all_edges {
            self.index_edge(edge);
        }

        tracing::info!(
            nodes = self.nodes.len(),
            edges = self.outbound_edges.values().map(|v| v.len()).sum::<usize>(),
            "graph loaded from store"
        );

        Ok(())
    }

    fn index_edge(&mut self, edge: CausalEdge) {
        let src_key = edge.source.as_ref().to_owned();
        let tgt_key = edge.target.as_ref().to_owned();
        self.inbound_edges.entry(tgt_key).or_default().push(edge.clone());
        self.outbound_edges.entry(src_key).or_default().push(edge);
    }

    pub fn node_view(&self, id: &SpecId) -> Result<NodeView, SpecDbError> {
        let key = id.as_ref();
        let node = self
            .nodes
            .get(key)
            .ok_or_else(|| SpecDbError::GraphError(format!("node not found: {id}")))?
            .clone();

        let outbound_edges = self.outbound_edges.get(key).cloned().unwrap_or_default();
        let inbound_edges = self.inbound_edges.get(key).cloned().unwrap_or_default();

        Ok(NodeView { node, inbound_edges, outbound_edges })
    }

    fn ensure_node_exists(&self, id: &SpecId) -> Result<(), SpecDbError> {
        if !self.nodes.contains_key(id.as_ref()) {
            return Err(SpecDbError::GraphError(format!("Spec not found: {id}")));
        }
        Ok(())
    }
}

impl CausalGraph for CausalEngine {
    fn upsert_node(&mut self, node: SpecNode) -> Result<(), SpecDbError> {
        self.store.put_node(&node)?;
        let key = node.id.as_ref().to_owned();
        self.nodes.insert(key, node);
        Ok(())
    }

    fn remove_node(&mut self, id: &SpecId) -> Result<(), SpecDbError> {
        self.store.remove_node(id)?;
        let key = id.as_ref();
        self.nodes.remove(key);
        self.outbound_edges.remove(key);
        self.inbound_edges.remove(key);
        for edges in self.outbound_edges.values_mut() {
            edges.retain(|e| e.target.as_ref() != key);
        }
        for edges in self.inbound_edges.values_mut() {
            edges.retain(|e| e.source.as_ref() != key);
        }
        Ok(())
    }

    fn get_node(&self, id: &SpecId) -> Result<Option<SpecNode>, SpecDbError> {
        Ok(self.nodes.get(id.as_ref()).cloned())
    }

    fn add_edge(&mut self, edge: CausalEdge) -> Result<(), SpecDbError> {
        self.store.put_edge(&edge)?;
        self.index_edge(edge);
        Ok(())
    }

    fn remove_edge(&mut self, source: &SpecId, target: &SpecId) -> Result<(), SpecDbError> {
        self.store.remove_edge(source, target)?;
        let src_key = source.as_ref();
        let tgt_key = target.as_ref();
        if let Some(edges) = self.outbound_edges.get_mut(src_key) {
            edges.retain(|e| e.target.as_ref() != tgt_key);
        }
        if let Some(edges) = self.inbound_edges.get_mut(tgt_key) {
            edges.retain(|e| e.source.as_ref() != src_key);
        }
        Ok(())
    }

    #[tracing::instrument(
        name = "spec_db.graph.traverse",
        skip(self),
        fields(operation = "trace_impact", start_id = %id, depth_limit = ?depth, result_count = tracing::field::Empty)
    )]
    fn trace_impact(&self, id: &SpecId, depth: Option<usize>) -> Result<Vec<SpecId>, SpecDbError> {
        self.ensure_node_exists(id)?;
        let impacted =
            traversal::bfs_traverse(&self.inbound_edges, id, depth, |edge| &edge.source)?;
        tracing::Span::current().record("result_count", impacted.len());
        Ok(impacted)
    }

    #[tracing::instrument(
        name = "spec_db.graph.traverse",
        skip(self),
        fields(operation = "find_dependencies", start_id = %id, depth_limit = ?depth, result_count = tracing::field::Empty)
    )]
    fn find_dependencies(
        &self,
        id: &SpecId,
        depth: Option<usize>,
    ) -> Result<Vec<SpecId>, SpecDbError> {
        self.ensure_node_exists(id)?;
        let dependencies =
            traversal::bfs_traverse(&self.outbound_edges, id, depth, |edge| &edge.target)?;
        tracing::Span::current().record("result_count", dependencies.len());
        Ok(dependencies)
    }

    fn edges_from(&self, id: &SpecId) -> Result<Vec<CausalEdge>, SpecDbError> {
        Ok(self.outbound_edges.get(id.as_ref()).cloned().unwrap_or_default())
    }

    fn edges_to(&self, id: &SpecId) -> Result<Vec<CausalEdge>, SpecDbError> {
        Ok(self.inbound_edges.get(id.as_ref()).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spec_db_core::{EdgeOrigin, TrustLevel};
    use std::time::{Duration, Instant};

    fn temp_engine() -> (tempfile::TempDir, CausalEngine) {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(FjallStore::open(dir.path()).unwrap());
        let engine = CausalEngine::from_store(store).unwrap();
        (dir, engine)
    }

    fn node(domain: &str, name: &str) -> SpecNode {
        SpecNode {
            id: SpecId::try_new(format!("spec::{domain}::{name}")).unwrap(),
            title: name.to_owned(),
            version: 1,
        }
    }

    fn human_edge(from: &SpecNode, to: &SpecNode) -> CausalEdge {
        CausalEdge {
            source: from.id.clone(),
            target: to.id.clone(),
            trust: TrustLevel::human(),
            origin: EdgeOrigin::Human,
        }
    }

    #[test]
    fn add_and_get_node() {
        let (_dir, mut engine) = temp_engine();
        let n = node("auth", "login");
        engine.upsert_node(n.clone()).unwrap();
        let loaded = engine.get_node(&n.id).unwrap().unwrap();
        assert_eq!(loaded.id.as_ref(), n.id.as_ref());
    }

    #[test]
    fn add_edge_creates_adjacency() {
        let (_dir, mut engine) = temp_engine();
        let a = node("auth", "login");
        let b = node("auth", "token");
        engine.upsert_node(a.clone()).unwrap();
        engine.upsert_node(b.clone()).unwrap();
        engine.add_edge(human_edge(&a, &b)).unwrap();

        assert_eq!(engine.edges_from(&a.id).unwrap().len(), 1);
        assert_eq!(engine.edges_to(&b.id).unwrap().len(), 1);
    }

    #[test]
    fn depends_on_edge_auto_trust() {
        let (_dir, mut engine) = temp_engine();
        let a = node("api", "handler");
        let b = node("api", "middleware");
        engine.upsert_node(a.clone()).unwrap();
        engine.upsert_node(b.clone()).unwrap();

        let edge = CausalEdge {
            source: a.id.clone(),
            target: b.id.clone(),
            trust: TrustLevel::human(),
            origin: EdgeOrigin::Human,
        };
        engine.add_edge(edge).unwrap();

        let outbound = engine.edges_from(&a.id).unwrap();
        assert_eq!(outbound.len(), 1);
        assert_eq!(outbound[0].trust.value(), 1.0);
        assert_eq!(outbound[0].origin, EdgeOrigin::Human);
    }

    #[test]
    fn node_view_shows_inbound_and_outbound() {
        let (_dir, mut engine) = temp_engine();
        let a = node("svc", "frontend");
        let b = node("svc", "backend");
        let c = node("svc", "database");
        engine.upsert_node(a.clone()).unwrap();
        engine.upsert_node(b.clone()).unwrap();
        engine.upsert_node(c.clone()).unwrap();
        engine.add_edge(human_edge(&a, &b)).unwrap();
        engine.add_edge(human_edge(&b, &c)).unwrap();

        let view = engine.node_view(&b.id).unwrap();
        assert_eq!(view.node.id.as_ref(), b.id.as_ref());
        assert_eq!(view.outbound_edges.len(), 1);
        assert_eq!(view.inbound_edges.len(), 1);
    }

    #[test]
    fn trace_impact_transitive() {
        let (_dir, mut engine) = temp_engine();
        let a = node("chain", "leaf");
        let b = node("chain", "mid");
        let c = node("chain", "root");
        engine.upsert_node(a.clone()).unwrap();
        engine.upsert_node(b.clone()).unwrap();
        engine.upsert_node(c.clone()).unwrap();
        engine.add_edge(human_edge(&a, &b)).unwrap();
        engine.add_edge(human_edge(&b, &c)).unwrap();

        let impacted = engine.trace_impact(&c.id, None).unwrap();
        assert_eq!(impacted.len(), 2);
    }

    #[test]
    fn find_dependencies_transitive() {
        let (_dir, mut engine) = temp_engine();
        let a = node("dep", "top");
        let b = node("dep", "mid");
        let c = node("dep", "bottom");
        engine.upsert_node(a.clone()).unwrap();
        engine.upsert_node(b.clone()).unwrap();
        engine.upsert_node(c.clone()).unwrap();
        engine.add_edge(human_edge(&a, &b)).unwrap();
        engine.add_edge(human_edge(&b, &c)).unwrap();

        let deps = engine.find_dependencies(&a.id, None).unwrap();
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn depth_limited_trace_impact() {
        let (_dir, mut engine) = temp_engine();
        let a = node("chain", "a");
        let b = node("chain", "b");
        let c = node("chain", "c");
        let d = node("chain", "d");
        let e = node("chain", "e");

        for n in [&a, &b, &c, &d, &e] {
            engine.upsert_node(n.clone()).unwrap();
        }

        engine.add_edge(human_edge(&a, &b)).unwrap();
        engine.add_edge(human_edge(&b, &c)).unwrap();
        engine.add_edge(human_edge(&c, &d)).unwrap();
        engine.add_edge(human_edge(&d, &e)).unwrap();

        let impacted = engine.trace_impact(&e.id, Some(2)).unwrap();
        assert_eq!(impacted, vec![d.id.clone(), c.id.clone()]);
    }

    #[test]
    fn depth_limited_find_dependencies() {
        let (_dir, mut engine) = temp_engine();
        let a = node("chain", "a");
        let b = node("chain", "b");
        let c = node("chain", "c");
        let d = node("chain", "d");
        let e = node("chain", "e");

        for n in [&a, &b, &c, &d, &e] {
            engine.upsert_node(n.clone()).unwrap();
        }

        engine.add_edge(human_edge(&a, &b)).unwrap();
        engine.add_edge(human_edge(&b, &c)).unwrap();
        engine.add_edge(human_edge(&c, &d)).unwrap();
        engine.add_edge(human_edge(&d, &e)).unwrap();

        let dependencies = engine.find_dependencies(&a.id, Some(2)).unwrap();
        assert_eq!(dependencies, vec![b.id.clone(), c.id.clone()]);
    }

    #[test]
    fn full_transitive_no_depth_limit() {
        let (_dir, mut engine) = temp_engine();
        let a = node("chain", "a");
        let b = node("chain", "b");
        let c = node("chain", "c");
        let d = node("chain", "d");
        let e = node("chain", "e");

        for n in [&a, &b, &c, &d, &e] {
            engine.upsert_node(n.clone()).unwrap();
        }

        engine.add_edge(human_edge(&a, &b)).unwrap();
        engine.add_edge(human_edge(&b, &c)).unwrap();
        engine.add_edge(human_edge(&c, &d)).unwrap();
        engine.add_edge(human_edge(&d, &e)).unwrap();

        let impacted = engine.trace_impact(&e.id, None).unwrap();
        assert_eq!(impacted, vec![d.id.clone(), c.id.clone(), b.id.clone(), a.id.clone()]);
    }

    #[test]
    fn missing_node_returns_error() {
        let (_dir, engine) = temp_engine();
        let missing = SpecId::try_new("spec::missing::node").unwrap();
        let err = engine.trace_impact(&missing, None).unwrap_err();

        match err {
            SpecDbError::GraphError(message) => {
                assert_eq!(message, format!("Spec not found: {missing}"))
            }
            other => panic!("expected GraphError, got {other:?}"),
        }
    }

    #[test]
    fn traversal_perf_100_specs() {
        let (_dir, mut engine) = temp_engine();

        for i in 0..150 {
            let n = SpecNode {
                id: SpecId::try_new(format!("spec::perf::node-{i}")).unwrap(),
                title: format!("node {i}"),
                version: 1,
            };
            engine.upsert_node(n).unwrap();
        }

        for i in 0..149 {
            let from = SpecId::try_new(format!("spec::perf::node-{i}")).unwrap();
            let to = SpecId::try_new(format!("spec::perf::node-{}", i + 1)).unwrap();
            engine
                .add_edge(CausalEdge {
                    source: from,
                    target: to,
                    trust: TrustLevel::human(),
                    origin: EdgeOrigin::Human,
                })
                .unwrap();
        }

        let leaf = SpecId::try_new("spec::perf::node-149").unwrap();
        let root = SpecId::try_new("spec::perf::node-0").unwrap();

        let impact_start = Instant::now();
        let impacted = engine.trace_impact(&leaf, None).unwrap();
        let impact_elapsed = impact_start.elapsed();

        let dep_start = Instant::now();
        let dependencies = engine.find_dependencies(&root, None).unwrap();
        let dep_elapsed = dep_start.elapsed();

        assert_eq!(impacted.len(), 149);
        assert_eq!(dependencies.len(), 149);
        assert!(impact_elapsed < Duration::from_millis(50), "trace_impact took {impact_elapsed:?}");
        assert!(dep_elapsed < Duration::from_millis(50), "find_dependencies took {dep_elapsed:?}");
    }

    #[test]
    fn reload_from_store_preserves_graph() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(FjallStore::open(dir.path()).unwrap());

        {
            let mut engine = CausalEngine::from_store(store.clone()).unwrap();
            let a = node("persist", "alpha");
            let b = node("persist", "beta");
            engine.upsert_node(a.clone()).unwrap();
            engine.upsert_node(b.clone()).unwrap();
            engine.add_edge(human_edge(&a, &b)).unwrap();
        }

        {
            let engine = CausalEngine::from_store(store).unwrap();
            let id_a = SpecId::try_new("spec::persist::alpha").unwrap();
            let id_b = SpecId::try_new("spec::persist::beta").unwrap();
            assert!(engine.get_node(&id_a).unwrap().is_some());
            assert!(engine.get_node(&id_b).unwrap().is_some());
            assert_eq!(engine.edges_from(&id_a).unwrap().len(), 1);
        }
    }

    #[test]
    fn startup_performance_100_specs() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(FjallStore::open(dir.path()).unwrap());

        {
            let mut engine = CausalEngine::from_store(store.clone()).unwrap();
            for i in 0..100 {
                let n = SpecNode {
                    id: SpecId::try_new(format!("spec::perf::node-{i}")).unwrap(),
                    title: format!("node {i}"),
                    version: 1,
                };
                engine.upsert_node(n).unwrap();
            }
            for i in 0..99 {
                let from = SpecId::try_new(format!("spec::perf::node-{i}")).unwrap();
                let to = SpecId::try_new(format!("spec::perf::node-{}", i + 1)).unwrap();
                engine
                    .add_edge(CausalEdge {
                        source: from,
                        target: to,
                        trust: TrustLevel::human(),
                        origin: EdgeOrigin::Human,
                    })
                    .unwrap();
            }
        }

        let start = std::time::Instant::now();
        let _engine = CausalEngine::from_store(store).unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed.as_secs() < 1, "startup took {elapsed:?}, exceeds 1s threshold");
    }
}
