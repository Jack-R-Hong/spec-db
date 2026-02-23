use std::sync::Arc;

use rmcp::model::{
    GetPromptRequestParams, GetPromptResult, Prompt, PromptArgument, PromptMessage,
    PromptMessageRole,
};
use serde_json::{Value, json};
use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{CausalGraph, SpecDbError, SpecId};
use spec_db_search::SearchIndex;

use crate::tools::ToolHandler;

pub fn prompt_definitions() -> Vec<Prompt> {
    let spec_id_arg = PromptArgument {
        name: "spec_id".to_owned(),
        title: Some("Spec ID".to_owned()),
        description: Some(
            "The spec identifier to analyse (e.g. spec::auth::jwt-validation)".to_owned(),
        ),
        required: Some(true),
    };

    vec![
        Prompt::new(
            "impact_analysis",
            Some("Guides structured impact assessment for a spec before proposing changes"),
            Some(vec![spec_id_arg.clone()]),
        ),
        Prompt::new(
            "spec_review",
            Some("Guides structured spec review with quality checklist"),
            Some(vec![spec_id_arg]),
        ),
    ]
}

pub fn resolve_prompt(
    handler: &ToolHandler,
    params: &GetPromptRequestParams,
) -> Result<GetPromptResult, SpecDbError> {
    match params.name.as_str() {
        "impact_analysis" => resolve_impact_analysis(handler, params),
        "spec_review" => resolve_spec_review(handler, params),
        _ => Err(SpecDbError::ConfigError(format!(
            "mcp_error::{}",
            json!({
                "error_type": "not_found",
                "message": "Prompt not found",
                "context": { "name": params.name },
            })
        ))),
    }
}

fn extract_spec_id(params: &GetPromptRequestParams) -> Result<SpecId, SpecDbError> {
    let spec_id_raw = params
        .arguments
        .as_ref()
        .and_then(|args| args.get("spec_id"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            SpecDbError::IngestError(format!(
                "mcp_error::{}",
                json!({
                    "error_type": "validation_error",
                    "message": "Missing required argument: spec_id",
                    "context": Value::Null,
                })
            ))
        })?;

    SpecId::try_new(spec_id_raw).map_err(|_| {
        SpecDbError::IngestError(format!(
            "mcp_error::{}",
            json!({
                "error_type": "validation_error",
                "message": "Invalid spec id",
                "context": { "id": spec_id_raw },
            })
        ))
    })
}

fn lookup_spec(
    handler: &ToolHandler,
    spec_id: &SpecId,
) -> Result<spec_db_core::SpecDoc, SpecDbError> {
    let search = SearchIndex::open_or_create(&handler.tantivy_dir)?;
    search.get_spec(spec_id)?.ok_or_else(|| {
        SpecDbError::IngestError(format!(
            "mcp_error::{}",
            json!({
                "error_type": "not_found",
                "message": "Spec not found",
                "context": { "id": spec_id.to_string() },
            })
        ))
    })
}

fn resolve_impact_analysis(
    handler: &ToolHandler,
    params: &GetPromptRequestParams,
) -> Result<GetPromptResult, SpecDbError> {
    let spec_id = extract_spec_id(params)?;
    let spec = lookup_spec(handler, &spec_id)?;

    let store = Arc::new(FjallStore::open(&handler.fjall_dir)?);
    let graph = CausalEngine::from_store(store)?;

    let downstream_ids = graph.trace_impact(&spec_id, Some(1))?;
    let upstream_ids = graph.find_dependencies(&spec_id, Some(1))?;

    let downstream_edges = graph.edges_from(&spec_id)?;
    let upstream_edges = graph.edges_to(&spec_id)?;

    let is_isolated = downstream_edges.is_empty() && upstream_edges.is_empty();

    let mut messages = Vec::new();

    let section1 = format!(
        "# Impact Analysis: {id}\n\n\
         ## Spec Metadata\n\
         - **ID**: {id}\n\
         - **Title**: {title}\n\
         - **Version**: {version}\n\
         - **Tags**: {tags}\n\
         - **Created**: {created}\n\n\
         ## Spec Content\n\n{body}",
        id = spec.id,
        title = spec.title,
        version = spec.version,
        tags = if spec.tags.is_empty() { "none".to_owned() } else { spec.tags.join(", ") },
        created = spec.created,
        body = spec.body,
    );
    messages.push(PromptMessage::new_text(PromptMessageRole::User, section1));

    let section2 = if downstream_edges.is_empty() {
        "## Downstream Impact\n\nNo downstream dependents found.".to_owned()
    } else {
        let mut s = "## Downstream Impact\n\n\
                      Specs that depend on this spec:\n\n"
            .to_owned();
        for edge in &downstream_edges {
            s.push_str(&format!(
                "- **{}** (type: {}, trust: {:.2}, origin: {})\n",
                edge.target,
                edge.edge_type,
                edge.trust.value(),
                edge.origin,
            ));
        }
        let _ = downstream_ids;
        s
    };
    messages.push(PromptMessage::new_text(PromptMessageRole::User, section2));

    let section3 = if upstream_edges.is_empty() {
        "## Upstream Dependencies\n\nNo upstream dependencies found.".to_owned()
    } else {
        let mut s = "## Upstream Dependencies\n\n\
                      Specs this spec depends on:\n\n"
            .to_owned();
        for edge in &upstream_edges {
            s.push_str(&format!(
                "- **{}** (type: {}, trust: {:.2}, origin: {})\n",
                edge.source,
                edge.edge_type,
                edge.trust.value(),
                edge.origin,
            ));
        }
        let _ = upstream_ids;
        s
    };
    messages.push(PromptMessage::new_text(PromptMessageRole::User, section3));

    let isolation_note = if is_isolated {
        "\n> **Note:** No causal relationships found — impact is isolated.\n"
    } else {
        ""
    };

    let section4 = format!(
        "## Assessment Template\n\
         {isolation_note}\n\
         Please complete the following impact assessment:\n\n\
         ### 1. Scope of Change\n\
         Describe what is being changed and why.\n\n\
         ### 2. Affected Specs\n\
         List all specs affected by this change, both direct and transitive.\n\n\
         ### 3. Risk Level\n\
         Rate the risk: **Low** / **Medium** / **High** / **Critical**\n\
         Justify your rating.\n\n\
         ### 4. Recommended Actions\n\
         List specific actions to mitigate risk and ensure consistency.",
        isolation_note = isolation_note,
    );
    messages.push(PromptMessage::new_text(PromptMessageRole::User, section4));

    Ok(GetPromptResult {
        description: Some(
            "Structured impact analysis for a spec before proposing changes".to_owned(),
        ),
        messages,
    })
}

fn resolve_spec_review(
    handler: &ToolHandler,
    params: &GetPromptRequestParams,
) -> Result<GetPromptResult, SpecDbError> {
    let spec_id = extract_spec_id(params)?;
    let spec = lookup_spec(handler, &spec_id)?;

    let store = Arc::new(FjallStore::open(&handler.fjall_dir)?);
    let graph = CausalEngine::from_store(store)?;

    let outbound_edges = graph.edges_from(&spec_id)?;
    let inbound_edges = graph.edges_to(&spec_id)?;

    let mut broken_deps: Vec<String> = Vec::new();
    for dep_id in &spec.depends_on {
        if graph.get_node(dep_id)?.is_none() {
            broken_deps.push(dep_id.to_string());
        }
    }

    let mut messages = Vec::new();

    let section1 = format!(
        "# Spec Review: {id}\n\n\
         ## Spec Content\n\
         - **ID**: {id}\n\
         - **Title**: {title}\n\
         - **Version**: {version}\n\
         - **Tags**: {tags}\n\
         - **Created**: {created}\n\
         - **Owner**: {owner}\n\
         - **Depends On**: {depends_on}\n\n\
         ## Body\n\n{body}",
        id = spec.id,
        title = spec.title,
        version = spec.version,
        tags = if spec.tags.is_empty() { "none".to_owned() } else { spec.tags.join(", ") },
        created = spec.created,
        owner = spec.owner.as_deref().unwrap_or("not specified"),
        depends_on = if spec.depends_on.is_empty() {
            "none".to_owned()
        } else {
            spec.depends_on.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ")
        },
        body = spec.body,
    );
    messages.push(PromptMessage::new_text(PromptMessageRole::User, section1));

    let mut section2 = "## Causal Graph Context\n\n".to_owned();
    if outbound_edges.is_empty() && inbound_edges.is_empty() {
        section2.push_str("No causal edges found for this spec.\n");
    } else {
        if !outbound_edges.is_empty() {
            section2.push_str(
                "### Outbound Edges (this spec → others)\n\n\
                               | Target | Type | Trust | Origin |\n\
                               |--------|------|-------|--------|\n",
            );
            for edge in &outbound_edges {
                section2.push_str(&format!(
                    "| {} | {} | {:.2} | {} |\n",
                    edge.target,
                    edge.edge_type,
                    edge.trust.value(),
                    edge.origin,
                ));
            }
            section2.push('\n');
        }
        if !inbound_edges.is_empty() {
            section2.push_str(
                "### Inbound Edges (others → this spec)\n\n\
                               | Source | Type | Trust | Origin |\n\
                               |--------|------|-------|--------|\n",
            );
            for edge in &inbound_edges {
                section2.push_str(&format!(
                    "| {} | {} | {:.2} | {} |\n",
                    edge.source,
                    edge.edge_type,
                    edge.trust.value(),
                    edge.origin,
                ));
            }
            section2.push('\n');
        }
    }
    messages.push(PromptMessage::new_text(PromptMessageRole::User, section2));

    let has_owner = spec.owner.is_some();

    let broken_dep_finding = if broken_deps.is_empty() {
        "  - [x] All `depends_on` references resolve to existing specs".to_owned()
    } else {
        format!("  - [ ] **BROKEN DEPENDENCY REFERENCES**: {deps}", deps = broken_deps.join(", "))
    };

    let section3 = format!(
        "## Review Checklist\n\n\
         Please evaluate each dimension and check off items that pass:\n\n\
         ### Completeness\n\
         - [x] `id` field present\n\
         - [x] `title` field present\n\
         - [x] `version` field present\n\
         - {tags_check} `tags` field present and non-empty\n\
         - [x] `created` field present\n\
         - {owner_check} `owner` field present (recommended)\n\
         - [x] `depends_on` field present\n\n\
         ### Clarity\n\
         - [ ] Title clearly describes the spec's purpose\n\
         - [ ] Body content is sufficient and well-structured\n\
         - [ ] No ambiguous or contradictory statements\n\n\
         ### Dependency Accuracy\n\
         {broken_dep_finding}\n\
         - [ ] All causal relationships are appropriate and correctly typed\n\
         - [ ] No missing dependencies that should be declared\n\n\
         ### Consistency\n\
         - [ ] Tags are consistent with related specs\n\
         - [ ] Version number is appropriate\n\
         - [ ] Created date is valid\n\
         - [ ] Spec ID follows naming conventions",
        tags_check = if spec.tags.is_empty() { "[ ]" } else { "[x]" },
        owner_check = if has_owner { "[x]" } else { "[ ]" },
        broken_dep_finding = broken_dep_finding,
    );
    messages.push(PromptMessage::new_text(PromptMessageRole::User, section3));

    Ok(GetPromptResult {
        description: Some("Structured spec review with quality checklist".to_owned()),
        messages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_definitions_contain_both_prompts() {
        let prompts = prompt_definitions();
        assert_eq!(prompts.len(), 2);

        let impact = &prompts[0];
        assert_eq!(impact.name, "impact_analysis");
        assert!(impact.description.as_ref().unwrap().contains("impact assessment"));
        let args = impact.arguments.as_ref().unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name, "spec_id");
        assert_eq!(args[0].required, Some(true));

        let review = &prompts[1];
        assert_eq!(review.name, "spec_review");
        assert!(review.description.as_ref().unwrap().contains("quality checklist"));
        let args = review.arguments.as_ref().unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name, "spec_id");
    }

    #[test]
    fn resolve_unknown_prompt_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let handler = ToolHandler {
            repo_path: dir.path().to_path_buf(),
            specs_root: "specs".to_owned(),
            tantivy_dir: dir.path().join("tantivy"),
            fjall_dir: dir.path().join("fjall"),
            ai_default_trust: 0.5,
        };
        let params =
            GetPromptRequestParams { meta: None, name: "nonexistent".to_owned(), arguments: None };
        let result = resolve_prompt(&handler, &params);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("not_found"));
    }

    #[test]
    fn resolve_impact_analysis_missing_spec_id_arg() {
        let dir = tempfile::tempdir().unwrap();
        let handler = ToolHandler {
            repo_path: dir.path().to_path_buf(),
            specs_root: "specs".to_owned(),
            tantivy_dir: dir.path().join("tantivy"),
            fjall_dir: dir.path().join("fjall"),
            ai_default_trust: 0.5,
        };
        let params = GetPromptRequestParams {
            meta: None,
            name: "impact_analysis".to_owned(),
            arguments: None,
        };
        let result = resolve_prompt(&handler, &params);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("Missing required argument"));
    }

    #[test]
    fn resolve_impact_analysis_spec_not_found() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("tantivy")).unwrap();
        std::fs::create_dir_all(dir.path().join("fjall")).unwrap();

        let handler = ToolHandler {
            repo_path: dir.path().to_path_buf(),
            specs_root: "specs".to_owned(),
            tantivy_dir: dir.path().join("tantivy"),
            fjall_dir: dir.path().join("fjall"),
            ai_default_trust: 0.5,
        };

        let mut args = serde_json::Map::new();
        args.insert("spec_id".to_owned(), json!("spec::missing::thing"));
        let params = GetPromptRequestParams {
            meta: None,
            name: "impact_analysis".to_owned(),
            arguments: Some(args),
        };
        let result = resolve_prompt(&handler, &params);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("not_found"));
        assert!(err_msg.contains("Spec not found"));
    }

    #[test]
    fn resolve_impact_analysis_isolated_node() {
        use crate::tools::AddSpecInput;

        let dir = tempfile::tempdir().unwrap();
        let tantivy_dir = dir.path().join("tantivy");
        let fjall_dir = dir.path().join("fjall");
        std::fs::create_dir_all(&tantivy_dir).unwrap();
        std::fs::create_dir_all(&fjall_dir).unwrap();

        let handler = ToolHandler {
            repo_path: dir.path().to_path_buf(),
            specs_root: "specs".to_owned(),
            tantivy_dir: tantivy_dir.clone(),
            fjall_dir: fjall_dir.clone(),
            ai_default_trust: 0.5,
        };

        handler
            .add_spec(AddSpecInput {
                markdown: r#"---
id: "spec::test::isolated"
title: "Isolated Spec"
version: 1
tags: ["test"]
depends_on: []
created: "2026-02-23"
---
# Isolated Spec
This spec has no causal edges.
"#
                .to_owned(),
            })
            .unwrap();

        let mut args = serde_json::Map::new();
        args.insert("spec_id".to_owned(), json!("spec::test::isolated"));
        let params = GetPromptRequestParams {
            meta: None,
            name: "impact_analysis".to_owned(),
            arguments: Some(args),
        };
        let result = resolve_prompt(&handler, &params).unwrap();

        assert_eq!(result.messages.len(), 4);

        let msg1 = &result.messages[0];
        assert!(matches!(msg1.role, PromptMessageRole::User));
        if let rmcp::model::PromptMessageContent::Text { text } = &msg1.content {
            assert!(text.contains("spec::test::isolated"));
            assert!(text.contains("Isolated Spec"));
        } else {
            panic!("expected text content");
        }

        let msg4 = &result.messages[3];
        if let rmcp::model::PromptMessageContent::Text { text } = &msg4.content {
            assert!(
                text.contains("No causal relationships found"),
                "Expected isolation note, got: {text}"
            );
        } else {
            panic!("expected text content");
        }
    }

    #[test]
    fn resolve_impact_analysis_with_edges() {
        use crate::tools::{AddCausalLinkInput, AddSpecInput};

        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path().join("repo");
        let tantivy_dir = dir.path().join("tantivy");
        let fjall_dir = dir.path().join("fjall");
        std::fs::create_dir_all(&repo_path).unwrap();
        std::fs::create_dir_all(&tantivy_dir).unwrap();
        std::fs::create_dir_all(&fjall_dir).unwrap();

        let handler = ToolHandler {
            repo_path,
            specs_root: "specs".to_owned(),
            tantivy_dir: tantivy_dir.clone(),
            fjall_dir: fjall_dir.clone(),
            ai_default_trust: 0.7,
        };

        handler
            .add_spec(AddSpecInput {
                markdown: r#"---
id: "spec::test::a"
title: "Spec A"
version: 1
tags: ["auth"]
depends_on: []
created: "2026-02-23"
---
# Spec A
Auth module.
"#
                .to_owned(),
            })
            .unwrap();

        handler
            .add_spec(AddSpecInput {
                markdown: r#"---
id: "spec::test::b"
title: "Spec B"
version: 1
tags: ["api"]
depends_on: []
created: "2026-02-23"
---
# Spec B
API module.
"#
                .to_owned(),
            })
            .unwrap();

        handler
            .add_spec(AddSpecInput {
                markdown: r#"---
id: "spec::test::c"
title: "Spec C"
version: 1
tags: ["core"]
depends_on: []
created: "2026-02-23"
---
# Spec C
Core module.
"#
                .to_owned(),
            })
            .unwrap();

        handler
            .add_causal_link(AddCausalLinkInput {
                source: "spec::test::a".to_owned(),
                target: "spec::test::b".to_owned(),
                edge_type: Some("depends_on".to_owned()),
            })
            .unwrap();

        handler
            .add_causal_link(AddCausalLinkInput {
                source: "spec::test::c".to_owned(),
                target: "spec::test::a".to_owned(),
                edge_type: Some("depends_on".to_owned()),
            })
            .unwrap();

        let mut args = serde_json::Map::new();
        args.insert("spec_id".to_owned(), json!("spec::test::a"));
        let params = GetPromptRequestParams {
            meta: None,
            name: "impact_analysis".to_owned(),
            arguments: Some(args),
        };
        let result = resolve_prompt(&handler, &params).unwrap();

        assert_eq!(result.messages.len(), 4);

        if let rmcp::model::PromptMessageContent::Text { text } = &result.messages[0].content {
            assert!(text.contains("spec::test::a"));
            assert!(text.contains("Spec A"));
        } else {
            panic!("expected text content");
        }

        if let rmcp::model::PromptMessageContent::Text { text } = &result.messages[1].content {
            assert!(text.contains("Downstream Impact"));
            assert!(text.contains("spec::test::b"));
        } else {
            panic!("expected text content");
        }

        if let rmcp::model::PromptMessageContent::Text { text } = &result.messages[2].content {
            assert!(text.contains("Upstream Dependencies"));
            assert!(text.contains("spec::test::c"));
        } else {
            panic!("expected text content");
        }

        if let rmcp::model::PromptMessageContent::Text { text } = &result.messages[3].content {
            assert!(text.contains("Assessment Template"));
            assert!(!text.contains("impact is isolated"));
        } else {
            panic!("expected text content");
        }
    }

    #[test]
    fn resolve_spec_review_not_found() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("tantivy")).unwrap();
        std::fs::create_dir_all(dir.path().join("fjall")).unwrap();

        let handler = ToolHandler {
            repo_path: dir.path().to_path_buf(),
            specs_root: "specs".to_owned(),
            tantivy_dir: dir.path().join("tantivy"),
            fjall_dir: dir.path().join("fjall"),
            ai_default_trust: 0.5,
        };

        let mut args = serde_json::Map::new();
        args.insert("spec_id".to_owned(), json!("spec::missing::thing"));
        let params = GetPromptRequestParams {
            meta: None,
            name: "spec_review".to_owned(),
            arguments: Some(args),
        };
        let result = resolve_prompt(&handler, &params);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("not_found"));
        assert!(err_msg.contains("Spec not found"));
    }

    #[test]
    fn resolve_spec_review_healthy_spec() {
        use crate::tools::AddSpecInput;

        let dir = tempfile::tempdir().unwrap();
        let tantivy_dir = dir.path().join("tantivy");
        let fjall_dir = dir.path().join("fjall");
        std::fs::create_dir_all(&tantivy_dir).unwrap();
        std::fs::create_dir_all(&fjall_dir).unwrap();

        let handler = ToolHandler {
            repo_path: dir.path().to_path_buf(),
            specs_root: "specs".to_owned(),
            tantivy_dir,
            fjall_dir,
            ai_default_trust: 0.5,
        };

        handler
            .add_spec(AddSpecInput {
                markdown: r#"---
id: "spec::test::healthy"
title: "Healthy Spec"
version: 1
tags: ["test", "review"]
depends_on: []
created: "2026-02-23"
---
# Healthy Spec
Well-formed spec for review testing.
"#
                .to_owned(),
            })
            .unwrap();

        let mut args = serde_json::Map::new();
        args.insert("spec_id".to_owned(), json!("spec::test::healthy"));
        let params = GetPromptRequestParams {
            meta: None,
            name: "spec_review".to_owned(),
            arguments: Some(args),
        };
        let result = resolve_prompt(&handler, &params).unwrap();

        assert_eq!(result.messages.len(), 3);
        assert!(result.description.as_ref().unwrap().contains("quality checklist"));

        if let rmcp::model::PromptMessageContent::Text { text } = &result.messages[0].content {
            assert!(text.contains("spec::test::healthy"));
            assert!(text.contains("Healthy Spec"));
            assert!(text.contains("Spec Review:"));
        } else {
            panic!("expected text content");
        }

        if let rmcp::model::PromptMessageContent::Text { text } = &result.messages[1].content {
            assert!(text.contains("Causal Graph Context"));
        } else {
            panic!("expected text content");
        }

        if let rmcp::model::PromptMessageContent::Text { text } = &result.messages[2].content {
            assert!(text.contains("Review Checklist"));
            assert!(text.contains("Completeness"));
            assert!(text.contains("Clarity"));
            assert!(text.contains("Dependency Accuracy"));
            assert!(text.contains("Consistency"));
            assert!(text.contains("All `depends_on` references resolve"));
        } else {
            panic!("expected text content");
        }
    }

    #[test]
    fn resolve_spec_review_broken_dependencies() {
        use crate::tools::AddSpecInput;

        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path().join("repo");
        let tantivy_dir = dir.path().join("tantivy");
        let fjall_dir = dir.path().join("fjall");
        std::fs::create_dir_all(&repo_path).unwrap();
        std::fs::create_dir_all(&tantivy_dir).unwrap();
        std::fs::create_dir_all(&fjall_dir).unwrap();

        let handler = ToolHandler {
            repo_path,
            specs_root: "specs".to_owned(),
            tantivy_dir,
            fjall_dir,
            ai_default_trust: 0.5,
        };

        handler
            .add_spec(AddSpecInput {
                markdown: r#"---
id: "spec::test::with-broken-deps"
title: "Spec With Broken Deps"
version: 1
tags: ["test"]
depends_on: ["spec::test::nonexistent-dep"]
created: "2026-02-23"
---
# Spec With Broken Deps
This spec depends on something that doesn't exist.
"#
                .to_owned(),
            })
            .unwrap();

        let mut args = serde_json::Map::new();
        args.insert("spec_id".to_owned(), json!("spec::test::with-broken-deps"));
        let params = GetPromptRequestParams {
            meta: None,
            name: "spec_review".to_owned(),
            arguments: Some(args),
        };
        let result = resolve_prompt(&handler, &params).unwrap();

        assert_eq!(result.messages.len(), 3);

        if let rmcp::model::PromptMessageContent::Text { text } = &result.messages[2].content {
            assert!(
                text.contains("BROKEN DEPENDENCY REFERENCES"),
                "Expected broken dep flag, got: {text}"
            );
            assert!(text.contains("spec::test::nonexistent-dep"));
        } else {
            panic!("expected text content");
        }
    }
}
