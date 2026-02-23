use std::path::Path;

use spec_db_core::{CausalEdge, SpecDbError, SpecId, SpecNode, SpecStore};

pub struct FjallStore {
    db: fjall::Database,
    nodes: fjall::Keyspace,
    edges: fjall::Keyspace,
    meta: fjall::Keyspace,
}

const META_LAST_SYNC_SHA: &str = "last_sync_sha";
const META_DOC_COUNT: &str = "doc_count";

fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, SpecDbError> {
    bincode::serde::encode_to_vec(value, bincode::config::standard())
        .map_err(|e| SpecDbError::GraphError(format!("bincode encode failed: {e}")))
}

fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, SpecDbError> {
    let (val, _len) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())
        .map_err(|e| SpecDbError::GraphError(format!("bincode decode failed: {e}")))?;
    Ok(val)
}

fn node_key(id: &SpecId) -> String {
    id.as_ref().to_owned()
}

// edge key = `{from_id}\x00{to_id}` as bytes
fn edge_key(from: &SpecId, to: &SpecId) -> Vec<u8> {
    let mut key = Vec::with_capacity(from.as_ref().len() + 1 + to.as_ref().len());
    key.extend_from_slice(from.as_ref().as_bytes());
    key.push(0x00);
    key.extend_from_slice(to.as_ref().as_bytes());
    key
}

#[allow(dead_code)]
pub fn edge_key_parts(key: &[u8]) -> Result<(SpecId, SpecId), SpecDbError> {
    let pos = key
        .iter()
        .position(|&b| b == 0x00)
        .ok_or_else(|| SpecDbError::GraphError("edge key missing null separator".into()))?;
    let from_str = std::str::from_utf8(&key[..pos])
        .map_err(|e| SpecDbError::GraphError(format!("edge key from_id not UTF-8: {e}")))?;
    let to_str = std::str::from_utf8(&key[pos + 1..])
        .map_err(|e| SpecDbError::GraphError(format!("edge key to_id not UTF-8: {e}")))?;
    Ok((SpecId::try_new(from_str)?, SpecId::try_new(to_str)?))
}

fn map_fjall_err(e: fjall::Error) -> SpecDbError {
    SpecDbError::GraphError(format!("fjall error: {e}"))
}

impl FjallStore {
    pub fn open(path: &Path) -> Result<Self, SpecDbError> {
        let db = fjall::Database::builder(path).open().map_err(map_fjall_err)?;
        let nodes =
            db.keyspace("nodes", fjall::KeyspaceCreateOptions::default).map_err(map_fjall_err)?;
        let edges =
            db.keyspace("edges", fjall::KeyspaceCreateOptions::default).map_err(map_fjall_err)?;
        let meta =
            db.keyspace("meta", fjall::KeyspaceCreateOptions::default).map_err(map_fjall_err)?;
        Ok(Self { db, nodes, edges, meta })
    }

    pub fn put_node(&self, node: &SpecNode) -> Result<(), SpecDbError> {
        let key = node_key(&node.id);
        let value = encode(node)?;
        self.nodes.insert(key, value).map_err(map_fjall_err)
    }

    pub fn get_node(&self, id: &SpecId) -> Result<Option<SpecNode>, SpecDbError> {
        let key = node_key(id);
        match self.nodes.get(key).map_err(map_fjall_err)? {
            Some(bytes) => Ok(Some(decode(&bytes)?)),
            None => Ok(None),
        }
    }

    pub fn put_edge(&self, edge: &CausalEdge) -> Result<(), SpecDbError> {
        let key = edge_key(&edge.source, &edge.target);
        let value = encode(edge)?;
        self.edges.insert(key, value).map_err(map_fjall_err)
    }

    pub fn get_edge(&self, from: &SpecId, to: &SpecId) -> Result<Option<CausalEdge>, SpecDbError> {
        let key = edge_key(from, to);
        match self.edges.get(key).map_err(map_fjall_err)? {
            Some(bytes) => Ok(Some(decode(&bytes)?)),
            None => Ok(None),
        }
    }

    pub fn iter_edges(&self) -> Result<Vec<CausalEdge>, SpecDbError> {
        let mut result = Vec::new();
        for guard in self.edges.iter() {
            let (_k, v) = guard.into_inner().map_err(map_fjall_err)?;
            let edge: CausalEdge = decode(&v)?;
            result.push(edge);
        }
        Ok(result)
    }

    pub fn iter_nodes(&self) -> Result<Vec<SpecNode>, SpecDbError> {
        let mut result = Vec::new();
        for guard in self.nodes.iter() {
            let (_k, v) = guard.into_inner().map_err(map_fjall_err)?;
            let node: SpecNode = decode(&v)?;
            result.push(node);
        }
        Ok(result)
    }

    pub fn set_last_sync_sha(&self, sha: &str) -> Result<(), SpecDbError> {
        self.meta.insert(META_LAST_SYNC_SHA, sha.as_bytes()).map_err(map_fjall_err)
    }

    pub fn last_sync_sha(&self) -> Result<Option<String>, SpecDbError> {
        match self.meta.get(META_LAST_SYNC_SHA).map_err(map_fjall_err)? {
            Some(bytes) => {
                let s = String::from_utf8(bytes.to_vec()).map_err(|e| {
                    SpecDbError::GraphError(format!("last_sync_sha not UTF-8: {e}"))
                })?;
                Ok(Some(s))
            }
            None => Ok(None),
        }
    }

    pub fn set_doc_count(&self, count: usize) -> Result<(), SpecDbError> {
        self.meta.insert(META_DOC_COUNT, count.to_string().as_bytes()).map_err(map_fjall_err)
    }

    pub fn doc_count(&self) -> Result<Option<usize>, SpecDbError> {
        match self.meta.get(META_DOC_COUNT).map_err(map_fjall_err)? {
            Some(bytes) => {
                let s = String::from_utf8(bytes.to_vec())
                    .map_err(|e| SpecDbError::GraphError(format!("doc_count not UTF-8: {e}")))?;
                let count = s
                    .parse::<usize>()
                    .map_err(|e| SpecDbError::GraphError(format!("doc_count not a number: {e}")))?;
                Ok(Some(count))
            }
            None => Ok(None),
        }
    }

    pub fn put_node_with_edges(
        &self,
        node: &SpecNode,
        edges: &[CausalEdge],
    ) -> Result<(), SpecDbError> {
        let node_k = node_key(&node.id);
        let node_v = encode(node)?;

        let mut encoded_edges = Vec::with_capacity(edges.len());
        for edge in edges {
            let k = edge_key(&edge.source, &edge.target);
            let v = encode(edge)?;
            encoded_edges.push((k, v));
        }

        let mut batch = self.db.batch();
        batch.insert(&self.nodes, node_k, node_v);
        for (k, v) in &encoded_edges {
            batch.insert(&self.edges, k.as_slice(), v.as_slice());
        }
        batch.commit().map_err(map_fjall_err)
    }

    pub fn remove_node(&self, id: &SpecId) -> Result<(), SpecDbError> {
        let key = node_key(id);
        self.nodes.remove(key).map_err(map_fjall_err)
    }

    pub fn remove_edge(&self, from: &SpecId, to: &SpecId) -> Result<(), SpecDbError> {
        let key = edge_key(from, to);
        self.edges.remove(key).map_err(map_fjall_err)
    }
}

impl SpecStore for FjallStore {
    fn put(&mut self, doc: spec_db_core::SpecDoc) -> Result<(), SpecDbError> {
        let node = SpecNode { id: doc.id.clone(), title: doc.title.clone(), version: doc.version };
        let edges: Vec<CausalEdge> = doc
            .depends_on
            .iter()
            .map(|dep| CausalEdge {
                source: doc.id.clone(),
                target: dep.clone(),
                trust: spec_db_core::TrustLevel::human(),
                origin: spec_db_core::EdgeOrigin::Human,
            })
            .collect();
        self.put_node_with_edges(&node, &edges)
    }

    fn get(&self, _id: &SpecId) -> Result<Option<spec_db_core::SpecDoc>, SpecDbError> {
        Ok(None)
    }

    fn remove(&mut self, id: &SpecId) -> Result<(), SpecDbError> {
        self.remove_node(id)
    }

    fn list_ids(&self) -> Result<Vec<SpecId>, SpecDbError> {
        let mut ids = Vec::new();
        for guard in self.nodes.iter() {
            let (k, _v) = guard.into_inner().map_err(map_fjall_err)?;
            let key_str = std::str::from_utf8(&k)
                .map_err(|e| SpecDbError::GraphError(format!("node key not UTF-8: {e}")))?;
            ids.push(SpecId::try_new(key_str)?);
        }
        Ok(ids)
    }

    fn get_metadata(&self, id: &SpecId) -> Result<Option<SpecNode>, SpecDbError> {
        self.get_node(id)
    }
}
