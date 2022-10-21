use crate::models::{NewUser, User};
use diesel::prelude::*;

pub fn create_user(connection: &mut MysqlConnection, new_user: NewUser) {
    use crate::schema::user::dsl::*;
    
    diesel::insert_into(user)
        .values(&new_user)
        .execute(connection)
        .expect("Error saving new user");
}

pub fn get_user(connection: &mut MysqlConnection, name: &str) -> Option<User> {
    use crate::schema::user::{username, dsl::user};
    use diesel::{select};

    /*
    let result: User = user
        .filter(username.eq(name))
        //.limit(1)
        .load(connection)
        .unwrap_or(vec![])
        .get(0)
        .unwrap();*/

    None
}

pub fn user_exists(connection: &mut MysqlConnection, name: &str) -> bool {
    use crate::schema::user::dsl::*;
    use diesel::{select, dsl::exists};
    
    let result = select(exists(user.filter(username.eq(name))))
        .get_result(connection);

    if let Ok(true) = result {
        true
    } else {
        false
    }
}