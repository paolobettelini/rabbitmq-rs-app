use clap::Parser;
use bytes::BufMut;
use log::{debug, error, info, warn};
use once_cell::sync::OnceCell;
use serde_json::json;
use serde::Deserialize;
use std::path::PathBuf;
use std::{
    collections::HashMap,
    env,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::Path,
    sync::Arc,
};
use futures::TryStreamExt;
use tokio::sync::Mutex;
use warp::{
    filters::cookie,
    http::{Response, StatusCode},
    multipart::{FormData, Part},
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
    // TODO from config
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

fn get_routes(www: &'static str) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let static_files = warp::fs::dir(www);

    macro_rules! bad_request {
        () => {
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/html; charset=UTF-8")
                .body("BAD_REQUEST".to_owned())
                .unwrap()
        };
    }

    let index_page = warp::path::end()
        .and(cookie::optional::<String>("token"))
        .then(|token| async {
            let content = APP.get().unwrap().lock().await.render_index(token);

            reply::html(content)
        });

    let login_page = warp::path("login")
        .and(cookie::optional::<String>("token"))
        .then(|token: Option<String>| async move {
            let app = APP.get().unwrap().lock().await;

            if token.is_some() {
                return Response::builder()
                    .status(StatusCode::FOUND)
                    .header("Location", "/logout")
                    .body("".to_owned())
                    .unwrap();
            }

            let content = app.render_login(None);

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=UTF-8")
                .body(content)
                .unwrap()
        });

    let register_page = warp::path("register")
        .and(cookie::optional::<String>("token"))
        .then(|token: Option<String>| async move {
            let app = APP.get().unwrap().lock().await;

            if token.is_some() {
                return Response::builder()
                    .status(StatusCode::FOUND)
                    .header("Location", "/logout")
                    .body("".to_owned())
                    .unwrap();
            }

            let content = app.render_register(None);

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=UTF-8")
                .body(content)
                .unwrap()
        });

    let logout_page = warp::path("logout")
        .and(cookie::cookie::<String>("token"))
        .then(|token| async move {
            let app = APP.get().unwrap().lock().await;

            let content = app.render_logout();

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=UTF-8")
                .body(content)
                .unwrap()
        });

    let gallery_page = warp::path("gallery")
        .and(cookie::cookie::<String>("token"))
        .then(|token| async move {
            let app = APP.get().unwrap().lock().await;

            let content = app.render_gallery();

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=UTF-8")
                .body(content)
                .unwrap()
        });

    let upload_page = warp::path("upload")
        .and(cookie::cookie::<String>("token"))
        .then(|token| async move {
            let app = APP.get().unwrap().lock().await;

            let content = app.render_upload();

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=UTF-8")
                .body(content)
                .unwrap()
        });

    macro_rules! read_form {
        ($form:tt, $var: tt) => {
            let $var = $form.get(stringify!($var));
            let $var = if let Some($var) = $var {
                $var
            } else {
                return bad_request!();
            };
        };
    }

    let login_api = warp::path!("api" / "login")
        .and(warp::body::form())
        .and(warp::post())
        .then(|form: HashMap<String, String>| async move {
            read_form!(form, username);
            read_form!(form, password);

            debug!("Register event: [{username}, {password}]");

            let password = if let Ok(bytes) = utils::from_base64(password) {
                bytes
            } else {
                return bad_request!();
            };

            let username = username.to_string();
            let login_req = LoginRequestData { username, password };

            let app = APP.get().unwrap().lock().await;

            let response = app.send_login_request(login_req).await;

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
                }
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

            debug!("Register event: [{username}, {email}, {password}]");

            let password = if let Ok(bytes) = utils::from_base64(password) {
                bytes
            } else {
                return bad_request!();
            };

            let register_req = RegisterRequestData {
                mail: email.to_string(),
                username: username.to_string(),
                password: password,
            };

            let app = APP.get().unwrap().lock().await;

            let response = app.send_register_request(register_req).await;

            match response {
                RegisterResponseData::Ok(data) => {
                    // set cookie
                    let token = utils::to_base64(data.token);
                    let cookie = format!("token={}; Path=/; HttpOnly; Max-Age=1209600", token);

                    return Response::builder()
                        .status(StatusCode::FOUND)
                        .header("Location", "/")
                        .header("Set-Cookie", cookie)
                        .body(String::from(""))
                        .unwrap();
                }
                RegisterResponseData::Err(err) => {
                    let content = app.render_register(Some(err));

                    return Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "text/html; charset=UTF-8")
                        .body(content)
                        .unwrap();
                }
            };
        });

    let logout_api = warp::path!("api" / "logout").and(warp::post()).map(|| {
        let cookie = "token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT";

        return Response::builder()
            .status(StatusCode::FOUND)
            .header("Location", "/")
            .header("Set-Cookie", cookie)
            .body("".to_owned())
            .unwrap();
    });

    let upload_api = warp::path!("api" / "upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(2_500_000))
        .and(cookie::cookie::<String>("token"))
        .then(|form: FormData, token| async move {
            debug!("Receive file");

            let token = {
                let result = utils::from_base64(token);

                if let Ok(value) = result {
                    value
                } else {
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body("".to_string())
                        .unwrap();
                }
            };

            macro_rules! json_response {
                ($msg:tt) => {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .body(json!({
                            "response": $msg
                        }).to_string())
                        .unwrap()
                };
            }
            
            let parts = {
                let parts: Result<Vec<Part>, _> = form
                    .try_collect()
                    .await;

                if let Ok(data) = parts {
                    data
                } else {
                    return json_response!("Invalid Request");
                }
            };

            for part in parts {
                if part.name() != "file" {
                    return json_response!("Invalid Request");
                }

                if let Some(content_type) = part.content_type() {
                    if !content_type.starts_with("image") {
                        return json_response!("Invalid Image");
                    }
                } else {
                    return json_response!("Invalid Image");
                }
                
                let image = {
                    let value = part
                    .stream()
                    .try_fold(Vec::new(), |mut vec, data| {
                        vec.put(data);
                        async move { Ok(vec) }
                    })
                    .await;

                    if let Ok(data) = value {
                        data
                    } else {
                        return json_response!("An error has occured");
                    }
                };

                let app = APP.get().unwrap().lock().await;
                
                let upload_req = ShrinkAndUploadData { token, image }; 

                let response = app.send_upload_request(upload_req).await;
                
                let status = match response {
                    ShrinkAndUploadResponseData::Ok => "Ok",
                    ShrinkAndUploadResponseData::InvalidImage => "Invalid Image",
                };

                return json_response!(status);

                break;
            }


            json_response!("Invalid Request")
        });

    let get_image_api = warp::path!("api" / "image" / u16).then(|index| async move {
        Response::builder()
            .status(StatusCode::OK)
            .body("".to_owned())
            .unwrap()
    });

    let index_block = warp::path::path("index.html")
        .map(|| reply::with_status("404 NOT_FOUND", StatusCode::NOT_FOUND));
    let login_block = warp::path::path("login.html")
        .map(|| reply::with_status("404 NOT_FOUND", StatusCode::NOT_FOUND));
    let register_block = warp::path::path("register.html")
        .map(|| reply::with_status("404 NOT_FOUND", StatusCode::NOT_FOUND));
    let logout_block = warp::path::path("logout.html")
        .map(|| reply::with_status("404 NOT_FOUND", StatusCode::NOT_FOUND));
    let upload_block = warp::path::path("upload.html")
        .map(|| reply::with_status("404 NOT_FOUND", StatusCode::NOT_FOUND));
    let gallery_block = warp::path::path("gallery.html")
        .map(|| reply::with_status("404 NOT_FOUND", StatusCode::NOT_FOUND));

    let templates = index_page
        .or(login_page)
        .or(register_page)
        .or(logout_page)
        .or(upload_page)
        .or(gallery_page);

    let methods = login_api
        .or(register_api)
        .or(logout_api)
        .or(upload_api)
        .or(get_image_api);

    let blocks = index_block
        .or(login_block)
        .or(register_block)
        .or(logout_block)
        .or(upload_block)
        .or(gallery_block);

    let routes = methods.or(blocks).or(templates).or(static_files);

    routes
}
