use std::path::Path;

use spec_db_core::{CausalEdge, SpecDbError, SpecId, SpecNode, SpecStore};

/// Identify the process holding a flock on `lock_path` by scanning `/proc/locks`.
///
/// Returns `(pid, command_line)` when the holder is found.
#[cfg(target_os = "linux")]
fn find_lock_holder(lock_path: &Path) -> Option<(u32, String)> {
    use std::os::unix::fs::MetadataExt;

    let meta = std::fs::metadata(lock_path).ok()?;
    let dev = meta.dev();
    let ino = meta.ino();

    // Extract major/minor matching the kernel's %02x:%02x format in /proc/locks.
    let target_major = ((dev >> 8) & 0xfff) | ((dev >> 32) & !0xfff);
    let target_minor = (dev & 0xff) | ((dev >> 12) & !0xff);

    let contents = std::fs::read_to_string("/proc/locks").ok()?;
    for line in contents.lines() {
        // "1: FLOCK  ADVISORY  WRITE 12345 fd:00:4194827 0 EOF"
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 8 {
            continue;
        }

        let dev_ino: Vec<&str> = fields[5].split(':').collect();
        if dev_ino.len() != 3 {
            continue;
        }

        let major = match u64::from_str_radix(dev_ino[0], 16) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let minor = match u64::from_str_radix(dev_ino[1], 16) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let file_ino: u64 = match dev_ino[2].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        if major == target_major && minor == target_minor && file_ino == ino {
            let pid: u32 = match fields[4].parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let cmd = std::fs::read_to_string(format!("/proc/{pid}/cmdline"))
                .unwrap_or_default()
                .replace('\0', " ");
            let cmd = cmd.trim();
            let cmd = if cmd.is_empty() { "unknown" } else { cmd };
            return Some((pid, cmd.to_owned()));
        }
    }

    None
}

#[cfg(not(target_os = "linux"))]
fn find_lock_holder(_lock_path: &Path) -> Option<(u32, String)> {
    None
}

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
        let db = fjall::Database::builder(path).open().map_err(|e| {
            if matches!(e, fjall::Error::Locked) {
                let lock_path = path.join("lock");
                match find_lock_holder(&lock_path) {
                    Some((pid, cmd)) => SpecDbError::GraphError(format!(
                        "database is locked by PID {pid} ({cmd}). \
                         Stop that process or, if it crashed, remove: {}",
                        lock_path.display()
                    )),
                    None => SpecDbError::GraphError(format!(
                        "database is locked (holder not found — likely stale). \
                         Remove the lock file and retry: rm {}",
                        lock_path.display()
                    )),
                }
            } else {
                map_fjall_err(e)
            }
        })?;
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
                edge_type: spec_db_core::EdgeType::DependsOn,
                trust: spec_db_core::TrustLevel::human(),
                origin: spec_db_core::EdgeOrigin::Human,
                created_at: None,
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
