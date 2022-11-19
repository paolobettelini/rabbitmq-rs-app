use diesel::prelude::*;
use diesel_migrations::*;

use crate::models::{NewUser, User};
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

    pub fn create_user(&mut self, new_user: NewUser) {
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

    pub fn get_total_images(&mut self, username: &str) -> u32 {
        images::get_total_images(&mut self.connection, username)
    }

    pub fn run_embedded_migrations(&mut self) {
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

        self.connection.run_pending_migrations(MIGRATIONS);
    }
}
