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
    
    let result: Result<User, _> = user
        .filter(username.eq(name))
        .first(connection);

    if let Ok(data) = result {
        Some(data)
    } else {
        None
    }
}

pub fn user_exists(connection: &mut MysqlConnection, name: &str) -> bool {
    use crate::schema::user::dsl::*;
    use diesel::{select, dsl::exists};
    
    let result = select(exists(user.filter(username.eq(name))))
        .get_result(connection);

    if let Ok(res) = result {
        res
    } else {
        false
    }
}

pub fn mail_exists(connection: &mut MysqlConnection, user_mail: &str) -> bool {
    use crate::schema::user::dsl::*;
    use diesel::{select, dsl::exists};
    
    let result = select(exists(user.filter(mail.eq(user_mail))))
        .get_result(connection);

    if let Ok(res) = result {
        res
    } else {
        false
    }
}

pub fn get_token_for(connection: &mut MysqlConnection, name: &str) -> Option<Vec<u8>> {
    use crate::schema::user::{username, token, dsl::user};
    
    
    let result: Result<Vec<u8>, _> = user
        .select(token)
        .filter(username.eq(name))
        .first(connection);

    if let Ok(data) = result {
        Some(data)
    } else {
        None
    }
}

pub fn get_user_id(connection: &mut MysqlConnection, name: &str) -> Option<i32> {
    use crate::schema::user::{username, id, dsl::user};
    
    
    let result: Result<i32, _> = user
        .select(id)
        .filter(username.eq(name))
        .first(connection);

    if let Ok(data) = result {
        Some(data)
    } else {
        None
    }
}

pub fn get_username(connection: &mut MysqlConnection, auth_token: &Vec<u8>) -> Option<String> {
    use crate::schema::user::{username, token, dsl::user};
    

    let result: Result<String, _> = user
        .select(username)
        .filter(token.eq(auth_token))
        .first(connection);

    if let Ok(data) = result {
        Some(data)
    } else {
        None
    }
}