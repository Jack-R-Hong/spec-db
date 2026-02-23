use std::fs;
use std::path::Path;

use serde::Serialize;
use spec_db_core::{CausalEdge, EdgeOrigin, SpecDbError};

#[derive(Serialize)]
struct EdgeExport {
    source: String,
    target: String,
    edge_type: String,
    trust: f64,
    origin: String,
    created_at: Option<String>,
}

#[derive(Serialize)]
struct EdgesFile {
    edges: Vec<EdgeExport>,
}

pub fn export_ai_edges(edges: &[CausalEdge], lattice_dir: &Path) -> Result<(), SpecDbError> {
    let ai_edges: Vec<EdgeExport> = edges
        .iter()
        .filter(|e| e.origin == EdgeOrigin::Ai)
        .map(|e| EdgeExport {
            source: e.source.to_string(),
            target: e.target.to_string(),
            edge_type: e.edge_type.to_string(),
            trust: e.trust.value(),
            origin: e.origin.to_string(),
            created_at: e.created_at.clone(),
        })
        .collect();

    let file = EdgesFile { edges: ai_edges };
    let yaml = serde_yml::to_string(&file)
        .map_err(|e| SpecDbError::IngestError(format!("failed to serialize edges YAML: {e}")))?;

    fs::create_dir_all(lattice_dir)
        .map_err(|e| SpecDbError::IngestError(format!("failed to create .lattice dir: {e}")))?;

    let target_path = lattice_dir.join("edges.yaml");
    let tmp_path = lattice_dir.join("edges.yaml.tmp");

    fs::write(&tmp_path, yaml.as_bytes())
        .map_err(|e| SpecDbError::IngestError(format!("failed to write edges.yaml.tmp: {e}")))?;

    fs::rename(&tmp_path, &target_path)
        .map_err(|e| SpecDbError::IngestError(format!("failed to rename edges.yaml.tmp: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spec_db_core::{EdgeType, SpecId, TrustLevel};
    use tempfile::tempdir;

    fn ai_edge(source: &str, target: &str) -> CausalEdge {
        CausalEdge {
            source: SpecId::try_new(source).unwrap(),
            target: SpecId::try_new(target).unwrap(),
            edge_type: EdgeType::DependsOn,
            trust: TrustLevel::new(0.5),
            origin: EdgeOrigin::Ai,
            created_at: Some("2026-02-23T10:00:00Z".to_owned()),
        }
    }

    fn human_edge(source: &str, target: &str) -> CausalEdge {
        CausalEdge {
            source: SpecId::try_new(source).unwrap(),
            target: SpecId::try_new(target).unwrap(),
            edge_type: EdgeType::DependsOn,
            trust: TrustLevel::human(),
            origin: EdgeOrigin::Human,
            created_at: None,
        }
    }

    #[test]
    fn export_produces_correct_yaml_structure() {
        let dir = tempdir().unwrap();
        let lattice = dir.path().join(".lattice");
        let edges = vec![ai_edge("spec::svc::api", "spec::svc::auth")];

        export_ai_edges(&edges, &lattice).unwrap();

        let content = fs::read_to_string(lattice.join("edges.yaml")).unwrap();
        assert!(content.contains("source: spec::svc::api"));
        assert!(content.contains("target: spec::svc::auth"));
        assert!(content.contains("edge_type: depends_on"));
        assert!(content.contains("trust: 0.5"));
        assert!(content.contains("origin: ai"));
        assert!(content.contains("created_at: '2026-02-23T10:00:00Z'"));
    }

    #[test]
    fn only_ai_edges_appear_in_export() {
        let dir = tempdir().unwrap();
        let lattice = dir.path().join(".lattice");
        let edges = vec![
            ai_edge("spec::svc::api", "spec::svc::auth"),
            human_edge("spec::svc::db", "spec::svc::cache"),
        ];

        export_ai_edges(&edges, &lattice).unwrap();

        let content = fs::read_to_string(lattice.join("edges.yaml")).unwrap();
        assert!(content.contains("spec::svc::api"));
        assert!(!content.contains("spec::svc::db"));
    }

    #[test]
    fn human_edges_excluded() {
        let dir = tempdir().unwrap();
        let lattice = dir.path().join(".lattice");
        let edges = vec![human_edge("spec::svc::db", "spec::svc::cache")];

        export_ai_edges(&edges, &lattice).unwrap();

        let content = fs::read_to_string(lattice.join("edges.yaml")).unwrap();
        assert!(content.contains("edges: []"));
    }

    #[test]
    fn empty_edge_list_produces_empty_array() {
        let dir = tempdir().unwrap();
        let lattice = dir.path().join(".lattice");

        export_ai_edges(&[], &lattice).unwrap();

        let content = fs::read_to_string(lattice.join("edges.yaml")).unwrap();
        assert!(content.contains("edges: []"));
    }

    #[test]
    fn atomic_write_uses_temp_file() {
        let dir = tempdir().unwrap();
        let lattice = dir.path().join(".lattice");
        let edges = vec![ai_edge("spec::svc::api", "spec::svc::auth")];

        export_ai_edges(&edges, &lattice).unwrap();

        assert!(lattice.join("edges.yaml").exists());
        assert!(!lattice.join("edges.yaml.tmp").exists());
    }
}
