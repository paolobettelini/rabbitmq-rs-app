use log::{error, info};

use database::{db::Database, models::*};
use messaging::mb::*;
use protocol::{
    rabbit::{RabbitMessage::*, *},
    Parcel, Settings,
};

mod utils;

pub struct App {
    database: Database,
    amqp: Rabbit,
}

impl App {
    pub async fn new(db_connection_url: String, mb_connection_url: String) -> Self {
        let mut db_connection = Database::new(db_connection_url);

        info!("Running embedded migrations");
        db_connection.run_embedded_migrations();

        let mb_connection = Rabbit::new(mb_connection_url).await;

        App {
            database: db_connection,
            amqp: mb_connection,
        }
    }

    pub async fn start(&mut self) {
        let consumer = |data: &Delivery| self.consume(data);

        //borrow error :(
        /*self.amqp
            .consume_messages("my_queue", "my_consumer", consumer)
            .await;*/
    }

    fn consume(&mut self, msg: &Delivery) -> Option<Vec<u8>> {
        let result = RabbitMessage::from_raw_bytes(&msg.data, &Settings::default()).unwrap();

        info!("Consuming a message {:?}", result);

        let response = match result {
            LoginRequest(ref data) => self.on_login_request(&data),
            RegisterRequest(ref data) => self.on_register_request(&data),
            GetImage(ref data) => self.on_get_image(&data),
            ShrinkAndUpload(ref data) => self.on_shrink_and_upload(&data),
            GetTotalImages(ref data) => self.on_get_total_images(&data),
            _ => return None,
        };

        // Return result
        let res = response.raw_bytes(&Settings::default());

        if let Ok(res) = res {
            Some(res)
        } else {
            None
        }
    }

    fn on_login_request(&mut self, data: &LoginRequestData) -> RabbitMessage {
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

    fn on_register_request(&mut self, data: &RegisterRequestData) -> RabbitMessage {
        let exists = self.database.user_exists(&data.username);

        if exists {
            let error = RabbitMessage::RegisterResponse(RegisterResponseData::Err(
                RegisterResponseDataErr::AlreadyExists,
            ));

            return error;
        }

        /*
        if !utils::is_mail_valid(&data.mail) {
            let error = RabbitMessage::RegisterResponse(RegisterResponseData::Err(
                RegisterResponseDataErr::InvalidMail,
            ));

            return error;
        }

        if !utils::is_username_valid(&data.username) {
            let error = RabbitMessage::RegisterResponse(RegisterResponseData::Err(
                RegisterResponseDataErr::InvalidUsername,
            ));

            return error;
        }*/

        let user = NewUser {
            mail: &data.mail,
            username: &data.username,
            password: &data.password,
        };

        self.database.create_user(user);

        let token = utils::generate_random_token();

        let response =
            RabbitMessage::RegisterResponse(RegisterResponseData::Ok(RegisterResponseDataOk {
                token,
            }));

        response
    }

    fn on_get_image(&mut self, data: &GetImageData) -> RabbitMessage {
        if !self.check_authentication(&data.username, &data.token) {
            let error = RabbitMessage::ErrorResponse(ErrorResponseData::AuthenticationRequired);

            return error;
        }
        
        todo!()
    }

    fn on_shrink_and_upload(&mut self, data: &ShrinkAndUploadData) -> RabbitMessage {
        if !self.check_authentication(&data.username, &data.token) {
            let error = RabbitMessage::ErrorResponse(ErrorResponseData::AuthenticationRequired);

            return error;
        }

        todo!()
    }

    fn on_get_total_images(&mut self, data: &GetTotalImagesData) -> RabbitMessage {
        if !self.check_authentication(&data.username, &data.token) {
            let error = RabbitMessage::ErrorResponse(ErrorResponseData::AuthenticationRequired);

            return error;
        }

        let amount = self.database.get_total_images(&data.username);

        if amount == 0 {
            let exists = self.database.user_exists(&data.username);

            if !exists {
                return RabbitMessage::ErrorResponse(ErrorResponseData::UnknownUsername);
            }
        }

        let response = RabbitMessage::GetTotalImagesResponse(GetTotalImagesResponseData { amount });

        response
    }

    fn check_authentication(&mut self, username: &str, token: &Vec<u8>) -> bool {
        true

        // TODO
    }

    fn get_token_for(&mut self, username: &str) -> Vec<u8> {
        vec![]

        // TODO
    }
}
