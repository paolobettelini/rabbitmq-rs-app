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