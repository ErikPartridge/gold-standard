#![allow(proc_macro_derive_resolution_fallback)]
extern crate chrono;
extern crate serde;
use chrono::{NaiveDate, Utc};
use schema::*;
extern crate diesel;
use diesel::RunQueryDsl;
use routes::DbConn;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(
    Queryable, Associations, AsChangeset, Clone, Debug, Serialize, Deserialize, Identifiable,
)]
#[belongs_to(model::field::Field)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub author: String,
    pub url: Option<String>,
    pub purchase_name: Option<String>,
    pub medium: String,
    pub description: String,
    pub source: String,
    pub reasoning: String,
    pub blurb: String,
    pub isbn: Option<String>,
    pub year_of_creation: String,
    pub slug: String,
    pub citation: String,
    pub flags: Vec<String>,
    pub field_id: Option<i32>,
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
}

#[derive(Insertable)]
#[table_name = "products"]
pub struct NewProduct {
    pub name: String,
    pub author: String,
    pub url: Option<String>,
    pub purchase_name: Option<String>,
    pub medium: String,
    pub description: String,
    pub source: String,
    pub reasoning: String,
    pub blurb: String,
    pub isbn: Option<String>,
    pub year_of_creation: String,
    pub slug: String,
    pub citation: String,
    pub flags: Vec<String>,
    pub field_id: Option<i32>,
}

impl Product {
    /*fn save(&self, conn: DbConn) -> Option<Product> {
        let date = Utc::now().naive_utc().date();
        let updated = Product {
            updated_at: date,
            ..self.clone()
        };
        let res = diesel::update(products::table)
            .set(&updated)
            .get_result::<Product>(&*conn);
        match res {
            Ok(x) => return Some(x),
            _ => return None,
        }
    }*/
}

impl NewProduct {
    fn save(&self, conn: DbConn) -> Option<Product> {
        let res = diesel::insert_into(products::dsl::products)
            .values(self)
            .get_result::<Product>(&*conn);
        match res {
            Ok(product) => Some(product),
            Err(_) => None,
        }
    }
}
