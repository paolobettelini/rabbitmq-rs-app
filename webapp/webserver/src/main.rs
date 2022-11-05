use clap::Parser;
use log::{error, info, warn};
use std::{
    env,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::Path,
    str::FromStr,
    sync::Arc
};
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tokio::{fs, sync::Mutex};
use std::path::PathBuf;
use warp::{Filter, Rejection, Reply, reply, http::StatusCode, filters::cookie};
use once_cell::sync::OnceCell;
use serde::Deserialize;

mod args;
mod app;

use config::{Connectable, WebserverConfig};
use messaging::mb::*;
use app::*;

#[macro_use]
extern crate lazy_static;

static APP: OnceCell<Arc<Mutex<App>>> = OnceCell::new();

lazy_static! {
    static ref CONFIG: Box<WebserverConfig> = {
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

    let app: Arc<Mutex<App>> = {
        let mb_connection_url = if let Some(mb) = &CONFIG.rabbit {
            mb.get_connection_string()
        } else {
            if let Ok(var) = env::var("AMQP_URL") {
                var
            } else {
                error!("No AMQP connection found in configuration file. You can also set AMQP_URL");
                std::process::exit(1);
            }
        };

        create_app(&CONFIG.http.www, &mb_connection_url).await
    };
    
    APP.set(app).unwrap();
    
    info!("Starting");
    
    start_service(&CONFIG.http.www).await;
}

async fn create_app(www: &str, amqp: &str) -> Arc<Mutex<App>> {
    Arc::new(Mutex::new(App::new(www, amqp).await))
}

async fn start_service(www: &'static str) {
    let routes = get_routes(www);
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

fn get_routes(www: &'static str) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let static_files = warp::fs::dir(www);
    
    let index_route = warp::path::end()
        .and(cookie::optional::<String>("token"))
        .and(cookie::optional::<String>("token2"))
        .then(|token, tokn2| async {
        let content = APP.get().unwrap().lock().await.render_index(token);

        reply::html(content)
    });
    let login_route = warp::path("login").map(|| "login override");
    let register_route = warp::path("register").map(|| "register override");

    let index_block = warp::path::path("index.html")
        .map(|| reply::with_status("404 NOT_FOUND", StatusCode::NOT_FOUND));
    //let login_block = warp::path::path("login.html")
    //    .map(|| reply::with_status("404 NOT_FOUND", StatusCode::NOT_FOUND));
    //let register_block = warp::path::path("register.html")
    //    .map(|| reply::with_status("404 NOT_FOUND", StatusCode::NOT_FOUND));

    let methods = index_route
        .or(login_route)
        .or(register_route);

    let blocks = index_block;
    //    .or(login_block)
    //    .or(register_block);

    let routes = methods
        .or(blocks)
        .or(static_files);

    routes
}