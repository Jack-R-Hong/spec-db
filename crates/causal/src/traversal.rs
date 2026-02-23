use std::collections::{HashMap, HashSet, VecDeque};

use spec_db_core::{CausalEdge, SpecDbError, SpecId};

pub fn bfs_traverse(
    adjacency: &HashMap<String, Vec<CausalEdge>>,
    start: &SpecId,
    depth: Option<usize>,
    get_neighbor: fn(&CausalEdge) -> &SpecId,
) -> Result<Vec<SpecId>, SpecDbError> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();

    visited.insert(start.clone());
    queue.push_back((start.clone(), 0));

    while let Some((current, current_depth)) = queue.pop_front() {
        if depth.is_some_and(|limit| current_depth >= limit) {
            continue;
        }

        if let Some(edges) = adjacency.get(current.as_ref()) {
            for edge in edges {
                let neighbor = get_neighbor(edge).clone();
                if visited.insert(neighbor.clone()) {
                    result.push(neighbor.clone());
                    queue.push_back((neighbor, current_depth + 1));
                }
            }
        }
    }

    Ok(result)
}
