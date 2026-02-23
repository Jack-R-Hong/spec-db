//! Acceptance tests for Story 2.1: Tantivy Schema Definition & Spec Indexing

use spec_db_core::{SearchEngine, SpecDoc, SpecId};
use spec_db_search::schema::{FIELD_BODY, FIELD_ID, FIELD_META, FIELD_TAGS, FIELD_TITLE};
use spec_db_search::{SearchIndex, schema};

fn _spec_id(value: &str) -> SpecId {
    SpecId::try_new(value).unwrap()
}

fn _doc(id: &str, title: &str, body: &str, tags: &[&str]) -> SpecDoc {
    SpecDoc {
        id: _spec_id(id),
        title: title.to_owned(),
        version: 1,
        tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
        depends_on: Vec::new(),
        owner: Some("acceptance".to_owned()),
        created: "2026-02-23".to_owned(),
        body: body.to_owned(),
    }
}

/// AC1: Schema contains id/title/body/tags/meta fields with required indexing and storage options.
#[test]
fn ac1_schema_contains_expected_fields_and_flags() {
    let fields = schema::build_schema();
    let built_schema = &fields.schema;

    let id_field = built_schema.get_field(FIELD_ID).unwrap();
    let title_field = built_schema.get_field(FIELD_TITLE).unwrap();
    let body_field = built_schema.get_field(FIELD_BODY).unwrap();
    let tags_field = built_schema.get_field(FIELD_TAGS).unwrap();
    let meta_field = built_schema.get_field(FIELD_META).unwrap();

    let id_entry = format!("{:?}", built_schema.get_field_entry(id_field).field_type());
    let title_entry = format!("{:?}", built_schema.get_field_entry(title_field).field_type());
    let body_entry = format!("{:?}", built_schema.get_field_entry(body_field).field_type());
    let tags_entry = format!("{:?}", built_schema.get_field_entry(tags_field).field_type());
    let meta_entry = format!("{:?}", built_schema.get_field_entry(meta_field).field_type());

    assert!(id_entry.contains("stored: true"), "id field config: {id_entry}");
    assert!(id_entry.contains("Basic"), "id field config: {id_entry}");

    assert!(title_entry.contains("stored: true"), "title field config: {title_entry}");
    assert!(title_entry.contains("WithFreqsAndPositions"), "title field config: {title_entry}");

    assert!(body_entry.contains("stored: false"), "body field config: {body_entry}");
    assert!(body_entry.contains("WithFreqsAndPositions"), "body field config: {body_entry}");

    assert!(tags_entry.contains("stored: true"), "tags field config: {tags_entry}");
    assert!(tags_entry.contains("Basic"), "tags field config: {tags_entry}");

    assert!(meta_entry.contains("JsonObject"), "meta field config: {meta_entry}");
    assert!(meta_entry.contains("stored: true"), "meta field config: {meta_entry}");
}

/// AC2: add_doc + commit makes the document retrievable.
#[test]
fn ac2_add_doc_and_commit_makes_document_retrievable() {
    let dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(dir.path()).unwrap();
    let doc = _doc(
        "spec::search::ac2-roundtrip",
        "Roundtrip retrieval",
        "alpha beta gamma",
        &["search", "roundtrip"],
    );

    index.add_doc(&doc).unwrap();
    index.commit().unwrap();

    let ids = index.search("alpha", 10).unwrap();
    assert_eq!(ids, vec![doc.id]);
}

/// AC3: remove_doc + commit removes the document from search.
#[test]
fn ac3_remove_doc_and_commit_removes_document() {
    let dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(dir.path()).unwrap();
    let doc =
        _doc("spec::search::ac3-remove", "Remove me", "document should disappear", &["search"]);

    index.add_doc(&doc).unwrap();
    index.commit().unwrap();
    assert_eq!(index.search("disappear", 10).unwrap(), vec![doc.id.clone()]);

    index.remove_doc(&doc.id).unwrap();
    index.commit().unwrap();
    assert!(index.search("disappear", 10).unwrap().is_empty());
}

/// AC4: commit persists batched changes atomically.
#[test]
fn ac4_commit_persists_batched_changes_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(dir.path()).unwrap();

    let doc_a =
        _doc("spec::search::ac4-batch-a", "Batch A", "batched commit payload alpha", &["batch"]);
    let doc_b =
        _doc("spec::search::ac4-batch-b", "Batch B", "batched commit payload beta", &["batch"]);

    index.add_doc(&doc_a).unwrap();
    index.add_doc(&doc_b).unwrap();

    assert!(index.search("batched", 10).unwrap().is_empty());

    index.commit().unwrap();
    let mut ids = index.search("batched", 10).unwrap();
    ids.sort_by(|left, right| left.as_ref().cmp(right.as_ref()));

    let mut expected = vec![doc_a.id, doc_b.id];
    expected.sort_by(|left, right| left.as_ref().cmp(right.as_ref()));
    assert_eq!(ids, expected);
}
