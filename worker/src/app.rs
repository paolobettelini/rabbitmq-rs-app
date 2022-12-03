use log::{info};

use database::db::Database;
use image::{imageops::FilterType, ImageFormat};
use std::sync::Arc;
use messaging::mb::*;
use protocol::{
    rabbit::{RabbitMessage::*, *},
    Parcel, Settings,
};


mod utils;

const WIDTH: u32 = 200;
const HEIGHT: u32 = 200;
const FORMAT: ImageFormat = ImageFormat::Png;

pub struct App {
    amqp: Arc<Rabbit>,
    database: Arc<Database>,
}

impl App {
    pub async fn new(db_connection_url: String, mb_connection_url: String) -> Self {
        let amqp = Arc::new(Rabbit::new(mb_connection_url).await);
        let database = Arc::new(Database::new(db_connection_url));

        App {
            amqp,
            database,
        }
    }

    pub async fn start(&self) {
        info!("Starting consumer");
        
        let mut futs = vec![];

        for i in 0..num_cpus::get() {
            let amqp = self.amqp.clone();
            let database = self.database.clone();

            futs.push(async move {
                println!("Running thread {}", i);
                
                let consumer = AppLogic::new(database.clone());

                amqp
                    .consume_messages("queue", "consumer", consumer)
                    .await
                    .unwrap();
            });
        }

        futures::future::join_all(futs).await;
    }
}

#[derive(Clone)]
struct AppLogic {
    database: Arc<Database>,
}

impl MessageConsumer for AppLogic {
    fn consume(&self, delivery: &Delivery) -> Option<Vec<u8>> {
        let result = RabbitMessage::from_raw_bytes(&delivery.data, &Settings::default()).unwrap();

        info!("Received Delivery");

        let response = match result {
            LoginRequest(ref data) => self.on_login_request(&data),
            RegisterRequest(ref data) => self.on_register_request(&data),
            GetImage(ref data) => self.on_get_image(&data),
            ShrinkAndUpload(ref data) => self.on_shrink_and_upload(&data),
            GetTotalImages(ref data) => self.on_get_total_images(&data),
            _ => return None,
        };

        // Return result
        // println!("Returning {:?}", response);
        let res = response.raw_bytes(&Settings::default());

        if let Ok(res) = res {
            Some(res)
        } else {
            None
        }
    }
}

impl AppLogic {
    pub fn new(database: Arc<Database>) -> Self {
        info!("Running embedded migrations");
        database.run_embedded_migrations();

        AppLogic { database }
    }

    fn on_login_request(&self, data: &LoginRequestData) -> RabbitMessage {
        let user = self.database.get_user(&data.username);

        match user {
            None => {
                let error = RabbitMessage::LoginResponse(LoginResponseData::Err(
                    LoginResponseDataErr::NotFound,
                ));

                error
            }
            Some(user) => {
                if user.password != data.password {
                    let error = RabbitMessage::LoginResponse(LoginResponseData::Err(
                        LoginResponseDataErr::WrongPassword,
                    ));

                    return error;
                }

                let token = self.get_token_for(&data.username);

                let response =
                    RabbitMessage::LoginResponse(LoginResponseData::Ok(LoginResponseDataOk {
                        token,
                    }));

                response
            }
        }
    }

    fn on_register_request(&self, data: &RegisterRequestData) -> RabbitMessage {
        let user_exists = self.database.user_exists(&data.username);

        if user_exists {
            let error = RabbitMessage::RegisterResponse(RegisterResponseData::Err(
                RegisterResponseDataErr::UsernameAlreadyExists,
            ));

            return error;
        }

        let mail_exists = self.database.mail_exists(&data.mail);

        if mail_exists {
            let error = RabbitMessage::RegisterResponse(RegisterResponseData::Err(
                RegisterResponseDataErr::MailAlreadyExists,
            ));

            return error;
        }

        let token = utils::generate_random_token();

        self.database
            .create_user(&data.mail, &data.username, &data.password, &token);

        let response =
            RabbitMessage::RegisterResponse(RegisterResponseData::Ok(RegisterResponseDataOk {
                token,
            }));

        info!("Register Ok");
        response
    }

    fn on_get_image(&self, data: &GetImageData) -> RabbitMessage {
        if let Some(username) = self.get_username(&data.token) {
            if let Some(image) = self.database.get_image(&username, data.index as i32) {
                RabbitMessage::GetImageResponse(GetImageResponseData::Ok(GetImageResponseDataOk {
                    data: image.data,
                }))
            } else {
                RabbitMessage::GetImageResponse(GetImageResponseData::InvalidIndex)
            }
        } else {
            RabbitMessage::ErrorResponse(ErrorResponseData::AuthenticationRequired)
        }
    }

    fn on_shrink_and_upload(&self, data: &ShrinkAndUploadData) -> RabbitMessage {
        if let Some(username) = self.get_username(&data.token) {
            let result = image::load_from_memory(&data.image);

            if let Ok(image) = result {
                // Heavy lifting ~ order of seconds
                let image = image.resize(WIDTH, HEIGHT, FilterType::Lanczos3);

                // Convert to WebP
                let bytes = utils::image_to_format(image, FORMAT);

                // Save to database
                self.database.insert_image(&username, &bytes);

                RabbitMessage::ShrinkAndUploadResponse(ShrinkAndUploadResponseData::Ok)
            } else {
                return RabbitMessage::ShrinkAndUploadResponse(
                    ShrinkAndUploadResponseData::InvalidImage,
                );
            }
        } else {
            let error = RabbitMessage::ErrorResponse(ErrorResponseData::AuthenticationRequired);

            error
        }
    }

    fn on_get_total_images(&self, data: &GetTotalImagesData) -> RabbitMessage {
        if let Some(username) = self.get_username(&data.token) {
            let amount = self.database.get_total_images(&username);

            if amount == 0 {
                let exists = self.database.user_exists(&username);

                if !exists {
                    return RabbitMessage::ErrorResponse(ErrorResponseData::UnknownUsername);
                }
            }

            let response =
                RabbitMessage::GetTotalImagesResponse(GetTotalImagesResponseData { amount });

            response
        } else {
            let error = RabbitMessage::ErrorResponse(ErrorResponseData::AuthenticationRequired);

            error
        }
    }

    fn get_username(&self, token: &Vec<u8>) -> Option<String> {
        self.database.get_username(token)
    }

    // This call assume that the user exists
    fn get_token_for(&self, username: &str) -> Vec<u8> {
        self.database.get_token_for(username).unwrap()
    }
}
