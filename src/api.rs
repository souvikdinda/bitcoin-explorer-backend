use rocket::{get, routes, State};
use sqlx::{Pool, Postgres, Row}; // Import Row trait
use serde::Serialize;
use rocket::serde::json::Json;

#[derive(Serialize)]
struct BlockMetrics {
    block_height: i32,
    block_hash: String,
    transaction_count: i32,
    market_price: f64,
    total_sent_today: f64,
    network_hashrate: f64,
    blockchain_size: f64,
    // unique_addresses_24hr: i32,
}

#[get("/block_metrics")]
async fn get_block_metrics(pool: &State<Pool<Postgres>>) -> Option<Json<BlockMetrics>> {
    let result = sqlx::query(
        "SELECT block_height, block_hash, transaction_count, market_price, total_sent_today, network_hashrate, blockchain_size FROM metrics ORDER BY id DESC LIMIT 1"
    )
    .fetch_one(pool.inner())
    .await;

    match result {
        Ok(record) => {
            let block_height: i32 = match record.try_get("block_height") {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Error extracting block_height: {:?}", e);
                    return None;
                }
            };

            let block_hash: String = match record.try_get("block_hash") {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Error extracting block_hash: {:?}", e);
                    return None;
                }
            };

            let transaction_count: i32 = match record.try_get("transaction_count") {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Error extracting transaction_count: {:?}", e);
                    return None;
                }
            };

            let market_price: f64 = match record.try_get("market_price") {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Error extracting market_price: {:?}", e);
                    return None;
                }
            };

            let total_sent_today: f64 = match record.try_get("total_sent_today") {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Error extracting total_sent_today: {:?}", e);
                    return None;
                }
            };

            let network_hashrate: f64 = match record.try_get("network_hashrate") {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Error extracting network_hashrate: {:?}", e);
                    return None;
                }
            };

            let blockchain_size: f64 = match record.try_get("blockchain_size") {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Error extracting blockchain_size: {:?}", e);
                    return None;
                }
            };

            // let unique_addresses_24hr: i32 = match record.try_get("unique_addresses_24hr") {
            //     Ok(value) => value,
            //     Err(e) => {
            //         eprintln!("Error extracting unique_addresses_24hr: {:?}", e);
            //         return None;
            //     }
            // };

            Some(Json(BlockMetrics {
                block_height,
                block_hash,
                transaction_count,
                market_price,
                total_sent_today,
                network_hashrate,
                blockchain_size,
                // unique_addresses_24hr,
            }))
        },
        Err(e) => {
            eprintln!("Error fetching block metrics: {:?}", e);
            None
        },
    }
}

pub fn start_server(pool: Pool<Postgres>) -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .manage(pool)
        .mount("/", routes![get_block_metrics])
}