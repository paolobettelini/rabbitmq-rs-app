use diesel::{prelude::*, r2d2::{PooledConnection, Pool, ConnectionManager}};
use diesel_migrations::*;

use crate::{
    models::{NewUser, User, NewImage, Image, NewLog},
    ops::{
        user_ops as users,
        image_ops as images,
        log_ops as logs
    }
};

type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;

pub struct Database {
    pool: MysqlPool,
}

impl Database {
    pub fn new(url: String) -> Self {
        let pool = MysqlPool::builder()
            .max_size(10)
            .build(ConnectionManager::new(url))
            .unwrap();

        Self { pool }
    }

    fn get_connection(&self) -> PooledConnection<ConnectionManager<diesel::MysqlConnection>> {
        self.pool.get().unwrap()
    }

    pub fn create_user(&self, mail: &str, username: &str, password: &Vec<u8>, token: &Vec<u8>) {
        let new_user = NewUser { mail, username, password, token};

        users::create_user(&mut self.get_connection(), new_user);
    }

    pub fn user_exists(&self, username: &str) -> bool {
        users::user_exists(&mut self.get_connection(), username)
    }

    pub fn mail_exists(&self, user_mail: &str) -> bool {
        users::mail_exists(&mut self.get_connection(), user_mail)
    }

    pub fn get_user(&self, username: &str) -> Option<User> {
        users::get_user(&mut self.get_connection(), username)
    }

    pub fn get_username(&self, token: &Vec<u8>) -> Option<String> {
        users::get_username(&mut self.get_connection(), token)
    }

    pub fn get_token_for(&self, username: &str) -> Option<Vec<u8>> {
        users::get_token_for(&mut self.get_connection(), username)
    }

    pub fn get_user_id(&self, username: &str) -> Option<i32> {
        users::get_user_id(&mut self.get_connection(), username)
    }

    pub fn insert_image(&self, username: &str, data: &Vec<u8>) {
        if let Some(user_id) = self.get_user_id(&username) {
            let total_images = self.get_total_images(username) as i32;
            let id = total_images + 1;
            
            let new_image = NewImage { id, user_id, data };

            images::insert_image(&mut self.get_connection(), &username, new_image);
        }
    }

    pub fn get_image(&self, username: &str, index: i32) -> Option<Image> {
        images::get_image(&mut self.get_connection(), username, index)
    }

    pub fn get_total_images(&self, username: &str) -> u32 {
        images::get_total_images(&mut self.get_connection(), username)
    }

    pub fn insert_log(&self, message: &str) {
        let new_log = NewLog { message };

        logs::insert_log(&mut self.get_connection(), new_log);
    }

    pub fn run_embedded_migrations(&self) {
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

        self.get_connection().run_pending_migrations(MIGRATIONS).unwrap();
    }
}
