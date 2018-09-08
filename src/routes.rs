extern crate diesel;
extern crate r2d2;
extern crate rocket;
extern crate rocket_contrib;
extern crate tera;
extern crate nanoid;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use chrono::{NaiveDate, Utc};
use rand::Rng;
use rocket::http::Status;
use rocket::request::{Form,FromRequest, FromForm};
use rocket::request;
use rocket::response::NamedFile;
use rocket::response::Redirect;
use rocket::{Outcome, Request, State};
use rocket_contrib::Template;
use std::iter;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use tera::Context;

use diesel::prelude::*;
use model::field::{Field};
use model::product::{Product};
use model::submission::{NewSubmission};
use schema::*;
use mail::{NewSubmissionEmail, Mailer};

#[derive(FromForm)]
struct UserSubmission {
    pub name: String,
    email: String,
    bio: String,
    title: String,
    reference: String,
    author: String,
    category: String,
    message: String,
}

type PgPool = Pool<ConnectionManager<PgConnection>>;

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

#[get("/")]
fn index() -> rocket_contrib::Template {
    return Template::render("index", Context::new());
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

#[get("/item/<slug>")]
fn get_item(slug: String, conn: DbConn) -> rocket_contrib::Template {
    let conn = &*conn;
    let mut context = Context::new();
    let mut result = products::dsl::products
        .filter(products::dsl::slug.eq(slug))
        .first::<Product>(conn)
        .expect("Issue loading products");
    result.description = result.description.replace('\n', "<br>");
    context.add("product", &result);
    return Template::render("item", &context);
}

#[post(
    "/learn",
    format = "application/x-www-form-urlencoded",
    data = "<category>"
)]
fn search_submit(category: String) -> Redirect {
    let mut splitter = category.splitn(2, '=');
    splitter.next();
    let second = splitter.next().unwrap();
    let result = format!("/learn/{}", second.to_lowercase());
    return Redirect::to(&result);
}

#[get("/learn/<category>")]
fn get_category(category: String, conn: DbConn) -> rocket_contrib::Template {
    let mut context = Context::new();
    let conn = &*conn;
    use schema::fields::dsl::*;
    let field = fields.filter(name.eq(&category)).first::<Field>(conn);
    let res;
    match field {
        Err(_) => return Template::render("index", Context::new()),
        Ok(f) => res = f,
    }
    let resources = products::dsl::products
        .filter(products::dsl::field_id.eq(res.id))
        .load::<Product>(conn)
        .expect("Issue loading products");
    context.add("title", &category);
    context.add("field", &res);
    context.add("resources", &resources);
    return Template::render("category", &context);
}

#[post(
    "/submission",
    data = "<i>",
    format = "application/x-www-form-urlencoded"
)]
fn submit(i: Form<UserSubmission>, conn: DbConn) -> rocket::response::Redirect {
    let alphabet: [char; 20] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h','m', 'k'
    ];
    let reference_code = nanoid::custom(6, &alphabet).to_uppercase();
    let input = i.get();
    let new_sub = NewSubmission {
        name: input.name.clone(),
        reference_code: reference_code.clone(),
        email: input.email.clone(),
        bio: input.bio.clone(),
        reference: input.reference.clone(),
        title: input.title.clone(),
        author: input.author.clone(),
        category: input.category.clone(),
        message: input.message.clone(),
        created_at: Utc::now().naive_utc().date(),
        updated_at: Utc::now().naive_utc().date()
    };
    let opt_sub = new_sub.save(conn);
    match opt_sub {
        Some(sub) => {
            let email = NewSubmissionEmail{identifier:reference_code, submission: sub};
            email.send();
            return Redirect::to("/thank_you");
        },
        None => return Redirect::to("/error")
    }
}

#[get("/submission")]
fn submission() -> rocket_contrib::Template {
    return Template::render("submission", Context::new());
}

#[get("/thank_you")]
fn thank_you() -> rocket_contrib::Template {
    return Template::render("thank_you", Context::new());
}

#[get("/about")]
fn about() -> rocket_contrib::Template {
    return Template::render("about", Context::new());
}
