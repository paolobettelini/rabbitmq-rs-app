use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Deserialize;

use crate::schema::{image, user};

#[derive(Queryable, Debug)]
#[diesel(table_name = user)]
pub struct User {
    pub id: i32,
    pub mail: String,
    pub username: String,
    pub password: Vec<u8>,
    pub token: Vec<u8>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = user)]
pub struct NewUser<'a> {
    pub mail: &'a str,
    pub username: &'a str,
    pub password: &'a Vec<u8>,
    pub token: &'a Vec<u8>,
}

#[derive(Queryable, Debug)]
#[diesel(belongs_to(User))]
#[diesel(table_name = image)]
pub struct Image {
    pub id: i32,
    pub user_id: i32,
    pub uploaded_at: NaiveDateTime,
    pub data: Vec<u8>,
}

#[derive(Insertable)]
#[diesel(belongs_to(User))]
#[diesel(table_name = image)]
pub struct NewImage<'a> {
    pub id: i32,
    pub user_id: i32,
    pub data: &'a Vec<u8>,
}
