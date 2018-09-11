#![allow(proc_macro_derive_resolution_fallback)]
extern crate chrono;
extern crate serde;
use chrono::{NaiveDate};
use schema::*;
extern crate diesel;
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use routes::DbConn;

#[derive(
    Queryable, Debug, Clone, AsChangeset, Associations, Serialize, Deserialize, Identifiable,
)]
pub struct Field {
    pub id: i32,
    pub name: String,
    pub synonyms: Vec<String>,
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
    pub slug: Option<String>,
}

impl Field {

    pub fn all(conn: DbConn) -> Option<Vec<Field>> {
        let target =
            fields::dsl::fields.filter(fields::dsl::id.gt(0));
        let results = target.load::<Field>(&*conn);
        match results {
            Ok(vector) => return Some(vector),
            _ => return None,
        }
    }
}