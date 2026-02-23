use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use deep_causality::{
    CausableGraph, Causaloid, CausaloidGraph, IdentificationValue, PropagatingEffect,
};
use spec_db_core::{
    CausalEdge, CausalGraph, EdgeOrigin, EdgeType, SpecDbError, SpecId, SpecNode, TrustLevel,
};
use ultragraph::GraphTraversal;

use crate::store::FjallStore;
use crate::traversal;

type SpecCausaloid = Causaloid<bool, bool, (), ()>;

fn identity_causal_fn(input: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::from_value(input)
}

pub struct NodeView {
    pub node: SpecNode,
    pub inbound_edges: Vec<CausalEdge>,
    pub outbound_edges: Vec<CausalEdge>,
}

pub struct CausalEngine {
    store: Arc<FjallStore>,
    graph: CausaloidGraph<SpecCausaloid>,
    id_to_index: HashMap<String, usize>,
    index_to_id: HashMap<usize, String>,
    nodes: HashMap<String, SpecNode>,
    edge_meta: HashMap<(usize, usize), (TrustLevel, EdgeOrigin, EdgeType)>,
    next_causaloid_id: IdentificationValue,
}

impl CausalEngine {
    pub fn from_store(store: Arc<FjallStore>) -> Result<Self, SpecDbError> {
        let mut engine = Self {
            store,
            graph: CausaloidGraph::new(0),
            id_to_index: HashMap::new(),
            index_to_id: HashMap::new(),
            nodes: HashMap::new(),
            edge_meta: HashMap::new(),
            next_causaloid_id: 0,
        };
        engine.load_from_store()?;
        Ok(engine)
    }

    #[tracing::instrument(name = "spec_db.graph.load", skip(self))]
    pub fn load_from_store(&mut self) -> Result<(), SpecDbError> {
        self.graph = CausaloidGraph::new(0);
        self.id_to_index.clear();
        self.index_to_id.clear();
        self.nodes.clear();
        self.edge_meta.clear();
        self.next_causaloid_id = 0;

        let mut all_nodes = self.store.iter_nodes()?;
        all_nodes.sort_by(|a, b| a.id.as_ref().cmp(b.id.as_ref()));

        for node in all_nodes {
            let key = node.id.as_ref().to_owned();
            self.add_causaloid_for_id(&node.id, node.title.as_str())?;
            self.nodes.insert(key, node);
        }

        let mut all_edges = self.store.iter_edges()?;
        all_edges.sort_by(|a, b| {
            a.source.as_ref().cmp(b.source.as_ref()).then(a.target.as_ref().cmp(b.target.as_ref()))
        });

        for edge in all_edges {
            let source = self.ensure_index_for_id(&edge.source)?;
            let target = self.ensure_index_for_id(&edge.target)?;
            self.graph
                .add_edge(source, target)
                .map_err(|e| SpecDbError::GraphError(format!("failed to add edge: {e}")))?;
            self.edge_meta.insert((source, target), (edge.trust, edge.origin, edge.edge_type));
        }

        self.graph.freeze();

        tracing::info!(
            nodes = self.nodes.len(),
            edges = self.edge_meta.len(),
            "graph loaded from store"
        );

        Ok(())
    }

    fn add_causaloid_for_id(
        &mut self,
        id: &SpecId,
        description: &str,
    ) -> Result<usize, SpecDbError> {
        let causaloid = SpecCausaloid::new(self.next_causaloid_id, identity_causal_fn, description);
        self.next_causaloid_id = self.next_causaloid_id.saturating_add(1);
        let index = self
            .graph
            .add_causaloid(causaloid)
            .map_err(|e| SpecDbError::GraphError(format!("failed to add causaloid: {e}")))?;
        let key = id.as_ref().to_owned();
        self.id_to_index.insert(key.clone(), index);
        self.index_to_id.insert(index, key);
        Ok(index)
    }

    fn ensure_index_for_id(&mut self, id: &SpecId) -> Result<usize, SpecDbError> {
        if let Some(index) = self.id_to_index.get(id.as_ref()).copied() {
            return Ok(index);
        }
        self.add_causaloid_for_id(id, id.as_ref())
    }

    pub fn all_edges(&self) -> Result<Vec<CausalEdge>, SpecDbError> {
        self.store.iter_edges()
    }

    pub fn update_edge_origin(
        &mut self,
        source: &SpecId,
        target: &SpecId,
        new_origin: EdgeOrigin,
        new_trust: TrustLevel,
    ) -> Result<(), SpecDbError> {
        let src_index = self.index_for_spec_id(source)?;
        let tgt_index = self.index_for_spec_id(target)?;

        let meta = self.edge_meta.get_mut(&(src_index, tgt_index)).ok_or_else(|| {
            SpecDbError::GraphError(format!("Edge not found: {source} -> {target}"))
        })?;

        meta.0 = new_trust;
        meta.1 = new_origin;

        let updated_edge = CausalEdge {
            source: source.clone(),
            target: target.clone(),
            edge_type: meta.2,
            trust: new_trust,
            origin: new_origin,
            created_at: None,
        };
        self.store.put_edge(&updated_edge)
    }

    pub fn node_view(&self, id: &SpecId) -> Result<NodeView, SpecDbError> {
        let key = id.as_ref();
        let node = self
            .nodes
            .get(key)
            .ok_or_else(|| SpecDbError::GraphError(format!("node not found: {id}")))?
            .clone();

        let outbound_edges = self.edges_from(id)?;
        let inbound_edges = self.edges_to(id)?;

        Ok(NodeView { node, inbound_edges, outbound_edges })
    }

    fn ensure_node_exists(&self, id: &SpecId) -> Result<(), SpecDbError> {
        if !self.nodes.contains_key(id.as_ref()) {
            return Err(SpecDbError::GraphError(format!("Spec not found: {id}")));
        }
        Ok(())
    }

    fn spec_id_for_index(&self, index: usize) -> Result<SpecId, SpecDbError> {
        let id = self.index_to_id.get(&index).ok_or_else(|| {
            SpecDbError::GraphError(format!("missing SpecId mapping for graph index {index}"))
        })?;
        SpecId::try_new(id.clone())
    }

    fn index_for_spec_id(&self, id: &SpecId) -> Result<usize, SpecDbError> {
        self.id_to_index
            .get(id.as_ref())
            .copied()
            .ok_or_else(|| SpecDbError::GraphError(format!("Spec not found: {id}")))
    }

    pub fn has_path(&self, from: usize, to: usize) -> Result<Option<Vec<String>>, SpecDbError> {
        if !self.index_to_id.contains_key(&from) {
            return Err(SpecDbError::GraphError(format!(
                "missing SpecId mapping for graph index {from}"
            )));
        }

        if !self.index_to_id.contains_key(&to) {
            return Err(SpecDbError::GraphError(format!(
                "missing SpecId mapping for graph index {to}"
            )));
        }

        if from == to {
            return Ok(Some(vec![self.spec_id_for_index(from)?.to_string()]));
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<usize, usize> = HashMap::new();

        visited.insert(from);
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            for neighbor in self.outbound_neighbors(current)? {
                if !visited.insert(neighbor) {
                    continue;
                }

                parent.insert(neighbor, current);

                if neighbor == to {
                    return self.reconstruct_path(from, to, &parent).map(Some);
                }

                queue.push_back(neighbor);
            }
        }

        Ok(None)
    }

    pub fn validate_no_cycle(
        &self,
        source: &SpecId,
        target: &SpecId,
    ) -> Result<Option<Vec<String>>, SpecDbError> {
        let source_index = self.index_for_spec_id(source)?;
        let target_index = self.index_for_spec_id(target)?;

        if source_index == target_index {
            let source_id = source.to_string();
            return Ok(Some(vec![source_id.clone(), source_id]));
        }

        let Some(path) = self.has_path(target_index, source_index)? else {
            return Ok(None);
        };

        let mut cycle = Vec::with_capacity(path.len() + 1);
        cycle.push(source.to_string());
        cycle.extend(path);
        Ok(Some(cycle))
    }

    fn reconstruct_path(
        &self,
        from: usize,
        to: usize,
        parent: &HashMap<usize, usize>,
    ) -> Result<Vec<String>, SpecDbError> {
        let mut path = vec![to];
        let mut current = to;

        while current != from {
            let previous = parent.get(&current).copied().ok_or_else(|| {
                SpecDbError::GraphError(format!(
                    "failed to reconstruct path from index {from} to index {to}"
                ))
            })?;
            path.push(previous);
            current = previous;
        }

        path.reverse();

        path.into_iter()
            .map(|index| self.spec_id_for_index(index).map(|id| id.to_string()))
            .collect()
    }

    fn build_edge(&self, source: usize, target: usize) -> Result<CausalEdge, SpecDbError> {
        let source_id = self.spec_id_for_index(source)?;
        let target_id = self.spec_id_for_index(target)?;
        let (trust, origin, edge_type) = self
            .edge_meta
            .get(&(source, target))
            .copied()
            .unwrap_or((TrustLevel::human(), EdgeOrigin::Human, EdgeType::DependsOn));

        Ok(CausalEdge {
            source: source_id,
            target: target_id,
            edge_type,
            trust,
            origin,
            created_at: None,
        })
    }

    fn inbound_neighbors(&self, index: usize) -> Result<Vec<usize>, SpecDbError> {
        let neighbors = self
            .graph
            .get_graph()
            .inbound_edges(index)
            .map_err(|e| SpecDbError::GraphError(format!("inbound traversal failed: {e}")))?
            .collect();
        Ok(neighbors)
    }

    fn outbound_neighbors(&self, index: usize) -> Result<Vec<usize>, SpecDbError> {
        let neighbors = self
            .graph
            .get_graph()
            .outbound_edges(index)
            .map_err(|e| SpecDbError::GraphError(format!("outbound traversal failed: {e}")))?
            .collect();
        Ok(neighbors)
    }
}

impl CausalGraph for CausalEngine {
    fn upsert_node(&mut self, node: SpecNode) -> Result<(), SpecDbError> {
        let key = node.id.as_ref().to_owned();

        self.graph.unfreeze();
        if !self.id_to_index.contains_key(&key) {
            let _ = self.add_causaloid_for_id(&node.id, node.title.as_str())?;
        }

        self.nodes.insert(key, node.clone());
        let result = self.store.put_node(&node);
        self.graph.freeze();
        result
    }

    fn remove_node(&mut self, id: &SpecId) -> Result<(), SpecDbError> {
        let key = id.as_ref().to_owned();
        let incident_edges: Vec<(SpecId, SpecId)> = self
            .store
            .iter_edges()?
            .into_iter()
            .filter(|edge| edge.source.as_ref() == key || edge.target.as_ref() == key)
            .map(|edge| (edge.source, edge.target))
            .collect();

        self.graph.unfreeze();

        if let Some(index) = self.id_to_index.remove(&key) {
            self.index_to_id.remove(&index);
            self.edge_meta.retain(|(source, target), _| *source != index && *target != index);

            if self.graph.contains_causaloid(index) {
                self.graph.remove_causaloid(index).map_err(|e| {
                    SpecDbError::GraphError(format!("failed to remove causaloid {index}: {e}"))
                })?;
            }
        }

        self.nodes.remove(&key);

        let mut result = self.store.remove_node(id);
        if result.is_ok() {
            for (source, target) in incident_edges {
                if let Err(e) = self.store.remove_edge(&source, &target) {
                    result = Err(e);
                    break;
                }
            }
        }

        self.graph.freeze();
        result
    }

    fn get_node(&self, id: &SpecId) -> Result<Option<SpecNode>, SpecDbError> {
        Ok(self.nodes.get(id.as_ref()).cloned())
    }

    fn add_edge(&mut self, edge: CausalEdge) -> Result<(), SpecDbError> {
        self.graph.unfreeze();
        let source = self.ensure_index_for_id(&edge.source)?;
        let target = self.ensure_index_for_id(&edge.target)?;
        self.graph
            .add_edge(source, target)
            .map_err(|e| SpecDbError::GraphError(format!("failed to add edge: {e}")))?;
        self.edge_meta.insert((source, target), (edge.trust, edge.origin, edge.edge_type));
        let result = self.store.put_edge(&edge);
        self.graph.freeze();
        result
    }

    fn remove_edge(&mut self, source: &SpecId, target: &SpecId) -> Result<(), SpecDbError> {
        let src_index = self.id_to_index.get(source.as_ref()).copied();
        let tgt_index = self.id_to_index.get(target.as_ref()).copied();

        self.graph.unfreeze();
        if let (Some(src), Some(tgt)) = (src_index, tgt_index) {
            if self.graph.contains_edge(src, tgt) {
                self.graph
                    .remove_edge(src, tgt)
                    .map_err(|e| SpecDbError::GraphError(format!("failed to remove edge: {e}")))?;
            }
            self.edge_meta.remove(&(src, tgt));
        }

        let result = self.store.remove_edge(source, target);
        self.graph.freeze();
        result
    }

    #[tracing::instrument(
        name = "spec_db.graph.traverse",
        skip(self),
        fields(operation = "trace_impact", start_id = %id, depth_limit = ?depth, result_count = tracing::field::Empty)
    )]
    fn trace_impact(&self, id: &SpecId, depth: Option<usize>) -> Result<Vec<SpecId>, SpecDbError> {
        self.ensure_node_exists(id)?;
        let start_index = self
            .id_to_index
            .get(id.as_ref())
            .copied()
            .ok_or_else(|| SpecDbError::GraphError(format!("Spec not found: {id}")))?;
        let impacted = traversal::bfs_traverse_indices(start_index, depth, |current| {
            self.inbound_neighbors(current)
        })?;
        let impacted: Vec<SpecId> = impacted
            .into_iter()
            .map(|index| self.spec_id_for_index(index))
            .collect::<Result<_, _>>()?;
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
        let start_index = self
            .id_to_index
            .get(id.as_ref())
            .copied()
            .ok_or_else(|| SpecDbError::GraphError(format!("Spec not found: {id}")))?;
        let dependencies = traversal::bfs_traverse_indices(start_index, depth, |current| {
            self.outbound_neighbors(current)
        })?;
        let dependencies: Vec<SpecId> = dependencies
            .into_iter()
            .map(|index| self.spec_id_for_index(index))
            .collect::<Result<_, _>>()?;
        tracing::Span::current().record("result_count", dependencies.len());
        Ok(dependencies)
    }

    fn edges_from(&self, id: &SpecId) -> Result<Vec<CausalEdge>, SpecDbError> {
        let Some(index) = self.id_to_index.get(id.as_ref()).copied() else {
            return Ok(Vec::new());
        };

        self.graph
            .get_graph()
            .outbound_edges(index)
            .map_err(|e| SpecDbError::GraphError(format!("outbound edge read failed: {e}")))?
            .map(|target| self.build_edge(index, target))
            .collect()
    }

    fn edges_to(&self, id: &SpecId) -> Result<Vec<CausalEdge>, SpecDbError> {
        let Some(index) = self.id_to_index.get(id.as_ref()).copied() else {
            return Ok(Vec::new());
        };

        self.graph
            .get_graph()
            .inbound_edges(index)
            .map_err(|e| SpecDbError::GraphError(format!("inbound edge read failed: {e}")))?
            .map(|source| self.build_edge(source, index))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spec_db_core::{EdgeOrigin, EdgeType, TrustLevel};
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
            edge_type: EdgeType::DependsOn,
            trust: TrustLevel::human(),
            origin: EdgeOrigin::Human,
            created_at: None,
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
            edge_type: EdgeType::DependsOn,
            trust: TrustLevel::human(),
            origin: EdgeOrigin::Human,
            created_at: None,
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
                    edge_type: EdgeType::DependsOn,
                    trust: TrustLevel::human(),
                    origin: EdgeOrigin::Human,
                    created_at: None,
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
                        edge_type: EdgeType::DependsOn,
                        trust: TrustLevel::human(),
                        origin: EdgeOrigin::Human,
                        created_at: None,
                    })
                    .unwrap();
            }
        }

        let start = std::time::Instant::now();
        let _engine = CausalEngine::from_store(store).unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed.as_secs() < 1, "startup took {elapsed:?}, exceeds 1s threshold");
    }

    #[test]
    fn validate_no_cycle_rejects_direct_cycle() {
        let (_dir, mut engine) = temp_engine();
        let a = node("cycle", "a");
        let b = node("cycle", "b");
        engine.upsert_node(a.clone()).unwrap();
        engine.upsert_node(b.clone()).unwrap();
        engine.add_edge(human_edge(&a, &b)).unwrap();

        let cycle =
            engine.validate_no_cycle(&b.id, &a.id).unwrap().expect("cycle should be detected");
        assert_eq!(cycle, vec![b.id.to_string(), a.id.to_string(), b.id.to_string()]);
    }

    #[test]
    fn validate_no_cycle_rejects_indirect_cycle() {
        let (_dir, mut engine) = temp_engine();
        let a = node("cycle", "a");
        let b = node("cycle", "b");
        let c = node("cycle", "c");
        engine.upsert_node(a.clone()).unwrap();
        engine.upsert_node(b.clone()).unwrap();
        engine.upsert_node(c.clone()).unwrap();
        engine.add_edge(human_edge(&a, &b)).unwrap();
        engine.add_edge(human_edge(&b, &c)).unwrap();

        let cycle =
            engine.validate_no_cycle(&c.id, &a.id).unwrap().expect("cycle should be detected");
        assert_eq!(
            cycle,
            vec![c.id.to_string(), a.id.to_string(), b.id.to_string(), c.id.to_string()]
        );
    }

    #[test]
    fn validate_no_cycle_allows_connected_non_cycle_edge() {
        let (_dir, mut engine) = temp_engine();
        let a = node("cycle", "a");
        let b = node("cycle", "b");
        let c = node("cycle", "c");
        engine.upsert_node(a.clone()).unwrap();
        engine.upsert_node(b.clone()).unwrap();
        engine.upsert_node(c.clone()).unwrap();
        engine.add_edge(human_edge(&a, &b)).unwrap();

        let cycle = engine.validate_no_cycle(&b.id, &c.id).unwrap();
        assert!(cycle.is_none());
    }

    #[test]
    fn validate_no_cycle_allows_disconnected_subgraph_connection() {
        let (_dir, mut engine) = temp_engine();
        let a = node("comp1", "a");
        let b = node("comp1", "b");
        let c = node("comp2", "c");
        let d = node("comp2", "d");
        engine.upsert_node(a.clone()).unwrap();
        engine.upsert_node(b.clone()).unwrap();
        engine.upsert_node(c.clone()).unwrap();
        engine.upsert_node(d.clone()).unwrap();
        engine.add_edge(human_edge(&a, &b)).unwrap();
        engine.add_edge(human_edge(&c, &d)).unwrap();

        let cycle = engine.validate_no_cycle(&b.id, &c.id).unwrap();
        assert!(cycle.is_none());
    }

    #[test]
    fn validate_no_cycle_detects_self_loop() {
        let (_dir, mut engine) = temp_engine();
        let a = node("self", "a");
        engine.upsert_node(a.clone()).unwrap();

        let cycle =
            engine.validate_no_cycle(&a.id, &a.id).unwrap().expect("self loop should be detected");
        assert_eq!(cycle, vec![a.id.to_string(), a.id.to_string()]);
    }

    #[test]
    fn validation_perf_100_specs_under_100ms() {
        let (_dir, mut engine) = temp_engine();

        for i in 0..150 {
            let n = SpecNode {
                id: SpecId::try_new(format!("spec::csm-perf::node-{i}")).unwrap(),
                title: format!("node {i}"),
                version: 1,
            };
            engine.upsert_node(n).unwrap();
        }

        for i in 0..149 {
            let from = SpecId::try_new(format!("spec::csm-perf::node-{i}")).unwrap();
            let to = SpecId::try_new(format!("spec::csm-perf::node-{}", i + 1)).unwrap();
            engine
                .add_edge(CausalEdge {
                    source: from,
                    target: to,
                    edge_type: EdgeType::DependsOn,
                    trust: TrustLevel::human(),
                    origin: EdgeOrigin::Human,
                    created_at: None,
                })
                .unwrap();
        }

        let source = SpecId::try_new("spec::csm-perf::node-149").unwrap();
        let target = SpecId::try_new("spec::csm-perf::node-0").unwrap();

        let start = Instant::now();
        let cycle = engine.validate_no_cycle(&source, &target).unwrap();
        let elapsed = start.elapsed();

        assert!(cycle.is_some(), "cycle should be detected");
        assert!(elapsed < Duration::from_millis(100), "validation took {elapsed:?}");
    }
}
