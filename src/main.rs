mod db;
mod api;
mod bitcoin;
mod ingestion;
mod models;

use dotenv::dotenv;
use std::env;
use rocket::tokio;
use rocket::Rocket;
use rocket::Build;
use rocket_cors::{CorsOptions, AllowedOrigins, AllowedHeaders};
use rocket::figment::Figment;

async fn build_rocket() -> Rocket<Build> {
    // Load environment variables from .env
    dotenv().ok();

    // Initialize the database connection and create tables if not exist
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let db_pool = db::init_db(&database_url)
        .await
        .expect("Failed to initialize database");

    // Start the background data ingestion task
    tokio::spawn(ingestion::start_ingestion(db_pool.clone()));

    // Set up CORS (Cross-Origin Resource Sharing) options
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_headers(AllowedHeaders::all())
        .to_cors()
        .expect("Failed to create CORS fairing");

    // Configure Rocket server (address and port from environment or default values)
    let config = Figment::from(rocket::Config::default())
        .merge(("address", "0.0.0.0"))
        .merge(("port", 8000));

    // Return the Rocket instance to launch the server
    api::start_server(db_pool)
        .attach(cors)
        .configure(config)
}

#[rocket::launch]
async fn rocket() -> _ {
    build_rocket().await
}