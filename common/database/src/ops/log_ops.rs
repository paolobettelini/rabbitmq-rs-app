use crate::models::NewLog;
use diesel::prelude::*;

pub fn insert_log(connection: &mut MysqlConnection, new_log: NewLog) {
    use crate::schema::log::dsl::*;

    diesel::insert_into(log)
        .values(&new_log)
        .execute(connection)
        .expect("Error saving log");
}