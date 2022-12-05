use messaging::mb::*;
use protocol::{
    rabbit::{RabbitMessage::*, *},
    Parcel, Settings,
};
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

        use std::{fs::read_to_string, path::Path};

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
        template!("logout", "logout.html");
        template!("upload", "upload.html");
        template!("gallery", "gallery.html");

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
                    RegisterResponseDataErr::UsernameAlreadyExists => "Username already exists",
                    RegisterResponseDataErr::MailAlreadyExists => "Mail already used",
                },
            );
        }

        self.render_template("register", context).unwrap()
    }

    pub fn render_logout(&self) -> String {
        let context = Context::new();

        self.render_template("logout", context).unwrap()
    }

    pub fn render_gallery(&self) -> String {
        let context = Context::new();

        self.render_template("gallery", context).unwrap()
    }

    pub fn render_upload(&self) -> String {
        let context = Context::new();

        self.render_template("upload", context).unwrap()
    }

    pub async fn send_login_request(&self, data: LoginRequestData) -> LoginResponseData {
        let message = RabbitMessage::LoginRequest(data);
        let payload = message.raw_bytes(&Settings::default()).unwrap();

        let answer = self
            .rabbit
            .publish_and_await_reply("queue", "consumer", &payload)
            .await;

        let error = LoginResponseData::Err(LoginResponseDataErr::NotFound);

        if let Ok(data) = answer {
            let res = RabbitMessage::from_raw_bytes(&data, &Settings::default()).unwrap();

            if let LoginResponse(data) = res {
                data
            } else {
                error
            }
        } else {
            error
        }
    }

    pub async fn send_register_request(&self, data: RegisterRequestData) -> RegisterResponseData {
        let message = RabbitMessage::RegisterRequest(data);
        let payload = message.raw_bytes(&Settings::default()).unwrap();

        let answer = self
            .rabbit
            .publish_and_await_reply("queue", "consumer", &payload)
            .await;

        let error = RegisterResponseData::Err(RegisterResponseDataErr::UsernameAlreadyExists);

        if let Ok(data) = answer {
            let res = RabbitMessage::from_raw_bytes(&data, &Settings::default()).unwrap();

            if let RegisterResponse(data) = res {
                data
            } else {
                error
            }
        } else {
            error
        }
    }

    pub async fn send_upload_request(&self, data: ShrinkAndUploadData) -> &str {
        let message = RabbitMessage::ShrinkAndUpload(data);
        let payload = message.raw_bytes(&Settings::default()).unwrap();

        let answer = self
            .rabbit
            .publish_and_await_reply("queue", "consumer", &payload)
            .await;

        if let Ok(data) = answer {
            let res = RabbitMessage::from_raw_bytes(&data, &Settings::default()).unwrap();

            if let ShrinkAndUploadResponse(data) = res {
                match data {
                    ShrinkAndUploadResponseData::Ok => "Ok",
                    ShrinkAndUploadResponseData::Err(v) => match v {
                        ShrinkAndUploadResponseDataErr::InvalidImage => "Invalid image",
                        ShrinkAndUploadResponseDataErr::AuthenticationRequired => {
                            "Authentication Required"
                        }
                    },
                }
            } else {
                "Invalid server response"
            }
        } else {
            "Invalid server response"
        }
    }

    pub async fn send_total_images_request(&self, data: GetTotalImagesData) -> String {
        let message = RabbitMessage::GetTotalImages(data);
        let payload = message.raw_bytes(&Settings::default()).unwrap();

        let answer = self
            .rabbit
            .publish_and_await_reply("queue", "consumer", &payload)
            .await;

        if let Ok(data) = answer {
            let res = RabbitMessage::from_raw_bytes(&data, &Settings::default()).unwrap();

            if let GetTotalImagesResponse(data) = res {
                match data {
                    GetTotalImagesResponseData::Ok(v) => v.amount.to_string(),
                    GetTotalImagesResponseData::Err(v) => match v {
                        GetTotalImagesResponseDataErr::AuthenticationRequired => {
                            "Authentication Required".to_owned()
                        }
                    },
                }
            } else {
                "Invalid server response".to_owned()
            }
        } else {
            "Invalid server response".to_owned()
        }
    }

    pub async fn send_get_image_request(&self, data: GetImageData) -> Vec<u8> {
        let message = RabbitMessage::GetImage(data);
        let payload = message.raw_bytes(&Settings::default()).unwrap();

        let answer = self
            .rabbit
            .publish_and_await_reply("queue", "consumer", &payload)
            .await;

        if let Ok(data) = answer {
            let res = RabbitMessage::from_raw_bytes(&data, &Settings::default()).unwrap();

            if let GetImageResponse(GetImageResponseData::Ok(image)) = res {
                image.data
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}

// global consumer name and queue name
