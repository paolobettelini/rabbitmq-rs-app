#[macro_use]
extern crate diesel_migrations;

pub mod db;
pub mod models;

mod ops;
mod schema;

// <URL> --database-url mysql://worker:root@192.168.56.10:3306/service
// diesel migration generate data
// diesel migration run <URL>
// diesel print-schema > src/schema.rs <URL>