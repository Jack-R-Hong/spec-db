use spec_db_core::{SpecDbError, SpecDoc, SpecId};
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, QueryParser, TermQuery};
use tantivy::schema::{IndexRecordOption, TantivyDocument, Value};
use tantivy::snippet::SnippetGenerator;
use tantivy::{Index, Searcher, Term};

use crate::schema::SearchSchemaFields;

const TITLE_BOOST: f32 = 2.0;
const MAX_SNIPPET_CHARS: usize = 180;

pub struct SearchHit {
    pub id: SpecId,
    pub title: String,
    pub score: f32,
    pub snippet: String,
    pub tags: Vec<String>,
}

pub fn build_text_query(
    index: &Index,
    fields: &SearchSchemaFields,
    q: &str,
) -> Result<Box<dyn tantivy::query::Query>, SpecDbError> {
    let mut query_parser = QueryParser::for_index(index, vec![fields.title, fields.body]);
    query_parser.set_field_boost(fields.title, TITLE_BOOST);
    query_parser
        .parse_query(q)
        .map_err(|e| SpecDbError::SearchError(format!("failed to parse query '{q}': {e}")))
}

pub fn build_tag_filter(fields: &SearchSchemaFields, tag: &str) -> Box<dyn tantivy::query::Query> {
    let tag_term = Term::from_field_text(fields.tags, tag);
    Box::new(TermQuery::new(tag_term, IndexRecordOption::Basic))
}

pub fn execute_search(
    searcher: &Searcher,
    fields: &SearchSchemaFields,
    query: &dyn tantivy::query::Query,
    limit: usize,
) -> Result<Vec<SearchHit>, SpecDbError> {
    let top_docs = searcher
        .search(query, &TopDocs::with_limit(limit))
        .map_err(|e| SpecDbError::SearchError(format!("search execution failed: {e}")))?;

    if top_docs.is_empty() {
        return Ok(Vec::new());
    }

    let mut snippet_generator =
        SnippetGenerator::create(searcher, query, fields.body).map_err(|e| {
            SpecDbError::SearchError(format!("failed to create snippet generator: {e}"))
        })?;
    snippet_generator.set_max_num_chars(MAX_SNIPPET_CHARS);

    let mut hits = Vec::with_capacity(top_docs.len());
    for (score, address) in top_docs {
        let document: TantivyDocument = searcher
            .doc(address)
            .map_err(|e| SpecDbError::SearchError(format!("failed to fetch search hit: {e}")))?;

        let id = parse_spec_id(&document, fields)?;
        let title = read_required_string(&document, fields.title, "title")?;
        let tags = document
            .get_all(fields.tags)
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect();
        let snippet = snippet_generator.snippet_from_doc(&document).fragment().to_owned();

        hits.push(SearchHit { id, title, score, snippet, tags });
    }

    Ok(hits)
}

pub fn get_spec_by_id(
    searcher: &Searcher,
    fields: &SearchSchemaFields,
    id: &SpecId,
) -> Result<Option<SpecDoc>, SpecDbError> {
    let id_term = Term::from_field_text(fields.id, id.as_ref());
    let query = TermQuery::new(id_term, IndexRecordOption::Basic);
    let results = searcher
        .search(&query, &TopDocs::with_limit(1))
        .map_err(|e| SpecDbError::SearchError(format!("id lookup failed for '{id}': {e}")))?;

    let Some((_, address)) = results.into_iter().next() else {
        return Ok(None);
    };

    let document: TantivyDocument = searcher
        .doc(address)
        .map_err(|e| SpecDbError::SearchError(format!("failed to fetch spec '{id}': {e}")))?;

    let doc_id = parse_spec_id(&document, fields)?;
    let title = read_required_string(&document, fields.title, "title")?;
    let tags = document
        .get_all(fields.tags)
        .filter_map(|value| value.as_str().map(ToOwned::to_owned))
        .collect();

    let meta = document.get_first(fields.meta).ok_or_else(|| {
        SpecDbError::SearchError(format!("missing meta field for spec '{doc_id}'"))
    })?;
    let (version, depends_on, owner, created) = parse_meta(meta, &doc_id)?;

    Ok(Some(SpecDoc {
        id: doc_id,
        title,
        version,
        tags,
        depends_on,
        owner,
        created,
        body: String::new(),
    }))
}

pub fn combine_text_with_tags(
    text_query: Box<dyn tantivy::query::Query>,
    fields: &SearchSchemaFields,
    tags: &[String],
) -> BooleanQuery {
    let mut clauses: Vec<(Occur, Box<dyn tantivy::query::Query>)> =
        Vec::with_capacity(tags.len() + 1);
    clauses.push((Occur::Must, text_query));
    for tag in tags {
        clauses.push((Occur::Must, build_tag_filter(fields, tag)));
    }
    BooleanQuery::new(clauses)
}

fn read_required_string(
    document: &TantivyDocument,
    field: tantivy::schema::Field,
    field_name: &str,
) -> Result<String, SpecDbError> {
    document
        .get_first(field)
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| SpecDbError::SearchError(format!("missing or invalid '{field_name}' field")))
}

fn parse_spec_id(
    document: &TantivyDocument,
    fields: &SearchSchemaFields,
) -> Result<SpecId, SpecDbError> {
    let raw_id = read_required_string(document, fields.id, "id")?;
    SpecId::try_new(raw_id.clone()).map_err(|e| {
        SpecDbError::SearchError(format!("invalid spec id stored in search index '{raw_id}': {e}"))
    })
}

fn parse_meta<'a>(
    meta: impl Value<'a>,
    doc_id: &SpecId,
) -> Result<(u32, Vec<SpecId>, Option<String>, String), SpecDbError> {
    let mut version: Option<u32> = None;
    let mut depends_on: Vec<SpecId> = Vec::new();
    let mut owner: Option<String> = None;
    let mut created: Option<String> = None;

    let Some(object_entries) = meta.as_object() else {
        return Err(SpecDbError::SearchError(format!(
            "meta field is not an object for spec '{doc_id}'"
        )));
    };

    for (key, value) in object_entries {
        match key {
            "version" => {
                let raw_version = value.as_u64().ok_or_else(|| {
                    SpecDbError::SearchError(format!("invalid meta.version for spec '{doc_id}'"))
                })?;
                let converted_version = u32::try_from(raw_version).map_err(|_| {
                    SpecDbError::SearchError(format!(
                        "meta.version out of range for spec '{doc_id}'"
                    ))
                })?;
                version = Some(converted_version);
            }
            "depends_on" => {
                let Some(dep_values) = value.as_array() else {
                    return Err(SpecDbError::SearchError(format!(
                        "meta.depends_on must be an array for spec '{doc_id}'"
                    )));
                };

                for dep in dep_values {
                    let dep_text = dep.as_str().ok_or_else(|| {
                        SpecDbError::SearchError(format!(
                            "meta.depends_on has non-string entry for spec '{doc_id}'"
                        ))
                    })?;
                    let dep_id = SpecId::try_new(dep_text.to_owned()).map_err(|e| {
                        SpecDbError::SearchError(format!(
                            "meta.depends_on has invalid SpecId for spec '{doc_id}': {e}"
                        ))
                    })?;
                    depends_on.push(dep_id);
                }
            }
            "owner" => {
                owner = value.as_str().map(ToOwned::to_owned);
            }
            "created" => {
                let created_value = value.as_str().ok_or_else(|| {
                    SpecDbError::SearchError(format!("invalid meta.created for spec '{doc_id}'"))
                })?;
                created = Some(created_value.to_owned());
            }
            _ => {}
        }
    }

    let version = version.ok_or_else(|| {
        SpecDbError::SearchError(format!("missing meta.version for spec '{doc_id}'"))
    })?;
    let created = created.ok_or_else(|| {
        SpecDbError::SearchError(format!("missing meta.created for spec '{doc_id}'"))
    })?;

    Ok((version, depends_on, owner, created))
}
