mod db;
mod api;
mod bitcoin;
mod ingestion;

use dotenv::dotenv;
use std::env;
use rocket::tokio;
use rocket::Rocket;
use rocket::Build;
use rocket_cors::{CorsOptions, AllowedOrigins};

async fn build_rocket() -> Rocket<Build> {
    dotenv().ok();

    // Initialize the database connection and create tables if not exist
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let db_pool = db::init_db(&database_url).await.expect("Failed to initialize database");

    // Start the background data ingestion task
    tokio::spawn(ingestion::start_ingestion(db_pool.clone()));

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .expect("Failed to create CORS fairing");


    // Return the Rocket instance to launch the server
    api::start_server(db_pool)
        .attach(cors)
}

#[rocket::launch]
async fn rocket() -> _ {
    build_rocket().await
}
