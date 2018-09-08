extern crate chrono;
extern crate serde;
use chrono::{NaiveDate, Utc};
use schema::*;
extern crate diesel;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use diesel::RunQueryDsl;
use routes::DbConn;

#[derive(Queryable, Debug, AsChangeset, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub id: i32,
    pub reference_code: String,
    pub name: String,
    pub email: String,
    pub bio: String,
    pub reference: String,
    pub title: String,
    pub author: String,
    pub category: String,
    pub message: String,
    pub created_at: chrono::NaiveDate,
    pub updated_at: chrono::NaiveDate,
}

#[derive(Insertable)]
#[table_name = "submissions"]
pub struct NewSubmission {
    pub reference_code: String,
    pub name: String,
    pub email: String,
    pub bio: String,
    pub reference: String,
    pub title: String,
    pub author: String,
    pub category: String,
    pub message: String,
    pub created_at: chrono::NaiveDate,
    pub updated_at: chrono::NaiveDate
}


impl Submission {
    fn save(&self, conn: DbConn) -> Option<Submission> {
        let date = Utc::now().naive_utc().date();
        let updated = Submission{updated_at: date, .. self.clone()};
        let res = diesel::update(submissions::table).set(&updated).get_result::<Submission>(&*conn);
        match res {
            Ok(x) => return Some(x),
            _ => return None
        }
    }
}

impl NewSubmission {
    pub fn save(&self, conn : DbConn) -> Option<Submission> {
        let res = diesel::insert_into(submissions::dsl::submissions)
            .values(self)
            .get_result::<Submission>(&*conn);
        match res {
            Ok(submission) => Some(submission),
            Err(y) => {
                println!("{:?}", y);
                return None;
            }
        }
    }
}