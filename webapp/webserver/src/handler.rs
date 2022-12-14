use bytes::BufMut;
use clap::Parser;
use futures::TryStreamExt;
use log::debug;
use once_cell::sync::OnceCell;

use serde_json::json;

use std::{collections::HashMap, path::Path, sync::Arc};
use warp::{
    filters::cookie,
    http::{Response, StatusCode},
    multipart::{FormData, Part},
    reply, Filter, Rejection, Reply,
};

use crate::args;
use crate::utils;
use crate::App;

use config::WebserverConfig;

use protocol::{rabbit::*, validation::*};

pub static APP: OnceCell<Arc<App>> = OnceCell::new();

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

pub fn init_app(app: Arc<App>) {
    APP.set(app).unwrap();
}

fn get_app() -> Arc<App> {
    APP.get().unwrap().clone()
}

#[allow(opaque_hidden_inferred_bound)]
pub fn get_routes(
    www: &'static str,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
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

    macro_rules! log {
        ($app:tt, $msg:tt) => {
            $app.send_log_message(LogData { message: $msg.to_owned() }).await
        }
    }

    let index_page = warp::path::end()
        .and(cookie::optional::<String>("token"))
        .then(|token| async {
            let content = get_app().render_index(token);

            reply::html(content)
        });

    let login_page = warp::path("login")
        .and(cookie::optional::<String>("token"))
        .then(|token: Option<String>| async move {
            let app = get_app();

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
            let app = get_app();

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
        .then(|_token| async move {
            let app = get_app();

            let content = app.render_logout();

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=UTF-8")
                .body(content)
                .unwrap()
        });

    let gallery_page = warp::path("gallery")
        .and(cookie::cookie::<String>("token"))
        .then(|_token| async move {
            let app = get_app();

            let content = app.render_gallery();

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=UTF-8")
                .body(content)
                .unwrap()
        });

    let upload_page = warp::path("upload")
        .and(cookie::cookie::<String>("token"))
        .then(|_token| async move {
            let app = get_app();

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

            if !validate_username(&username) {
                return bad_request!();
            }

            let username = username.to_string();
            let login_req = LoginRequestData { username, password };

            let app = get_app();

            let response = app.send_login_request(login_req).await;
            
            match response {
                LoginResponseData::Ok(data) => {
                    log!(app, "User sent successful login request");

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
                    log!(app, "User sent failed login request");

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
            if !(validate_username(&username) && validate_email(&email)) {
                return bad_request!();
            }

            let username = username.to_string();
            let mail = email.to_string();
            let register_req = RegisterRequestData {
                mail,
                username,
                password,
            };

            let app = get_app();

            let response = app.send_register_request(register_req).await;

            match response {
                RegisterResponseData::Ok(data) => {
                    log!(app, "User sent successful register request");

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
                    log!(app, "User sent failed register request");
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

    macro_rules! json_response {
        ($msg:tt) => {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(json!({ "response": $msg }).to_string())
                .unwrap()
        };
    }

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
                    return bad_request!();
                }
            };

            let parts = {
                let parts: Result<Vec<Part>, _> = form.try_collect().await;

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

                let app = get_app();

                let upload_req = ShrinkAndUploadData { token, image };

                let response = app.send_upload_request(upload_req).await;

                log!(app, "User uploaded image");

                return json_response!(response);
            }

            json_response!("Invalid Request")
        });

    let get_image_api = warp::path!("api" / "image" / u16)
        .and(cookie::cookie::<String>("token"))
        .then(|index, token| async move {
            let token = {
                let result = utils::from_base64(token);

                if let Ok(value) = result {
                    value
                } else {
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(vec![])
                        .unwrap();
                }
            };

            let app = get_app();

            let get_image_req = GetImageData { token, index };

            let response = app.send_get_image_request(get_image_req).await;
            log!(app, "User requested image");

            return Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "image/*")
                .body(response)
                .unwrap();
        });

    let get_total_images_api = warp::path!("api" / "total-images")
        .and(cookie::cookie::<String>("token"))
        .then(|token| async move {
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

            let app = get_app();

            let get_request = GetTotalImagesData { token };

            let response = app.send_total_images_request(get_request).await;
            log!(app, "User requested amount of images");

            return json_response!(response);
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
        .or(get_image_api)
        .or(get_total_images_api);

    let blocks = index_block
        .or(login_block)
        .or(register_block)
        .or(logout_block)
        .or(upload_block)
        .or(gallery_block);

    let routes = methods.or(blocks).or(templates).or(static_files);

    routes
}
