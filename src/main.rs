#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use models::{Link, NewLink};
use nanoid::nanoid;
use rocket_contrib::json::Json;
use rocket::request::{Form};
use serde::{Serialize, Deserialize};

pub mod schema;
pub mod models;

fn main() {
	rocket::ignite()
    .mount("/", routes![create_new_shortened_link, get_shortened_link])
    .launch();
}

#[derive(Serialize, Deserialize, Debug)]
struct ResJson {
  id: String
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorJson {
  error: String
}

#[derive(FromForm, Serialize, Debug)]
struct InputForm {
  url: String
}

impl ResJson {
  fn new(id: String) -> Self {
    Self {
      id
    }
  }
}

impl ErrorJson {
  fn new(error: String) -> Self {
    Self {
      error
    }
  }
}

#[post("/create", data = "<inputform>")]
fn create_new_shortened_link(inputform: Form<InputForm>)
  -> Result<Json<ResJson>, Json<ErrorJson>> {
  let id = nanoid!(5);
  if let Err(e) = url::Url::parse(&inputform.url) {
    return Err(Json(ErrorJson::new(format!("Invalid url: {}", e))))
  }

  let post = new_link(&id, &inputform.url);

  Ok(Json(ResJson::new(post.id)))
}

#[get("/get/<id>")]
fn get_shortened_link(id: String) -> Result<Json<InputForm>, Json<ErrorJson>> {
  match get_link(id.clone()) {
    Err(_) => Err(Json(ErrorJson::new(format!("No links match with {}", id)))),
    Ok(link) => Ok(Json(InputForm { url: link.url }))
  }
}

fn new_link<'a>(id: &'a str, url: &'a str) -> Link {
  use schema::links;
  let conn = connect_db();

  let new_link = NewLink {
    id,
    url
  };

  diesel::insert_into(links::table)
    .values(&new_link)
    .get_result(&conn)
    .expect("Error saving to db")
}

fn get_link(url_id: String) -> Result<Link, ()> {
  use schema::links::dsl::*;

  let conn = connect_db();
  let res = links.filter(id.eq(url_id))
    .limit(1)
    .load::<Link>(&conn)
    .expect("Error loading links");

  for link in res {
    return Ok(link)
  }

  Err(())
}

#[allow(dead_code)]
fn delete_link(id_to_del: String) -> usize {
  use schema::links::dsl::*;

  let conn = connect_db();
  let pattern = format!("%{}%", id_to_del);

  diesel::delete(links.filter(id.like(pattern)))
    .execute(&conn)
    .expect("Error deleting posts")
}

fn connect_db() -> PgConnection {
  dotenv().ok();
  let db_uri = env::var("DATABASE_URL")
    .expect("Could not get db url from .env");

  PgConnection::establish(&db_uri)
    .expect("Could not connect to db")
}
