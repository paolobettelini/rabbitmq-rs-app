use clap::Parser;

use log::{error, info};
use std::{env, path::Path};

mod app;
mod args;

use app::*;
use config::{Connectable, WorkerConfig};

// cargo build --release
// target/release/worker -c config.toml

#[tokio::main]
async fn main() {
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

    // Setup logging system
    if let Some(log_config) = config.log {
        let env = env_logger::Env::default()
            .filter_or("RUST_LOG", log_config.log)
            .write_style_or("RUST_LOG_STYLE", log_config.style);

        env_logger::init_from_env(env);
    } else {
        env_logger::init();
    };

    // Read database configuration
    let db_connection_url = if let Some(db) = config.database {
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

    info!("Starting");

    // Read AMQP configuration
    let mb_connection_url = if let Some(mb) = config.rabbit {
        mb.get_connection_string()
    } else {
        if let Ok(var) = env::var("AMQP_URL") {
            var
        } else {
            error!("No AMQP connection found in configuration file. You can also set DATABASE_URL");
            std::process::exit(1);
        }
    };

    // Create App
    let app = App::new(db_connection_url, mb_connection_url).await;
    app.start().await;
}
