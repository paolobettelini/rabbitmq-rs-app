use clap::Parser;

use log::{error, info};
use std::{env, path::Path};

mod app;
mod args;

use messaging::mb::*;
use once_cell::sync::OnceCell;
use app::*;
use std::{future::Future, sync::Arc};
use config::{Connectable, WorkerConfig};

#[macro_use]
extern crate lazy_static;

// cargo build --release
// target/release/worker -c config.toml

lazy_static! {
    pub static ref CONFIG: Box<WorkerConfig> = {
        // Read CLI arguments
        let args = args::Args::parse();

        // Read configuration path
        let config_path = Path::new(&args.config);
        let config = config::parse_config::<_, WorkerConfig>(config_path);
        let config = match config {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        };

        config
    };

    pub static ref APP: OnceCell<Arc<App>> = OnceCell::new();

    pub static ref CONSUMER: AppLogic = {
        // Read database configuration
        let db_connection_url = if let Some(db) = &CONFIG.database {
            db.get_connection_string()
        } else {
            if let Ok(var) = env::var("DATABASE_URL") {
                var
            } else {
                error!(
                    "No database connection found in configuration file. You can also set DATABASE_URL"
                );
                std::process::exit(1);
            }
        };

        let consumer = App::create_db_consumer(db_connection_url);

        consumer
    };
}

#[tokio::main]
async fn main() {
    // Setup logging system
    if let Some(log_config) = &CONFIG.log {
        let env = env_logger::Env::default()
            .filter_or("RUST_LOG", log_config.log.clone())
            .write_style_or("RUST_LOG_STYLE", log_config.style.clone());

        env_logger::init_from_env(env);
    } else {
        env_logger::init();
    };

    info!("Starting");

    
    // Create App
    let app = {
        // Read AMQP configuration
        let mb_connection_url = if let Some(mb) = &CONFIG.rabbit {
            mb.get_connection_string()
        } else {
            if let Ok(var) = env::var("AMQP_URL") {
                var
            } else {
                error!("No AMQP connection found in configuration file. You can also set DATABASE_URL");
                std::process::exit(1);
            }
        };

        Arc::new(App::new(mb_connection_url).await)
    };

    APP.set(app).unwrap();

    let delegate = move |delivery: DeliveryResult| async move {
        if let Ok(Some(delivery)) = delivery {
            // Heavy lifting
            let answer = CONSUMER.consume(&delivery);

            let properties = &delivery.properties;
            
            if let Some(reply_queue) = properties.reply_to() {
                if let Some(answer) = answer {
                    let app: &App = &APP.get().unwrap().clone();
                    
                    // Publish answer to `reply_to` queue
                    println!("Responding");
                    app.publish(
                        reply_queue.as_str(),
                        &answer
                    ).await;
                }
            }
            
            // Ack the message
            let _ = &delivery
                .ack(BasicAckOptions::default())
                .await
                .unwrap();
        }
    };

    APP.get().unwrap().clone().start(delegate).await;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(200000));
    }
}
