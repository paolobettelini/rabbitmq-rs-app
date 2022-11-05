use clap::Parser;
use log::{error, info, warn};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::path::PathBuf;
use std::{
    collections::HashMap,
    env,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::Path,
    str::FromStr,
    sync::Arc,
};
use tokio::{fs, sync::Mutex};
use tower_http::services::ServeDir;
use warp::{
    filters::cookie,
    http::{Response, StatusCode},
    reply, Filter, Rejection, Reply,
};

mod app;
mod args;
mod utils;

use app::*;
use config::{Connectable, WebserverConfig};
use messaging::mb::*;
use protocol::rabbit::*;

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

    let index_page = warp::path::end()
        .and(cookie::optional::<String>("token"))
        .and(cookie::optional::<String>("token2"))
        .then(|token, tokn2| async {
            let content = APP.get().unwrap().lock().await.render_index(token);

            reply::html(content)
        });

    let login_page = warp::path("login")
        .then(|| async {
            let content = APP
                .get()
                .unwrap()
                .lock()
                .await
                .render_login(None);

            reply::html(content)
        });

    macro_rules! bad_request {
        () => {
            Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "text/html; charset=UTF-8")
                    .body("BAD_REQUEST".to_owned())
                    .unwrap()
        }
    }

    macro_rules! read_form {
        ($form:tt, $var: tt) => {
            let $var = $form.get(stringify!($var));
            let $var = if let Some($var) = $var {
                $var
            } else {
                return bad_request!();
            };
        }
    }

    let login_api = warp::path!("api" / "login")
        .and(warp::body::form())
        .and(warp::post())
        .then(|form: HashMap<String, String>| async move {
            read_form!(form, username);
            read_form!(form, email);
            read_form!(form, password);

            let password = if let Ok(bytes) = utils::from_base64(password) {
                bytes
            } else {
                return bad_request!();
            };

            let login_req = LoginRequestData {
                mail: email.to_string(),
                username: username.to_string(),
                password: password,
            };

            let app = APP.get()
                .unwrap()
                .lock()
                .await;

            let response = app
                .send_login_request(login_req)
                .await;

            match response {
                LoginResponseData::Ok(data) => {
                    // set cookie
                    let token = utils::to_base64(data.token);
                    let cookie = format!("token={}; Path=/; HttpOnly; Max-Age=1209600", token);
                    
                    return Response::builder()
                        .status(StatusCode::FOUND)
                        .header("Location", "/")
                        .header("Set-Cookie", cookie)
                        .body(String::from(""))
                        .unwrap();
                },
                LoginResponseData::Err(err) => {
                    let content = app.render_login(Some(err));

                    return Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "text/html; charset=UTF-8")
                        .body(content)
                        .unwrap();
                }
            };

        });

    let register_api = warp::path!("api" / "register")
        .and(warp::body::form())
        .and(warp::post())
        .then(|form: HashMap<String, String>| async move {
            read_form!(form, username);
            read_form!(form, email);
            read_form!(form, password);

            let content = format!("Username: {username}, Password: {password}");

            return Response::builder()
                .status(StatusCode::FOUND)
                .header("Location", "/")
                .body(content)
                .unwrap();
        });

    let index_block = warp::path::path("index.html")
        .map(|| reply::with_status("404 NOT_FOUND", StatusCode::NOT_FOUND));
    //let login_block = warp::path::path("login.html")
    //    .map(|| reply::with_status("404 NOT_FOUND", StatusCode::NOT_FOUND));
    //let register_block = warp::path::path("register.html")
    //    .map(|| reply::with_status("404 NOT_FOUND", StatusCode::NOT_FOUND));

    let templates = index_page
        .or(login_page);

    let methods = login_api
        .or(register_api);

    let blocks = index_block;
    //    .or(login_block)
    //    .or(register_block);

    let routes = methods
        .or(blocks)
        .or(templates)
        .or(static_files);

    routes
}

/*
            return Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=UTF-8")
                .body(content)
                .unwrap();
*/
