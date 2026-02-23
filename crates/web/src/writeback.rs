use std::path::{Path, PathBuf};

use git2::{IndexAddOption, Repository, Signature};
use serde::Deserialize;
use spec_db_core::{SpecDbError, SpecId};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WriteBackOp {
    EdgeAdd { source: String, target: String },
    EdgeRemove { source: String, target: String },
    FrontmatterEdit { spec_id: String, changes: FrontmatterChanges },
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FrontmatterChanges {
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub owner: Option<String>,
    pub depends_on: Option<Vec<String>>,
}

pub struct WriteBackPipeline {
    repo_path: PathBuf,
    specs_dir: String,
}

impl WriteBackPipeline {
    pub fn new(repo_path: PathBuf, specs_dir: String) -> Self {
        Self { repo_path, specs_dir }
    }

    pub fn apply(&self, op: &WriteBackOp) -> Result<String, SpecDbError> {
        let _span = tracing::info_span!("spec_db.web.writeback.apply").entered();

        match op {
            WriteBackOp::EdgeAdd { source, target } => {
                let source_id = SpecId::try_new(source)?;
                let target_id = SpecId::try_new(target)?;
                self.apply_edge_add(&source_id, &target_id)
            }
            WriteBackOp::EdgeRemove { source, target } => {
                let source_id = SpecId::try_new(source)?;
                let target_id = SpecId::try_new(target)?;
                self.apply_edge_remove(&source_id, &target_id)
            }
            WriteBackOp::FrontmatterEdit { spec_id, changes } => {
                let id = SpecId::try_new(spec_id)?;
                self.apply_frontmatter_edit(&id, changes)
            }
        }
    }

    fn apply_edge_add(&self, source: &SpecId, target: &SpecId) -> Result<String, SpecDbError> {
        let spec_path = self.find_spec_file(source)?;
        let content = std::fs::read_to_string(&spec_path).map_err(io_err)?;

        let (mut frontmatter, body) = split_frontmatter_body(&content)?;
        let deps = extract_depends_on(&frontmatter);
        let target_str = target.to_string();

        if deps.contains(&target_str) {
            return Err(SpecDbError::IngestError(format!(
                "edge already exists: {} -> {}",
                source, target
            )));
        }

        let mut new_deps = deps;
        new_deps.push(target_str);
        frontmatter = set_depends_on(&frontmatter, &new_deps);

        let new_content = reassemble(&frontmatter, &body);
        std::fs::write(&spec_path, &new_content).map_err(io_err)?;

        let commit_msg = format!("lattice: add depends_on edge from {} to {}", source, target);
        self.git_commit(&spec_path, &commit_msg)
    }

    fn apply_edge_remove(&self, source: &SpecId, target: &SpecId) -> Result<String, SpecDbError> {
        let spec_path = self.find_spec_file(source)?;
        let content = std::fs::read_to_string(&spec_path).map_err(io_err)?;

        let (mut frontmatter, body) = split_frontmatter_body(&content)?;
        let deps = extract_depends_on(&frontmatter);
        let target_str = target.to_string();

        let new_deps: Vec<String> = deps.into_iter().filter(|d| d != &target_str).collect();
        frontmatter = set_depends_on(&frontmatter, &new_deps);

        let new_content = reassemble(&frontmatter, &body);
        std::fs::write(&spec_path, &new_content).map_err(io_err)?;

        let commit_msg = format!("lattice: remove depends_on edge from {} to {}", source, target);
        self.git_commit(&spec_path, &commit_msg)
    }

    fn apply_frontmatter_edit(
        &self,
        spec_id: &SpecId,
        changes: &FrontmatterChanges,
    ) -> Result<String, SpecDbError> {
        let spec_path = self.find_spec_file(spec_id)?;
        let content = std::fs::read_to_string(&spec_path).map_err(io_err)?;

        let (mut frontmatter, body) = split_frontmatter_body(&content)?;

        if let Some(title) = &changes.title {
            frontmatter = set_field(&frontmatter, "title", &format!("\"{}\"", title));
        }
        if let Some(owner) = &changes.owner {
            frontmatter = set_field(&frontmatter, "owner", &format!("\"{}\"", owner));
        }
        if let Some(tags) = &changes.tags {
            let yaml_arr = format!(
                "[{}]",
                tags.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ")
            );
            frontmatter = set_field(&frontmatter, "tags", &yaml_arr);
        }
        if let Some(deps) = &changes.depends_on {
            frontmatter = set_depends_on(&frontmatter, deps);
        }

        let new_content = reassemble(&frontmatter, &body);
        std::fs::write(&spec_path, &new_content).map_err(io_err)?;

        let commit_msg = format!("lattice: edit frontmatter of {}", spec_id);
        self.git_commit(&spec_path, &commit_msg)
    }

    fn find_spec_file(&self, spec_id: &SpecId) -> Result<PathBuf, SpecDbError> {
        let specs_dir = self.repo_path.join(&self.specs_dir);
        find_spec_file_recursive(&specs_dir, spec_id)
    }

    fn git_commit(&self, file_path: &Path, message: &str) -> Result<String, SpecDbError> {
        let repo = Repository::open(&self.repo_path).map_err(git_err)?;

        let mut index = repo.index().map_err(git_err)?;
        let relative = file_path
            .strip_prefix(&self.repo_path)
            .map_err(|e| SpecDbError::SyncError(format!("path not relative to repo: {}", e)))?;
        index.add_all([relative], IndexAddOption::DEFAULT, None).map_err(git_err)?;
        index.write().map_err(git_err)?;
        let tree_oid = index.write_tree().map_err(git_err)?;
        let tree = repo.find_tree(tree_oid).map_err(git_err)?;

        let sig = Signature::now("Lattice", "lattice@localhost").map_err(git_err)?;

        let head = repo.head().map_err(git_err)?;
        let parent = head.peel_to_commit().map_err(git_err)?;

        let commit_oid =
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent]).map_err(git_err)?;

        Ok(commit_oid.to_string())
    }

    pub fn undo(&self, commit_sha: &str) -> Result<(), SpecDbError> {
        let _span = tracing::info_span!("spec_db.web.writeback.undo").entered();

        let repo = Repository::open(&self.repo_path).map_err(git_err)?;
        let oid = git2::Oid::from_str(commit_sha).map_err(git_err)?;
        let commit = repo.find_commit(oid).map_err(git_err)?;

        repo.revert(&commit, None).map_err(git_err)?;

        let index = repo.index().map_err(git_err)?;
        if index.has_conflicts() {
            repo.cleanup_state().map_err(git_err)?;
            return Err(SpecDbError::SyncError(
                "undo failed: revert produced conflicts".to_string(),
            ));
        }

        let mut index = repo.index().map_err(git_err)?;
        let tree_oid = index.write_tree().map_err(git_err)?;
        let tree = repo.find_tree(tree_oid).map_err(git_err)?;
        let sig = Signature::now("Lattice", "lattice@localhost").map_err(git_err)?;
        let head = repo.head().map_err(git_err)?;
        let parent = head.peel_to_commit().map_err(git_err)?;

        let revert_msg = format!("lattice: undo {}", &commit_sha[..7.min(commit_sha.len())]);
        repo.commit(Some("HEAD"), &sig, &sig, &revert_msg, &tree, &[&parent]).map_err(git_err)?;

        repo.cleanup_state().map_err(git_err)?;

        Ok(())
    }
}

fn split_frontmatter_body(content: &str) -> Result<(String, String), SpecDbError> {
    let lines: Vec<&str> = content.lines().collect();
    let mut first_delim = None;
    let mut second_delim = None;

    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "---" {
            if first_delim.is_none() {
                first_delim = Some(i);
            } else {
                second_delim = Some(i);
                break;
            }
        }
    }

    let (first, second) = match (first_delim, second_delim) {
        (Some(f), Some(s)) => (f, s),
        _ => {
            return Err(SpecDbError::IngestError("missing frontmatter delimiters".to_string()));
        }
    };

    let frontmatter = lines[first + 1..second].join("\n");

    let mut byte_offset = 0;
    for (i, line) in content.split_inclusive('\n').enumerate() {
        byte_offset += line.len();
        if i == second {
            break;
        }
    }
    if byte_offset > content.len() {
        byte_offset = content.len();
    }
    let body = content[byte_offset..].to_string();

    Ok((frontmatter, body))
}

fn reassemble(frontmatter: &str, body: &str) -> String {
    let mut result = String::new();
    result.push_str("---\n");
    result.push_str(frontmatter);
    if !frontmatter.ends_with('\n') {
        result.push('\n');
    }
    result.push_str("---\n");
    result.push_str(body);
    result
}

fn extract_depends_on(frontmatter: &str) -> Vec<String> {
    #[derive(Deserialize, Default)]
    struct Partial {
        #[serde(default)]
        depends_on: Option<Vec<String>>,
    }

    serde_yml::from_str::<Partial>(frontmatter).ok().and_then(|p| p.depends_on).unwrap_or_default()
}

fn set_depends_on(frontmatter: &str, deps: &[String]) -> String {
    let yaml_value = if deps.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", deps.iter().map(|d| format!("\"{}\"", d)).collect::<Vec<_>>().join(", "))
    };
    set_field(frontmatter, "depends_on", &yaml_value)
}

fn set_field(frontmatter: &str, key: &str, value: &str) -> String {
    let prefix = format!("{key}:");
    let mut replacing_block_seq = false;
    let mut result_lines: Vec<String> = Vec::new();

    for line in frontmatter.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(&prefix) {
            result_lines.push(format!("{}: {}", key, value));
            replacing_block_seq = true;
        } else if replacing_block_seq && trimmed.starts_with("- ") && !trimmed.contains(':') {
            continue;
        } else {
            replacing_block_seq = false;
            result_lines.push(line.to_string());
        }
    }

    if !result_lines.iter().any(|l| l.trim_start().starts_with(&prefix)) {
        result_lines.push(format!("{}: {}", key, value));
    }

    result_lines.join("\n")
}

fn find_spec_file_recursive(dir: &Path, target_id: &SpecId) -> Result<PathBuf, SpecDbError> {
    if !dir.exists() {
        return Err(SpecDbError::SyncError(format!(
            "specs directory not found: {}",
            dir.display()
        )));
    }

    for entry in walkdir(dir)? {
        if entry.extension().is_some_and(|ext| ext == "md")
            && let Ok(content) = std::fs::read_to_string(&entry)
            && let Ok(doc) = spec_db_ingest::parse_spec(&content)
            && doc.id == *target_id
        {
            return Ok(entry);
        }
    }

    Err(SpecDbError::SyncError(format!("spec file not found for id: {}", target_id)))
}

fn walkdir(dir: &Path) -> Result<Vec<PathBuf>, SpecDbError> {
    let mut files = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(io_err)?;
    for entry in entries {
        let entry = entry.map_err(io_err)?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(walkdir(&path)?);
        } else {
            files.push(path);
        }
    }
    Ok(files)
}

fn git_err(e: git2::Error) -> SpecDbError {
    SpecDbError::SyncError(e.to_string())
}

fn io_err(e: std::io::Error) -> SpecDbError {
    SpecDbError::SyncError(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_frontmatter_body_basic() {
        let content = "---\nid: \"spec::a::b\"\ntitle: \"Test\"\n---\n\n# Body\n";
        let (fm, body) = split_frontmatter_body(content).unwrap();
        assert!(fm.contains("id:"));
        assert!(fm.contains("title:"));
        assert!(body.contains("# Body"));
    }

    #[test]
    fn split_frontmatter_body_no_body() {
        let content = "---\nid: \"spec::a::b\"\n---\n";
        let (fm, body) = split_frontmatter_body(content).unwrap();
        assert!(fm.contains("id:"));
        assert!(body.trim().is_empty());
    }

    #[test]
    fn split_frontmatter_body_missing_delimiters() {
        let content = "just some text";
        assert!(split_frontmatter_body(content).is_err());
    }

    #[test]
    fn reassemble_roundtrip() {
        let fm = "id: \"spec::a::b\"\ntitle: \"Test\"";
        let body = "\n# Heading\n\nSome content.\n";
        let assembled = reassemble(fm, body);
        assert!(assembled.starts_with("---\n"));
        assert!(assembled.contains("---\n\n# Heading"));
    }

    #[test]
    fn extract_depends_on_present() {
        let fm = "id: \"spec::a::b\"\ndepends_on: [\"spec::c::d\", \"spec::e::f\"]";
        let deps = extract_depends_on(fm);
        assert_eq!(deps, vec!["spec::c::d", "spec::e::f"]);
    }

    #[test]
    fn extract_depends_on_missing() {
        let fm = "id: \"spec::a::b\"\ntitle: \"Test\"";
        let deps = extract_depends_on(fm);
        assert!(deps.is_empty());
    }

    #[test]
    fn set_field_replaces_existing() {
        let fm = "id: \"spec::a::b\"\ntitle: \"Old Title\"\nversion: 1";
        let result = set_field(fm, "title", "\"New Title\"");
        assert!(result.contains("title: \"New Title\""));
        assert!(!result.contains("Old Title"));
        assert!(result.contains("version: 1"));
    }

    #[test]
    fn set_field_appends_if_missing() {
        let fm = "id: \"spec::a::b\"\ntitle: \"Test\"";
        let result = set_field(fm, "owner", "\"alice\"");
        assert!(result.contains("owner: \"alice\""));
        assert!(result.contains("id:"));
    }

    #[test]
    fn set_depends_on_replaces_inline_array() {
        let fm = "id: \"spec::a::b\"\ndepends_on: [\"spec::old::dep\"]\ntags: [\"auth\"]";
        let result = set_depends_on(fm, &["spec::new::dep".to_string()]);
        assert!(result.contains("depends_on: [\"spec::new::dep\"]"));
        assert!(!result.contains("spec::old::dep"));
        assert!(result.contains("tags:"));
    }

    #[test]
    fn set_depends_on_empty() {
        let fm = "id: \"spec::a::b\"\ndepends_on: [\"spec::old::dep\"]";
        let result = set_depends_on(fm, &[]);
        assert!(result.contains("depends_on: []"));
    }

    #[test]
    fn writeback_pipeline_constructs() {
        let pipeline = WriteBackPipeline::new(PathBuf::from("/tmp"), "specs".to_string());
        assert_eq!(pipeline.specs_dir, "specs");
    }
}
