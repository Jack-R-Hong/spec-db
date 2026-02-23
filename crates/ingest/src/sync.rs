use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use git2::{
    Commit, Delta, DiffFindOptions, ObjectType, Oid, Repository, Tree, TreeWalkMode, TreeWalkResult,
};
use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{CausalGraph, SpecDbError, SpecId};
use spec_db_search::SearchIndex;

use crate::consistency::{ConsistencyReport, ConsistencyStatus, verify_cross_store_consistency};
use crate::parser::parse_spec;
use crate::pipeline::IngestPipeline;

const REBUILD_TMP_SUFFIX: &str = "_rebuild_tmp";
const OLD_SUFFIX: &str = "_old";
const SEARCH_METADATA_FILE: &str = "sync_metadata.json";

pub struct SyncReport {
    pub specs_ingested: usize,
    pub head_sha: String,
}

#[derive(Clone)]
pub struct StorePaths {
    pub tantivy_dir: PathBuf,
    pub fjall_dir: PathBuf,
}

pub struct GitSync {
    repo_path: PathBuf,
    specs_root: String,
    store_paths: StorePaths,
}

struct RebuildStaging {
    tantivy_tmp: PathBuf,
    fjall_tmp: PathBuf,
}

struct SwapPlan {
    live: PathBuf,
    staging: PathBuf,
    backup: PathBuf,
    had_live: bool,
}

impl GitSync {
    pub fn new(repo_path: PathBuf, specs_root: String, store_paths: StorePaths) -> Self {
        Self { repo_path, specs_root, store_paths }
    }

    pub fn full_rebuild(&self) -> Result<SyncReport, SpecDbError> {
        self.full_rebuild_internal(false)
    }

    fn full_rebuild_internal(&self, escalated: bool) -> Result<SyncReport, SpecDbError> {
        let _span = tracing::info_span!("spec_db.sync.full_rebuild").entered();

        let repo = Repository::open(&self.repo_path).map_err(map_git_error)?;
        let head_commit =
            repo.head().map_err(map_git_error)?.peel_to_commit().map_err(map_git_error)?;
        let head_sha = head_commit.id().to_string();

        let specs = self.discover_specs(&repo, &head_commit)?;
        let staging = self.prepare_rebuild_staging_dirs()?;

        let mut specs_ingested = 0usize;
        {
            let search = SearchIndex::open_or_create(&staging.tantivy_tmp)?;
            let fjall_store = Arc::new(FjallStore::open(&staging.fjall_tmp)?);
            let engine = CausalEngine::from_store(fjall_store.clone())?;
            let mut pipeline = IngestPipeline::new(search, engine);

            for (spec_path, content) in specs {
                match pipeline.add_spec(&content) {
                    Ok(_) => specs_ingested += 1,
                    Err(err) => {
                        tracing::warn!(spec_path = %spec_path, error = %err, "failed to ingest spec during full rebuild");
                    }
                }
            }

            fjall_store.set_last_sync_sha(&head_sha)?;
            fjall_store.set_doc_count(specs_ingested)?;
            write_search_metadata(&staging.tantivy_tmp, &head_sha, specs_ingested)?;

            drop(pipeline);
            drop(fjall_store);
        }

        self.atomic_swap_rebuild_outputs(&staging)?;

        let report = SyncReport { specs_ingested, head_sha };
        self.ensure_post_sync_consistency("full_rebuild", escalated)?;
        Ok(report)
    }

    pub fn incremental_sync(&self) -> Result<SyncReport, SpecDbError> {
        let _span = tracing::info_span!("spec_db.sync.incremental").entered();

        let repo = Repository::open(&self.repo_path).map_err(map_git_error)?;
        let head_commit =
            repo.head().map_err(map_git_error)?.peel_to_commit().map_err(map_git_error)?;
        let head_sha = head_commit.id().to_string();

        let search = SearchIndex::open_or_create(&self.store_paths.tantivy_dir)?;
        let fjall_store = Arc::new(FjallStore::open(&self.store_paths.fjall_dir)?);

        let Some(stored_sha) = fjall_store.last_sync_sha()? else {
            tracing::info!("missing last_sync_sha; escalating to full rebuild");
            return self.full_rebuild_internal(true);
        };

        if stored_sha == head_sha {
            return Ok(SyncReport { specs_ingested: 0, head_sha });
        }

        let old_oid = Oid::from_str(&stored_sha).map_err(map_git_error)?;
        let old_commit = repo.find_commit(old_oid).map_err(map_git_error)?;
        let old_tree = old_commit.tree().map_err(map_git_error)?;
        let new_tree = head_commit.tree().map_err(map_git_error)?;

        let mut pipeline =
            IngestPipeline::new(search, CausalEngine::from_store(fjall_store.clone())?);

        let mut expected_count = fjall_store.iter_nodes()?.len();
        let specs_root = normalize_specs_root(&self.specs_root);
        let mut specs_ingested = 0usize;

        {
            let _diff_span = tracing::info_span!("spec_db.sync.diff").entered();
            let mut diff = repo
                .diff_tree_to_tree(Some(&old_tree), Some(&new_tree), None)
                .map_err(map_git_error)?;
            let mut find_opts = DiffFindOptions::new();
            find_opts.renames(true);
            diff.find_similar(Some(&mut find_opts)).map_err(map_git_error)?;

            for delta in diff.deltas() {
                let status = delta.status();
                let old_path = delta.old_file().path();
                let new_path = delta.new_file().path();

                let old_in_scope =
                    old_path.is_some_and(|path| is_spec_markdown_path(path, &specs_root));
                let new_in_scope =
                    new_path.is_some_and(|path| is_spec_markdown_path(path, &specs_root));

                match status {
                    Delta::Added => {
                        if !new_in_scope {
                            continue;
                        }
                        if let Some(path) = new_path {
                            let content = read_blob_at_path(&repo, &new_tree, &path_string(path))?;
                            pipeline.add_spec(&content)?;
                            specs_ingested += 1;
                            expected_count += 1;
                        }
                    }
                    Delta::Modified => {
                        if !old_in_scope && !new_in_scope {
                            continue;
                        }
                        if let Some(path) = old_path.filter(|_| old_in_scope) {
                            let old_content =
                                read_blob_at_path(&repo, &old_tree, &path_string(path))?;
                            if remove_spec_from_content(&mut pipeline, &old_content)? {
                                expected_count = expected_count.saturating_sub(1);
                            }
                        }
                        if let Some(path) = new_path.filter(|_| new_in_scope) {
                            let new_content =
                                read_blob_at_path(&repo, &new_tree, &path_string(path))?;
                            pipeline.add_spec(&new_content)?;
                            specs_ingested += 1;
                            expected_count += 1;
                        }
                    }
                    Delta::Renamed => {
                        if !old_in_scope && !new_in_scope {
                            continue;
                        }
                        if let Some(path) = old_path.filter(|_| old_in_scope) {
                            let old_content =
                                read_blob_at_path(&repo, &old_tree, &path_string(path))?;
                            if remove_spec_from_content(&mut pipeline, &old_content)? {
                                expected_count = expected_count.saturating_sub(1);
                            }
                        }
                        if let Some(path) = new_path.filter(|_| new_in_scope) {
                            let new_content =
                                read_blob_at_path(&repo, &new_tree, &path_string(path))?;
                            pipeline.add_spec(&new_content)?;
                            specs_ingested += 1;
                            expected_count += 1;
                        }
                    }
                    Delta::Deleted => {
                        if !old_in_scope {
                            continue;
                        }
                        if let Some(path) = old_path {
                            let old_content =
                                read_blob_at_path(&repo, &old_tree, &path_string(path))?;
                            if remove_spec_from_content(&mut pipeline, &old_content)? {
                                expected_count = expected_count.saturating_sub(1);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let actual_count = fjall_store.iter_nodes()?.len();
        if expected_count != actual_count {
            tracing::warn!(
                expected_count,
                actual_count,
                "incremental sync count divergence; escalating to full rebuild"
            );
            return self.full_rebuild_internal(true);
        }

        fjall_store.set_last_sync_sha(&head_sha)?;
        fjall_store.set_doc_count(actual_count)?;
        write_search_metadata(&self.store_paths.tantivy_dir, &head_sha, actual_count)?;

        let persisted_count = fjall_store.doc_count()?.ok_or_else(|| {
            SpecDbError::ConsistencyError("missing doc_count after incremental sync".to_string())
        })?;
        if persisted_count != actual_count {
            tracing::warn!(
                persisted_count,
                actual_count,
                "persisted doc_count diverged; escalating to full rebuild"
            );
            return self.full_rebuild_internal(true);
        }

        drop(pipeline);
        drop(fjall_store);

        let sync_report = SyncReport { specs_ingested, head_sha };
        match self.load_consistency_report()? {
            ConsistencyReport { status: ConsistencyStatus::InSync, .. }
            | ConsistencyReport { status: ConsistencyStatus::NeverSynced, .. } => Ok(sync_report),
            report => {
                tracing::warn!(
                    status = ?report.status,
                    tantivy_sha = ?report.tantivy.git_sha,
                    tantivy_doc_count = ?report.tantivy.doc_count,
                    fjall_sha = ?report.fjall.git_sha,
                    fjall_doc_count = ?report.fjall.doc_count,
                    "post-incremental consistency drift detected; escalating to full rebuild"
                );
                self.full_rebuild_internal(true)
            }
        }
    }

    fn load_consistency_report(&self) -> Result<ConsistencyReport, SpecDbError> {
        let fjall_store = FjallStore::open(&self.store_paths.fjall_dir)?;
        let fjall_sha = fjall_store.last_sync_sha()?;
        let fjall_count = fjall_store.doc_count()?;

        let search = SearchIndex::open_or_create(&self.store_paths.tantivy_dir)?;
        let tantivy_doc_count = usize::try_from(search.doc_count()?).map_err(|_| {
            SpecDbError::ConsistencyError(
                "tantivy doc_count does not fit into usize for consistency check".to_string(),
            )
        })?;

        let (tantivy_sha, tantivy_count) = match search.sync_metadata()? {
            Some((sha, count)) => (Some(sha), Some(count)),
            None => (None, Some(tantivy_doc_count)),
        };

        verify_cross_store_consistency(fjall_sha, fjall_count, tantivy_sha, tantivy_count)
    }

    fn ensure_post_sync_consistency(
        &self,
        trigger: &str,
        escalated: bool,
    ) -> Result<(), SpecDbError> {
        let report = self.load_consistency_report()?;
        match report.status {
            ConsistencyStatus::InSync | ConsistencyStatus::NeverSynced => Ok(()),
            ConsistencyStatus::Drift { .. } => {
                tracing::warn!(
                    trigger,
                    escalated,
                    tantivy_sha = ?report.tantivy.git_sha,
                    tantivy_doc_count = ?report.tantivy.doc_count,
                    fjall_sha = ?report.fjall.git_sha,
                    fjall_doc_count = ?report.fjall.doc_count,
                    "cross-store consistency drift detected"
                );
                let message = if escalated {
                    format!(
                        "cross-store consistency drift persists after escalation: {}",
                        format_drift_details(&report)
                    )
                } else {
                    format!(
                        "cross-store consistency drift detected: {}",
                        format_drift_details(&report)
                    )
                };
                Err(SpecDbError::ConsistencyError(message))
            }
        }
    }

    fn discover_specs(
        &self,
        repo: &Repository,
        commit: &Commit<'_>,
    ) -> Result<Vec<(String, String)>, SpecDbError> {
        let _span = tracing::info_span!("spec_db.sync.tree_walk").entered();

        let tree = commit.tree().map_err(map_git_error)?;
        let specs_root = normalize_specs_root(&self.specs_root);
        let mut specs = Vec::new();
        let mut walk_error: Option<SpecDbError> = None;

        tree.walk(TreeWalkMode::PreOrder, |root, entry| {
            if entry.kind() != Some(ObjectType::Blob) {
                return TreeWalkResult::Ok;
            }

            let Some(name) = entry.name() else {
                return TreeWalkResult::Ok;
            };

            let full_path = format!("{root}{name}");
            if !full_path.starts_with(&specs_root) || !full_path.ends_with(".md") {
                return TreeWalkResult::Ok;
            }

            let blob = match repo.find_blob(entry.id()) {
                Ok(blob) => blob,
                Err(err) => {
                    walk_error = Some(map_git_error(err));
                    return TreeWalkResult::Abort;
                }
            };

            let content = match String::from_utf8(blob.content().to_vec()) {
                Ok(content) => content,
                Err(err) => {
                    walk_error = Some(SpecDbError::SyncError(format!(
                        "spec blob at {full_path} is not UTF-8: {err}"
                    )));
                    return TreeWalkResult::Abort;
                }
            };

            specs.push((full_path, content));
            TreeWalkResult::Ok
        })
        .map_err(map_git_error)?;

        if let Some(err) = walk_error {
            return Err(err);
        }

        specs.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(specs)
    }

    fn prepare_rebuild_staging_dirs(&self) -> Result<RebuildStaging, SpecDbError> {
        let tantivy_tmp = suffixed_path(&self.store_paths.tantivy_dir, REBUILD_TMP_SUFFIX)?;
        let fjall_tmp = suffixed_path(&self.store_paths.fjall_dir, REBUILD_TMP_SUFFIX)?;

        remove_dir_if_exists(&tantivy_tmp)?;
        remove_dir_if_exists(&fjall_tmp)?;

        fs::create_dir_all(&tantivy_tmp).map_err(io_sync_error)?;
        fs::create_dir_all(&fjall_tmp).map_err(io_sync_error)?;

        Ok(RebuildStaging { tantivy_tmp, fjall_tmp })
    }

    fn atomic_swap_rebuild_outputs(&self, staging: &RebuildStaging) -> Result<(), SpecDbError> {
        let _span = tracing::info_span!("spec_db.sync.atomic_swap").entered();

        let mut plans = vec![
            SwapPlan {
                live: self.store_paths.tantivy_dir.clone(),
                staging: staging.tantivy_tmp.clone(),
                backup: suffixed_path(&self.store_paths.tantivy_dir, OLD_SUFFIX)?,
                had_live: false,
            },
            SwapPlan {
                live: self.store_paths.fjall_dir.clone(),
                staging: staging.fjall_tmp.clone(),
                backup: suffixed_path(&self.store_paths.fjall_dir, OLD_SUFFIX)?,
                had_live: false,
            },
        ];

        for plan in &plans {
            remove_dir_if_exists(&plan.backup)?;
        }

        for plan in &mut plans {
            if plan.live.exists() {
                fs::rename(&plan.live, &plan.backup).map_err(io_sync_error)?;
                plan.had_live = true;
            }
        }

        for (idx, plan) in plans.iter().enumerate() {
            if let Err(err) = fs::rename(&plan.staging, &plan.live) {
                rollback_swap(&plans, idx);
                return Err(io_sync_error_with_context(
                    format!(
                        "failed to promote staging dir {} to {}",
                        plan.staging.display(),
                        plan.live.display()
                    ),
                    err,
                ));
            }
        }

        for plan in &plans {
            if plan.had_live {
                remove_dir_if_exists(&plan.backup)?;
            }
        }

        Ok(())
    }
}

fn format_drift_details(report: &ConsistencyReport) -> String {
    format!(
        "tantivy(sha={:?}, doc_count={:?}) fjall(sha={:?}, doc_count={:?})",
        report.tantivy.git_sha,
        report.tantivy.doc_count,
        report.fjall.git_sha,
        report.fjall.doc_count
    )
}

fn remove_spec_from_content<S, G>(
    pipeline: &mut IngestPipeline<S, G>,
    content: &str,
) -> Result<bool, SpecDbError>
where
    S: spec_db_core::SearchEngine,
    G: CausalGraph,
{
    let old_id = extract_spec_id_from_content(content)?;
    if pipeline.graph().get_node(&old_id)?.is_some() {
        pipeline.remove_spec(&old_id)?;
        return Ok(true);
    }
    Ok(false)
}

fn read_blob_at_path(
    repo: &Repository,
    tree: &Tree<'_>,
    path: &str,
) -> Result<String, SpecDbError> {
    let entry = tree.get_path(Path::new(path)).map_err(map_git_error)?;
    let blob = repo.find_blob(entry.id()).map_err(map_git_error)?;
    String::from_utf8(blob.content().to_vec())
        .map_err(|err| SpecDbError::SyncError(format!("blob at {path} is not UTF-8: {err}")))
}

fn extract_spec_id_from_content(content: &str) -> Result<SpecId, SpecDbError> {
    Ok(parse_spec(content)?.id)
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn is_spec_markdown_path(path: &Path, specs_root: &str) -> bool {
    let normalized = path_string(path);
    normalized.starts_with(specs_root) && normalized.ends_with(".md")
}

fn write_search_metadata(index_dir: &Path, sha: &str, doc_count: usize) -> Result<(), SpecDbError> {
    let metadata = format!("{{\"last_sync_sha\":\"{sha}\",\"doc_count\":{doc_count}}}\n");
    fs::write(index_dir.join(SEARCH_METADATA_FILE), metadata).map_err(io_sync_error)
}

fn rollback_swap(plans: &[SwapPlan], promoted_count: usize) {
    for idx in (0..promoted_count).rev() {
        let plan = &plans[idx];
        if plan.live.exists() {
            let _ = fs::remove_dir_all(&plan.live);
        }
        if plan.had_live {
            let _ = fs::rename(&plan.backup, &plan.live);
        }
    }

    for plan in plans.iter().skip(promoted_count) {
        if plan.had_live && !plan.live.exists() {
            let _ = fs::rename(&plan.backup, &plan.live);
        }
    }
}

fn remove_dir_if_exists(path: &Path) -> Result<(), SpecDbError> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(io_sync_error)?;
    }
    Ok(())
}

fn normalize_specs_root(specs_root: &str) -> String {
    let trimmed = specs_root.trim_start_matches('/');
    if trimmed.ends_with('/') { trimmed.to_owned() } else { format!("{trimmed}/") }
}

fn suffixed_path(path: &Path, suffix: &str) -> Result<PathBuf, SpecDbError> {
    let file_name = path.file_name().ok_or_else(|| {
        SpecDbError::SyncError(format!("path has no terminal component: {}", path.display()))
    })?;
    let mut next = file_name.to_os_string();
    next.push(suffix);
    Ok(path.with_file_name(next))
}

fn map_git_error(err: git2::Error) -> SpecDbError {
    SpecDbError::SyncError(err.to_string())
}

fn io_sync_error(err: std::io::Error) -> SpecDbError {
    SpecDbError::SyncError(err.to_string())
}

fn io_sync_error_with_context(context: String, err: std::io::Error) -> SpecDbError {
    SpecDbError::SyncError(format!("{context}: {err}"))
}
