#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(decl_macro)]
#![feature(custom_derive)]
extern crate chrono;
extern crate dotenv;
extern crate lettre;
extern crate lettre_email;
extern crate rand;
extern crate rocket;
extern crate serde;
extern crate tera;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde_derive;

extern crate rocket_contrib;
mod model;
// Bring both Template and Context into scope
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use rocket_contrib::Template;
mod routes;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use std::ops::Deref;
mod schema;
mod mail;

// The URL to the database, set via the `DATABASE_URL` environment variable.
static DATABASE_URL: &'static str = env!("DATABASE_URL");

type PgPool = Pool<ConnectionManager<PgConnection>>;
/// Initializes a database pool.
fn init_pool() -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL);
    Pool::new(manager).expect("db pool")
}

pub struct DbConn(pub PooledConnection<ConnectionManager<PgConnection>>);

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let pool = request.guard::<State<PgPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

// For the convenience of using an &DbConn as an &SqliteConnection.
impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                routes::thank_you,
                routes::submission,
                routes::submit,
                routes::about,
                routes::search_submit,
                routes::get_category,
                routes::get_item,
                routes::index
            ],
        ).mount("/cdn", routes![routes::files])
        .attach(Template::fairing())
        .manage(init_pool())
        .launch();
}
