use spec_db_core::SearchEngine;
use spec_db_core::{SpecDoc, SpecId};
use spec_db_search::schema::{FIELD_BODY, FIELD_ID, FIELD_META, FIELD_TAGS, FIELD_TITLE};
use spec_db_search::{SearchIndex, query, schema};
use std::time::{Duration, Instant};
use tantivy::collector::TopDocs;
use tantivy::query::{QueryParser, TermQuery};
use tantivy::schema::{FieldType, IndexRecordOption, TantivyDocument};
use tantivy::{DocAddress, Term};

fn doc(id: &str, title: &str, body: &str, tags: &[&str]) -> SpecDoc {
    SpecDoc {
        id: SpecId::try_new(id).unwrap(),
        title: title.to_owned(),
        version: 1,
        tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
        depends_on: Vec::new(),
        owner: Some("search-team".to_owned()),
        created: "2026-01-01T00:00:00Z".to_owned(),
        body: body.to_owned(),
    }
}

fn find_by_id(index: &SearchIndex, id: &SpecId) -> Vec<(f32, DocAddress)> {
    let searcher = index.reader().searcher();
    let term = Term::from_field_text(index.fields().id, id.as_ref());
    let query = TermQuery::new(term, IndexRecordOption::Basic);
    searcher.search(&query, &TopDocs::with_limit(10)).unwrap()
}

#[test]
fn schema_has_expected_fields() {
    let fields = schema::build_schema();
    let built_schema = &fields.schema;

    let id_field = built_schema.get_field(FIELD_ID).unwrap();
    let title_field = built_schema.get_field(FIELD_TITLE).unwrap();
    let body_field = built_schema.get_field(FIELD_BODY).unwrap();
    let tags_field = built_schema.get_field(FIELD_TAGS).unwrap();
    let meta_field = built_schema.get_field(FIELD_META).unwrap();

    match built_schema.get_field_entry(id_field).field_type() {
        FieldType::Str(options) => {
            assert!(options.is_stored());
            assert!(options.get_indexing_options().is_some());
        }
        other => panic!("expected id as string field, got {other:?}"),
    }

    match built_schema.get_field_entry(title_field).field_type() {
        FieldType::Str(options) => {
            assert!(options.is_stored());
            assert!(options.get_indexing_options().is_some());
        }
        other => panic!("expected title as text field, got {other:?}"),
    }

    match built_schema.get_field_entry(body_field).field_type() {
        FieldType::Str(options) => {
            assert!(!options.is_stored());
            assert!(options.get_indexing_options().is_some());
        }
        other => panic!("expected body as text field, got {other:?}"),
    }

    match built_schema.get_field_entry(tags_field).field_type() {
        FieldType::Str(options) => {
            assert!(options.is_stored());
            assert!(options.get_indexing_options().is_some());
        }
        other => panic!("expected tags as string field, got {other:?}"),
    }

    match built_schema.get_field_entry(meta_field).field_type() {
        FieldType::JsonObject(options) => {
            assert!(options.is_stored());
        }
        other => panic!("expected meta as JSON field, got {other:?}"),
    }
}

#[test]
fn add_and_commit_roundtrip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(temp_dir.path()).unwrap();
    let spec = doc(
        "spec::search::roundtrip",
        "Roundtrip Title",
        "alpha beta gamma",
        &["search", "roundtrip"],
    );

    index.add_doc(&spec).unwrap();
    index.commit().unwrap();

    let searcher = index.reader().searcher();
    let query =
        QueryParser::for_index(index.index(), vec![index.fields().title, index.fields().body])
            .parse_query("alpha")
            .unwrap();
    let docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

    assert_eq!(docs.len(), 1);
    let retrieved: TantivyDocument = searcher.doc(docs[0].1).unwrap();
    let id_values: Vec<_> = retrieved.get_all(index.fields().id).collect();
    assert_eq!(id_values.len(), 1);
}

#[test]
fn remove_and_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(temp_dir.path()).unwrap();
    let spec = doc("spec::search::remove-me", "Remove Me", "this should be removed", &["search"]);

    index.add_doc(&spec).unwrap();
    index.commit().unwrap();
    assert_eq!(find_by_id(&index, &spec.id).len(), 1);

    index.remove_doc(&spec.id).unwrap();
    index.commit().unwrap();
    assert!(find_by_id(&index, &spec.id).is_empty());
}

#[test]
fn multiple_updates_single_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(temp_dir.path()).unwrap();

    let docs = vec![
        doc("spec::search::batch-1", "Batch One", "first batched document", &["batch"]),
        doc("spec::search::batch-2", "Batch Two", "second batched document", &["batch"]),
        doc("spec::search::batch-3", "Batch Three", "third batched document", &["batch"]),
    ];

    for spec in &docs {
        index.add_doc(spec).unwrap();
    }
    index.commit().unwrap();

    for spec in &docs {
        assert_eq!(find_by_id(&index, &spec.id).len(), 1);
    }
}

#[test]
fn title_match_ranks_higher() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(temp_dir.path()).unwrap();

    let title_match = doc(
        "spec::search::title-match",
        "Rate limiting policy",
        "general API controls and quotas",
        &["api"],
    );
    let body_match = doc(
        "spec::search::body-match",
        "Traffic policy",
        "rate limiting appears only in this body text",
        &["api"],
    );

    index.add_doc(&title_match).unwrap();
    index.add_doc(&body_match).unwrap();
    index.commit().unwrap();

    let text_query =
        query::build_text_query(index.index(), index.fields(), "rate limiting").unwrap();
    let hits = query::execute_search(&index.searcher(), index.fields(), &*text_query, 10).unwrap();

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].id, title_match.id);
    assert_eq!(hits[1].id, body_match.id);
}

#[test]
fn tag_filter_exact_match() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(temp_dir.path()).unwrap();

    let auth_spec =
        doc("spec::search::auth-filter", "JWT validation", "token validation flow", &["auth"]);
    let api_spec =
        doc("spec::search::api-filter", "API pagination", "token pagination policy", &["api"]);

    index.add_doc(&auth_spec).unwrap();
    index.add_doc(&api_spec).unwrap();
    index.commit().unwrap();

    let ids = index.search_with_tags("token", &[String::from("auth")], 10).unwrap();

    assert_eq!(ids, vec![auth_spec.id]);
}

#[test]
fn search_returns_empty_for_no_match() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(temp_dir.path()).unwrap();
    let spec = doc(
        "spec::search::empty-match",
        "Caching strategy",
        "cache invalidation details",
        &["perf"],
    );
    index.add_doc(&spec).unwrap();
    index.commit().unwrap();

    let ids = index.search("term-that-does-not-exist", 10).unwrap();
    assert!(ids.is_empty());
}

#[test]
fn search_perf_100_specs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(temp_dir.path()).unwrap();

    for i in 0..150 {
        let spec = doc(
            &format!("spec::search::perf-{i}"),
            &format!("Performance Spec {i}"),
            "latency throughput saturation capacity planning",
            if i % 2 == 0 { &["perf"] } else { &["ops"] },
        );
        index.add_doc(&spec).unwrap();
    }
    index.commit().unwrap();

    let _ = index.search("latency", 20).unwrap();

    let start = Instant::now();
    let ids = index.search("latency", 20).unwrap();
    let elapsed = start.elapsed();

    assert!(!ids.is_empty());
    assert!(elapsed < Duration::from_millis(10), "search took {elapsed:?}");
}

#[test]
fn search_results_include_score() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(temp_dir.path()).unwrap();

    let spec =
        doc("spec::search::score", "Rate limiter", "rate limiting controls in detail", &["search"]);
    index.add_doc(&spec).unwrap();
    index.commit().unwrap();

    let text_query =
        query::build_text_query(index.index(), index.fields(), "rate limiting").unwrap();
    let hits = query::execute_search(&index.searcher(), index.fields(), &*text_query, 10).unwrap();

    assert!(!hits.is_empty());
    assert!(hits[0].score > 0.0);
}
