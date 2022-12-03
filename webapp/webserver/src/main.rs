use clap::Parser;

use log::{error, info};

use std::{env, net::IpAddr, sync::Arc};

mod app;
mod args;
mod handler;
mod utils;

use app::*;
use config::Connectable;

#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() {
    // Setup logging system
    if let Some(log_config) = &handler::CONFIG.log {
        let env = env_logger::Env::default()
            .filter_or("RUST_LOG", &log_config.log)
            .write_style_or("RUST_LOG_STYLE", &log_config.style);

        env_logger::init_from_env(env);
    } else {
        env_logger::init();
    };

    let app: Arc<App> = {
        let mb_connection_url = if let Some(mb) = &handler::CONFIG.rabbit {
            mb.get_connection_string()
        } else {
            if let Ok(var) = env::var("AMQP_URL") {
                var
            } else {
                error!("No AMQP connection found in configuration file. You can also set AMQP_URL");
                std::process::exit(1);
            }
        };

        create_app(&handler::CONFIG.http.www, &mb_connection_url).await
    };

    handler::init_app(app);

    info!("Starting");

    start_service(
        &handler::CONFIG.http.www,
        &handler::CONFIG.http.ip,
        handler::CONFIG.http.port,
    )
    .await;
}

async fn create_app(www: &str, amqp: &str) -> Arc<App> {
    Arc::new(App::new(www, amqp).await)
}

async fn start_service(www: &'static str, ip: &'static str, port: u16) {
    let routes = handler::get_routes(www);

    let ip = if let Ok(address) = ip.parse::<IpAddr>() {
        address
    } else {
        error!("Invalid IP");
        std::process::exit(1);
    };

    warp::serve(routes).run((ip, port)).await;
}
