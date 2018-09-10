#![allow(proc_macro_derive_resolution_fallback)]
extern crate chrono;
extern crate serde;
use chrono::{NaiveDate, Utc};
use schema::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
extern crate diesel;
use diesel::expression::dsl::now;
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
}

#[derive(Insertable)]
#[table_name = "fields"]
pub struct NewField {
    pub name: String,
    pub synonyms: Vec<String>,
}

impl Field {
    /*fn save(&self, conn: DbConn) -> Option<Field> {
        let date = Utc::now().naive_utc().date();
        let updated = Field {
            updated_at: date,
            ..self.clone()
        };
        let res = diesel::update(fields::table)
            .set(&updated)
            .get_result::<Field>(&*conn);
        match res {
            Ok(x) => return Some(x),
            _ => return None,
        }
    }*/

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

impl NewField {
    fn save(&self, conn: DbConn) -> Option<Field> {
        let res = diesel::insert_into(fields::dsl::fields)
            .values(self)
            .get_result::<Field>(&*conn);
        match res {
            Ok(field) => Some(field),
            Err(_) => None,
        }
    }
}
