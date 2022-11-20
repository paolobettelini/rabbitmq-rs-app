use diesel::prelude::*;
use diesel_migrations::*;

use crate::models::{NewUser, User, NewImage, Image};
use crate::ops::user_ops as users;
use crate::ops::image_ops as images;

pub struct Database {
    connection: MysqlConnection,
}

impl Database {
    pub fn new(url: String) -> Self {
        let connection = MysqlConnection::establish(&url).unwrap();

        Self { connection }
    }

    pub fn create_user(&mut self, mail: &str, username: &str, password: &Vec<u8>, token: &Vec<u8>) {
        let new_user = NewUser { mail, username, password, token};

        users::create_user(&mut self.connection, new_user);
    }

    pub fn user_exists(&mut self, username: &str) -> bool {
        users::user_exists(&mut self.connection, username)
    }

    pub fn mail_exists(&mut self, user_mail: &str) -> bool {
        users::mail_exists(&mut self.connection, user_mail)
    }

    pub fn get_user(&mut self, username: &str) -> Option<User> {
        users::get_user(&mut self.connection, username)
    }

    pub fn get_username(&mut self, token: &Vec<u8>) -> Option<String> {
        users::get_username(&mut self.connection, token)
    }

    pub fn get_token_for(&mut self, username: &str) -> Option<Vec<u8>> {
        users::get_token_for(&mut self.connection, username)
    }

    pub fn get_user_id(&mut self, username: &str) -> Option<i32> {
        users::get_user_id(&mut self.connection, username)
    }

    pub fn insert_image(&mut self, username: &str, data: &Vec<u8>) {
        if let Some(user_id) = self.get_user_id(&username) {
            let total_images = self.get_total_images(username) as i32;
            let id = total_images + 1;
            
            let new_image = NewImage { id, user_id, data };

            images::insert_image(&mut self.connection, &username, new_image);
        }
    }

    pub fn get_image(&mut self, username: &str, index: i32) -> Option<Image> {
        images::get_image(&mut self.connection, username, index)
    }

    pub fn get_total_images(&mut self, username: &str) -> u32 {
        images::get_total_images(&mut self.connection, username)
    }

    pub fn run_embedded_migrations(&mut self) {
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

        self.connection.run_pending_migrations(MIGRATIONS);
    }
}
