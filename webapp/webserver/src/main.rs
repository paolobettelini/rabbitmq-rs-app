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
mod app;

use config::{Connectable, WebserverConfig};
use messaging::mb::*;
use app::*;

#[macro_use]
extern crate lazy_static;

use warp::Filter;

lazy_static! {
    pub static ref CONFIG: Box<WebserverConfig> = {
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

        config
    };
}

#[tokio::main]
async fn main() {
    // Setup logging system
    if let Some(log_config) = &CONFIG.log {
        let env = env_logger::Env::default()
            .filter_or("RUST_LOG", &log_config.log)
            .write_style_or("RUST_LOG_STYLE", &log_config.style);

        env_logger::init_from_env(env);
    } else {
        env_logger::init();
    };

    // -----------------------------------
    info!("Starting");

    let mb_connection_url = if let Some(mb) = &CONFIG.rabbit {
        mb.get_connection_string()
    } else {
        if let Ok(var) = env::var("AMQP_URL") {
            var
        } else {
            error!("No AMQP connection found in configuration file. You can also set DATABASE_URL");
            std::process::exit(1);
        }
    };

    let mut app = App::new();
    app.start(&CONFIG.http.www).await;
/*
    let static_files = warp::path("static")
        .and(warp::fs::dir(CONFIG.http.www));
        
    let index_route = warp::path::end().map(|| "index override");
    let login_route = warp::path("login.html").map(|| "login override");

    let routes = index_route
        .or(static_files)
        .or(login_route);

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;*/
}