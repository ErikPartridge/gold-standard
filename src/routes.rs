extern crate diesel;
extern crate nanoid;
extern crate r2d2;
extern crate rocket;
extern crate rocket_contrib;
extern crate tera;
use chrono::{NaiveDate, Utc};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use rand::{Rng, thread_rng};
use rocket::response::status::NotFound;
use rocket::http::Status;
use rocket::request;
use rocket::request::{Form, FromForm, FromRequest};
use rocket::response::NamedFile;
use rocket::response::Redirect;
use rocket::{Outcome, Request, State};
use rocket_contrib::Template;
use std::iter;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use tera::Context;
use strsim::normalized_damerau_levenshtein;
use diesel::prelude::*;
use mail::{Mailer, NewSubmissionEmail};
use model::field::Field;
use model::product::Product;
use model::submission::NewSubmission;
use schema::*;

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

#[derive(FromForm)]
struct SearchQuery{
   pub field: String
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
fn index(conn: DbConn) -> Result<Template, NotFound<String>> {
    let mut context = Context::new();
    let mut result = Field::all(conn);
    let empty : Vec<Field> = Vec::new();
    let mut subjs;
    match result {
        Some(subjects) => subjs = subjects,
        None => subjs = empty
    }
    let mut rng = thread_rng();
    rng.shuffle(&mut subjs);
    subjs.truncate(12);
    context.add("subjects", &subjs);
    return Ok(Template::render("index", context));
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

#[get("/item/<slug>")]
fn get_item(slug: String, conn: DbConn) -> Result<Template, NotFound<String>> {
    let conn = &*conn;
    let mut context = Context::new();
    let result = products::dsl::products
        .filter(products::dsl::slug.eq(slug))
        .first::<Product>(conn);
    let res;
    match result {
        Ok(r) => res = Product{description: r.description.replace('\n', "<br>"), .. r},
        Err(x) => return Err(NotFound(format!("There was an error accessing the database. Trace: {:?}", x)))
    }
    context.add("product", &res);
    return Ok(Template::render("item", &context));
}

#[post(
    "/learn",
    format = "application/x-www-form-urlencoded",
    data = "<sq>"
)]
fn search_submit(sq: Form<SearchQuery>, conn: DbConn) -> Result<Redirect, NotFound<String>> {
    let search_query = sq.get();
    let f_opt = Field::all(conn);
    let fields;
    match f_opt {
        Some(f) => fields = f,
        None => return Err(NotFound("Unable to connect to the database.".to_string()))
    }
    
    let mut best_score = -0.1;
    let mut best_field: Option<Field> = None;
    for field in fields {
        let score = normalized_damerau_levenshtein(&field.name, &search_query.field);
        if(score > best_score) {
            best_field = Some(field.clone());
            best_score = score;
        }
        for synonym in field.clone().synonyms {
            let syn_score = normalized_damerau_levenshtein(&synonym, &search_query.field);
            if (syn_score > best_score) {
                best_field = Some(field.clone());
                best_score = score;
            }
        }
        if score == 1.0 {
            break;
        }
    }
    let result;
    match best_field {
        Some(field) => result = format!("/learn/{}", field.name.to_lowercase()),
        None => return Err(NotFound("No such subject found.".to_string()))
    }
    if best_score < 0.7 {
        return Err(NotFound("No confidence regarding subject matches.".to_string()));
    }
    return Ok(Redirect::to(&result));
}

#[get("/learn/<category>")]
fn get_category(category: String, conn: DbConn) -> Result<Template, NotFound<String>> {
    let mut context = Context::new();
    let conn = &*conn;
    use schema::fields::dsl::*;
    let field = fields.filter(name.eq(&category)).first::<Field>(conn);
    let res;
    match field {
        Err(_) => return Err(NotFound(format!("Unable to locate field with that name."))),
        Ok(f) => res = f,
    }
    let r = products::dsl::products
        .filter(products::dsl::field_id.eq(res.id))
        .load::<Product>(conn);
    let resources;
    match r {
        Ok(result) => resources = result,
        Err(_) => return Err(NotFound("Couldn't find any resources".to_string()))
    }
    context.add("title", &category);
    context.add("field", &res);
    context.add("resources", &resources);
    return Ok(Template::render("category", &context));
}

#[post(
    "/submission",
    data = "<i>",
    format = "application/x-www-form-urlencoded"
)]
fn submit(i: Form<UserSubmission>, conn: DbConn) -> rocket::response::Redirect {
    let alphabet: [char; 29] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'm', 'k', 'n', 'p', 'q', 'r','s','v','w','x','z'
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
        updated_at: Utc::now().naive_utc().date(),
    };
    let opt_sub = new_sub.save(conn);
    match opt_sub {
        Some(sub) => {
            let email = NewSubmissionEmail {
                identifier: reference_code,
                submission: sub,
            };
            email.send();
            return Redirect::to("/thank_you");
        }
        None => return Redirect::to("/error"),
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
