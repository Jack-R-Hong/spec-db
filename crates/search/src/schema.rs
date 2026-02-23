use tantivy::schema::{Field, STORED, STRING, Schema, TEXT};

pub const FIELD_ID: &str = "id";
pub const FIELD_TITLE: &str = "title";
pub const FIELD_BODY: &str = "body";
pub const FIELD_TAGS: &str = "tags";
pub const FIELD_META: &str = "meta";

#[derive(Clone)]
pub struct SearchSchemaFields {
    pub schema: Schema,
    pub id: Field,
    pub title: Field,
    pub body: Field,
    pub tags: Field,
    pub meta: Field,
}

pub fn build_schema() -> SearchSchemaFields {
    let mut builder = Schema::builder();
    let id = builder.add_text_field(FIELD_ID, STRING | STORED);
    let title = builder.add_text_field(FIELD_TITLE, TEXT | STORED);
    let body = builder.add_text_field(FIELD_BODY, TEXT);
    let tags = builder.add_text_field(FIELD_TAGS, STRING | STORED);
    let meta = builder.add_json_field(FIELD_META, STORED);
    let schema = builder.build();
    SearchSchemaFields { schema, id, title, body, tags, meta }
}
