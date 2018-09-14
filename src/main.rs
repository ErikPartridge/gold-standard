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
extern crate strsim;
extern crate percent_encoding;


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
use rocket::fairing::AdHoc;
use rocket::http::Header;
use std::ops::Deref;
mod mail;
mod schema;

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
                routes::index,
                routes::privacy
            ],
        ).mount("/cdn", routes![routes::files])
        .attach(Template::fairing())
        .attach(AdHoc::on_response(|_, res| {
            let xframe = Header::new("X-Frame-Options".to_string(), "DENY".to_string());
            let cors = Header::new("Cross-Origin-Resource-Policy".to_string(), "no-cors".to_string());
            let content_lang = Header::new("Content-Language".to_string(), "en-US".to_string());
            let referrer = Header::new("Referrer-Policy".to_string(), "no-referrer".to_string());
            res.set_header(xframe);
            res.set_header(cors);
            res.set_header(referrer);
            res.set_header(content_lang);
        }))
        .catch(catchers![routes::not_found])
        .manage(init_pool())
        .launch();
}
