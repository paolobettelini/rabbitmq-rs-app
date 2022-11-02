use tera::{Context, Tera};

pub struct App {
    tera: Tera,
}

impl App {
    pub fn new(www: &str) -> Self {
        let mut tera = Tera::default();

        use std::{fs::read_to_string, error::Error, path::Path};
    
        let www = Path::new(www);
    
        macro_rules! template {
            ($name: tt, $file: tt) => {
                {
                    let path = www.join($file);
                    let content = read_to_string(path).expect("Template file not found");
                    tera.add_raw_template($name, &content).expect("Template not valid");
                }
            };
        }
    
        template!("index", "index.html");
        //template!("login", "login.html");
        
        App { tera }
    }

    fn render_template(&self, name: &str, context: Context) -> Option<String> {
        /*

        */

        let res = self.tera.render(name, &context);

        if let Ok(res) = res {
            Some(res)
        } else {
            None
        }
    }

    pub fn render_index(&self) -> String {
        let mut context = Context::new();
        context.insert("name", "Rust");

        self.render_template("index", context).unwrap()
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