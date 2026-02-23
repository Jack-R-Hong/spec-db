use std::collections::{HashSet, VecDeque};

use spec_db_core::SpecDbError;

pub fn bfs_traverse_indices<F>(
    start: usize,
    depth: Option<usize>,
    mut neighbors: F,
) -> Result<Vec<usize>, SpecDbError>
where
    F: FnMut(usize) -> Result<Vec<usize>, SpecDbError>,
{
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();

    visited.insert(start);
    queue.push_back((start, 0));

    while let Some((current, current_depth)) = queue.pop_front() {
        if depth.is_some_and(|limit| current_depth >= limit) {
            continue;
        }

        for neighbor in neighbors(current)? {
            if visited.insert(neighbor) {
                result.push(neighbor);
                queue.push_back((neighbor, current_depth + 1));
            }
        }
    }

    Ok(result)
}
