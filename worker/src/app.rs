use log::info;

use database::db::Database;
use image::{imageops::FilterType, EncodableLayout, ImageFormat};
use messaging::mb::*;
use once_cell::sync::OnceCell;
use protocol::{
    rabbit::{RabbitMessage::*, *},
    Parcel, Settings,
};
use std::sync::Arc;
use uuid::Uuid;
use webp::{Encoder, PixelLayout, WebPImage, WebPMemory};

mod utils;

const WIDTH: u32 = 200;
const HEIGHT: u32 = 200;

#[derive(Debug)]
pub struct App {
    amqp: Rabbit,
}

impl App {
    pub async fn new(mb_connection_url: String) -> Self {
        let amqp = Rabbit::new(mb_connection_url).await;

        App { amqp }
    }

    pub fn create_db_consumer(db_connection_url: String) -> AppLogic {
        let database = Database::new(db_connection_url);

        info!("Running embedded migrations");
        database.run_embedded_migrations();

        let consumer = AppLogic::new(database);

        consumer
    }

    pub async fn start<D: ConsumerDelegate + 'static>(&self, delegate: D) {
        self.amqp
            .consume_messages("queue", "consumer", delegate)
            .await;
    }

    pub async fn publish(&self, queue_name: &str, payload: &[u8]) {
        self.amqp.publish(queue_name, payload).await;
    }
}

pub struct AppLogic {
    database: Database,
}

impl MessageConsumer for AppLogic {
    fn consume(&self, delivery: &Delivery) -> Option<Vec<u8>> {
        info!("Received Delivery");

        // Convert bytes to `RabbitMessage` structure
        let message = {
            let settings = &Settings::default();
            let res = RabbitMessage::from_raw_bytes(&delivery.data, settings);
            if let Ok(data) = res {
                data
            } else {
                return None;
            }
        };

        let response = match message {
            LoginRequest(ref data) => self.on_login_request(&data),
            RegisterRequest(ref data) => self.on_register_request(&data),
            GetImage(ref data) => self.on_get_image(&data),
            ShrinkAndUpload(ref data) => self.on_shrink_and_upload(&data),
            GetTotalImages(ref data) => self.on_get_total_images(&data),
            Log(ref data) => {
                // No response for log
                self.on_log(&data);
                return None;
            }
            _ => return None,
        };

        // Convert `RabbitMessage` response to bytes
        let res = response.raw_bytes(&Settings::default());

        res.ok()
    }
}

impl AppLogic {
    pub fn new(database: Database) -> Self {
        AppLogic { database }
    }

    fn on_login_request(&self, data: &LoginRequestData) -> RabbitMessage {
        let user = self.database.get_user(&data.username);

        match user {
            None => LoginResponse(LoginResponseData::Err(LoginResponseDataErr::NotFound)),
            Some(user) => {
                if user.password != data.password {
                    return LoginResponse(LoginResponseData::Err(
                        LoginResponseDataErr::WrongPassword,
                    ));
                }

                let token = self.get_token_for(&data.username);

                LoginResponse(LoginResponseData::Ok(LoginResponseDataOk { token }))
            }
        }
    }

    fn on_register_request(&self, data: &RegisterRequestData) -> RabbitMessage {
        let user_exists = self.database.user_exists(&data.username);

        if user_exists {
            return RegisterResponse(RegisterResponseData::Err(
                RegisterResponseDataErr::UsernameAlreadyExists,
            ));
        }

        let mail_exists = self.database.mail_exists(&data.mail);

        if mail_exists {
            return RegisterResponse(RegisterResponseData::Err(
                RegisterResponseDataErr::MailAlreadyExists,
            ));
        }

        let token = utils::generate_random_token();

        self.database
            .create_user(&data.mail, &data.username, &data.password, &token);

        RegisterResponse(RegisterResponseData::Ok(RegisterResponseDataOk { token }))
    }

    fn on_get_image(&self, data: &GetImageData) -> RabbitMessage {
        if let Some(username) = self.get_username(&data.token) {
            if let Some(image) = self.database.get_image(&username, data.index as i32) {
                GetImageResponse(GetImageResponseData::Ok(GetImageResponseDataOk {
                    data: image.data,
                }))
            } else {
                GetImageResponse(GetImageResponseData::Err(
                    GetImageResponseDataErr::InvalidIndex,
                ))
            }
        } else {
            GetImageResponse(GetImageResponseData::Err(
                GetImageResponseDataErr::AuthenticationRequired,
            ))
        }
    }

    fn on_shrink_and_upload(&self, data: &ShrinkAndUploadData) -> RabbitMessage {
        if let Some(username) = self.get_username(&data.token) {
            let result = image::load_from_memory(&data.image);

            if let Ok(image) = result {
                // Heavy lifting ~ order of seconds
                let image = image.resize(WIDTH, HEIGHT, FilterType::Lanczos3);

                // Convert to WebP
                let encoder: Encoder = {
                    let result = Encoder::from_image(&image);
                    if let Ok(encoder) = result {
                        encoder
                    } else {
                        return ShrinkAndUploadResponse(ShrinkAndUploadResponseData::Err(
                            ShrinkAndUploadResponseDataErr::InvalidImage,
                        ));
                    }
                };
                let bytes = encoder.encode(65f32).as_bytes().to_vec();

                // Save to database
                let saved = self.database.insert_image(&username, &bytes);

                ShrinkAndUploadResponse(if !saved {
                    ShrinkAndUploadResponseData::Err(ShrinkAndUploadResponseDataErr::InvalidImage)
                } else {
                    ShrinkAndUploadResponseData::Ok
                })
            } else {
                ShrinkAndUploadResponse(ShrinkAndUploadResponseData::Err(
                    ShrinkAndUploadResponseDataErr::InvalidImage,
                ))
            }
        } else {
            ShrinkAndUploadResponse(ShrinkAndUploadResponseData::Err(
                ShrinkAndUploadResponseDataErr::AuthenticationRequired,
            ))
        }
    }

    fn on_get_total_images(&self, data: &GetTotalImagesData) -> RabbitMessage {
        if let Some(username) = self.get_username(&data.token) {
            let amount = self.database.get_total_images(&username);

            GetTotalImagesResponse(GetTotalImagesResponseData::Ok(
                GetTotalImagesResponseDataOk { amount },
            ))
        } else {
            GetTotalImagesResponse(GetTotalImagesResponseData::Err(
                GetTotalImagesResponseDataErr::AuthenticationRequired,
            ))
        }
    }

    fn on_log(&self, data: &LogData) {
        self.database.insert_log(&data.message);
    }

    fn get_username(&self, token: &Vec<u8>) -> Option<String> {
        self.database.get_username(token)
    }

    // This call assume that the user exists
    fn get_token_for(&self, username: &str) -> Vec<u8> {
        self.database.get_token_for(username).unwrap()
    }
}
