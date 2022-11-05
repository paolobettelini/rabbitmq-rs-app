use messaging::mb::*;
use protocol::{rabbit::*, Parcel, Settings};
use tera::{Context, Tera};

#[derive(Debug)]
pub struct App {
    tera: Tera,
    rabbit: Rabbit,
}

impl App {
    pub async fn new(www: &str, amqp: &str) -> Self {
        let tera = Self::init_tera(www);
        let rabbit = Self::init_rabbit(amqp).await;

        App { tera, rabbit }
    }

    fn init_tera(www: &str) -> Tera {
        let mut tera = Tera::default();

        use std::{error::Error, fs::read_to_string, path::Path};

        let www = Path::new(www);

        macro_rules! template {
            ($name: tt, $file: tt) => {{
                let path = www.join($file);
                let content = read_to_string(path).expect("Template file not found");
                tera.add_raw_template($name, &content)
                    .expect("Template not valid");
            }};
        }

        template!("index", "index.html");
        template!("login", "login.html");
        template!("register", "register.html");

        tera
    }

    async fn init_rabbit(amqp: &str) -> Rabbit {
        Rabbit::new(amqp.to_owned()).await
    }

    fn render_template(&self, name: &str, context: Context) -> Option<String> {
        let res = self.tera.render(name, &context);

        if let Ok(res) = res {
            Some(res)
        } else {
            None
        }
    }

    pub fn render_index(&self, token: Option<String>) -> String {
        let mut context = Context::new();
        if let Some(value) = token {
            context.insert("token", &value);
        }

        self.render_template("index", context).unwrap()
    }

    pub fn render_login(&self, status: Option<LoginResponseDataErr>) -> String {
        let mut context = Context::new();

        if let Some(err) = status {
            context.insert(
                "error",
                &match err {
                    LoginResponseDataErr::NotFound => "Username not found",
                    LoginResponseDataErr::WrongPassword => "Password is incorrect",
                },
            );
        }

        self.render_template("login", context).unwrap()
    }

    pub fn render_register(&self, status: Option<RegisterResponseDataErr>) -> String {
        let mut context = Context::new();

        if let Some(err) = status {
            context.insert(
                "error",
                &match err {
                    RegisterResponseDataErr::AlreadyExists => "User already exists",
                },
            );
        }

        self.render_template("register", context).unwrap()
    }

    pub async fn send_login_request(&self, data: LoginRequestData) -> LoginResponseData {
        let message = RabbitMessage::LoginRequest(data);
        let payload = message.raw_bytes(&Settings::default()).unwrap();

        // self.rabbit.publish("my_queue", &payload).await;

        LoginResponseData::Err(LoginResponseDataErr::NotFound)
        //LoginResponseData::Ok(LoginResponseDataOk { token: vec![5,5,5,5] })
    }

    pub async fn send_register_request(&self, data: RegisterRequestData) -> RegisterResponseData {
        let message = RabbitMessage::RegisterRequest(data);
        let payload = message.raw_bytes(&Settings::default()).unwrap();

        RegisterResponseData::Ok(RegisterResponseDataOk {
            token: vec![5, 5, 5, 5, 5, 5],
        })
    }
}
