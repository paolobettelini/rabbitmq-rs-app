use tera::{Context, Tera};
use warp::{Filter, Rejection, Reply};

pub struct App {
    tera: Tera,
}

impl App {
    pub fn new() -> Self {
        let tera = Tera::default();

        App { tera }
    }

    pub async fn start(&mut self, www: &'static str) {
        self.init_templates(www);
        Self::init_routes(www).await;
    }

    fn init_templates(&mut self, www: &str) {
        use std::{fs::read_to_string, error::Error, path::Path};

        let www = Path::new(www);

        macro_rules! template {
            ($name: tt, $file: tt) => {
                {
                    let path = www.join($file);
                    let content = read_to_string(path).expect("Template file not found");
                    self.tera.add_raw_template($name, &content).expect("Template not valid");
                }
            };
        }

        // todo...
        //template!("index", "index.html");
        //template!("login", "login.html");
    }

    fn render_template(&mut self, name: &str, context: Context) -> Option<String> {
        /*
            let mut context = Context::new();
            context.insert("name", "Rust");
        */

        let res = self.tera.render(name, &context);

        if let Ok(res) = res {
            Some(res)
        } else {
            None
        }
    }

    async fn init_routes(www: &'static str) {
        let routes = Self::get_routes(www);
        warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
    }

    fn get_routes(www: &'static str) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let static_files = warp::path("static").and(warp::fs::dir(www));
        //let static_files = warp::path("static").and(warp::fs::dir("/home/paolo/www"));
        
        let index_route = warp::path::end().map(|| "index override");
        let login_route = warp::path("login.html").map(|| "login override");
    
        let routes = index_route
            .or(static_files)
            .or(login_route);

        routes
    }
}

/*
use messaging::mb::*;
use protocol::{
    Settings,
    Parcel,
    rabbit::*
};

pub trait WebserverRabbit {
    fn send_login_request(&self, data: LoginRequestData);

    fn send_register_request(&self, data: RegisterRequestData);
}

impl WebserverRabbit for Rabbit {

    fn send_login_request(&self, data: LoginRequestData) {
        let message = RabbitMessage::LoginRequest(data);
        let payload = message.raw_bytes(&Settings::default()).unwrap();
        
        // self.publish("my_queue", &payload).await;
    }

    fn send_register_request(&self, data: RegisterRequestData) {
        let message = RabbitMessage::RegisterRequest(data);
        let payload = message.raw_bytes(&Settings::default()).unwrap();
    }

    // todo

}

*/