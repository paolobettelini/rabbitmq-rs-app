use crate::models::{NewImage, Image};
use diesel::prelude::*;

pub fn insert_image(connection: &mut MysqlConnection, username: &str, new_image: NewImage) {
    use crate::schema::image::dsl::*;
    
    diesel::insert_into(image)
        .values(&new_image)
        .execute(connection)
        .expect("Error saving new user");
}

pub fn get_total_images(connection: &mut MysqlConnection, name: &str) -> u32 {
    use crate::schema::image::{id, user_id, dsl::image};
    use crate::schema::user::{id as id_field, username, dsl::user};
    use diesel::{select, dsl::count};
    
    let id_to_filter: i32 = user
        .filter(username.eq(name))
        .select(id_field)
        .first(connection)
        .unwrap_or(0); // TODO return option as below

    let result = image
        .filter(user_id.eq(id_to_filter))
        .select(count(id))
        .first(connection)
        .unwrap_or(0);

    result.try_into().unwrap_or(0u32)
}

pub fn get_image(connection: &mut MysqlConnection, name: &str, index: i32) -> Option<Vec<u8>> {
    use crate::schema::image::{id, user_id, data, dsl::image};
    use crate::schema::user::{id as id_field, username, dsl::user};
    use diesel::select;

    let id_to_filter: i32 = {
        let result = user
            .filter(username.eq(name))
            .select(id_field)
            .first(connection);

        if let Ok(value) = result {
            value
        } else {
            return None;
        }
    };

    let result: Result<Vec<u8>, _> = image
        .filter(user_id.eq(id_to_filter))
        .select(data)
        .first(connection);

    if let Ok(bytes) = result {
        Some(bytes)
    } else {
        return None;
    }
}