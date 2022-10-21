use axum::{
    body::{boxed, Body},
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::Parser;
use log::{error, info, warn};
use std::{
    env,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::Path,
    str::FromStr,
};
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tokio::fs;
use std::path::PathBuf;

mod args;
mod rabbit;

use config::{Connectable, WebserverConfig};
use messaging::mb::*;

#[tokio::main]
async fn main() {
    // Read CLI arguments
    let args = args::Args::parse();

    // Read configuration path
    let config_path = Path::new(&args.config);
    let config = config::parse_config::<_, WebserverConfig>(config_path);
    let config = match config {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    // Setup logging system
    if let Some(log_config) = config.log {
        let env = env_logger::Env::default()
            .filter_or("RUST_LOG", log_config.log)
            .write_style_or("RUST_LOG_STYLE", log_config.style);

        env_logger::init_from_env(env);
    } else {
        env_logger::init();
    };

    // -----------------------------------
    info!("Starting");

    let mb_connection_url = if let Some(mb) = config.rabbit {
        mb.get_connection_string()
    } else {
        if let Ok(var) = env::var("AMQP_URL") {
            var
        } else {
            error!("No AMQP connection found in configuration file. You can also set DATABASE_URL");
            std::process::exit(1);
        }
    };

    // TODO: config
    let address = "0.0.0.0";
    let port: u16 = 8081;
    let static_dir = "./dist".to_string();

    let app = Router::new()
        .route("/api/hello", get(index))
        .merge(axum_extra::routing::SpaRouter::new("/assets", static_dir));

    let socket_addr =
        SocketAddr::from((IpAddr::from_str(address).expect("Invalid IP address"), port));

    info!("Listening on http://{}", socket_addr);

    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");
}

async fn index() -> impl IntoResponse {
    "Helloooooooooooooooooooo"
}
