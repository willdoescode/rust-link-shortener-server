#[derive(Queryable)]
pub struct Link {
	pub id: String,
	pub url: String
}

use super::schema::links;

#[derive(Insertable)]
#[table_name = "links"]
pub struct NewLink<'a> {
	pub id: &'a str,
	pub url: &'a str
}