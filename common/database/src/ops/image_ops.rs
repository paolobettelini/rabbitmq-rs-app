use crate::models::{NewImage, Image};
use diesel::prelude::*;

pub fn insert_image(connection: &mut MysqlConnection, _username: &str, new_image: NewImage) {
    use crate::schema::image::dsl::*;

    diesel::insert_into(image)
        .values(&new_image)
        .execute(connection)
        .expect("Error saving new image");
}

pub fn get_total_images(connection: &mut MysqlConnection, name: &str) -> u32 {
    use crate::schema::image::{id, user_id, dsl::image};
    use crate::schema::user::{id as id_field, username, dsl::user};
    use diesel::{dsl::count};
    
    let id_to_filter: i32 = user
        .filter(username.eq(name))
        .select(id_field)
        .first(connection)
        .unwrap_or(0);

    let result = image
        .filter(user_id.eq(id_to_filter))
        .select(count(id))
        .first(connection)
        .unwrap_or(0);

    result.try_into().unwrap_or(0u32)
}

pub fn get_image(connection: &mut MysqlConnection, name: &str, index: i32) -> Option<Image> {
    use crate::schema::image::{id as image_id, user_id as user_id_image, dsl::image};
    use crate::schema::user::{id as user_id_user, username, dsl::user};
    

    let id_to_filter: i32 = {
        let result = user
            .filter(username.eq(name))
            .select(user_id_user)
            .first(connection);

        if let Ok(value) = result {
            value
        } else {
            return None;
        }
    };

    let result: Result<Image, _> = image
        .filter(user_id_image.eq(id_to_filter).and(image_id.eq(index)))
        .first(connection);

    if let Ok(bytes) = result {
        Some(bytes)
    } else {
        return None;
    }
}