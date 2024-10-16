use rocket::{get, routes, State};
use sqlx::{Pool, Postgres, Row};
use rocket::serde::json::Json;
use crate::models::BlockMetrics;

#[get("/latest_block_metrics")]
async fn latest_block_metrics(pool: &State<Pool<Postgres>>) -> Option<Json<BlockMetrics>> {
    let result = sqlx::query(
        "SELECT * FROM metrics ORDER BY block_height DESC LIMIT 1"
    )
    .fetch_one(pool.inner())
    .await;

    if let Ok(row) = result {
        Some(Json(BlockMetrics {
            block_height: row.get("block_height"),
            block_hash: row.get("block_hash"),
            btc: row.get("btc"),
            value: row.get("value"),
            value_today: row.get("market_price"),
            average_value: row.get("average_value"),
            median_value: row.get("median_value"),
            transaction_count: row.get("transaction_count"),
            size: row.get("size"),
            merkle_root: row.get("merkle_root"),
            difficulty: row.get("difficulty"),
            nonce: row.get("nonce"),
            weight: row.get("weight"),
            miner: row.get("miner"),
            network_hashrate: row.get("network_hashrate"),
            total_sent_today: row.get("total_sent_today"),
            blockchain_size: row.get("blockchain_size"),
        }))
    } else {
        None
    }
}


#[get("/latest_15_blocks")]
async fn latest_15_blocks(pool: &State<Pool<Postgres>>) -> Json<Vec<i64>> {
    let result = sqlx::query(
        "SELECT height FROM block_height ORDER BY height DESC LIMIT 15"
    )
    .fetch_all(pool.inner())
    .await;

    let mut blocks: Vec<i64> = Vec::new();
    if let Ok(rows) = result {
        for row in rows {
            blocks.push(row.get("height"));
        }
    }

    Json(blocks)
}

#[get("/block/<block_height>")]
async fn block_by_height(pool: &State<Pool<Postgres>>, block_height: i64) -> Option<Json<BlockMetrics>> {
    let result = sqlx::query(
        "SELECT * FROM metrics WHERE block_height = $1"
    )
    .bind(block_height)
    .fetch_one(pool.inner())
    .await;

    if let Ok(row) = result {
        Some(Json(BlockMetrics {
            block_height: row.get("block_height"),
            block_hash: row.get("block_hash"),
            btc: row.get("btc"),
            value: row.get("value"),
            value_today: row.get("market_price"),
            average_value: row.get("average_value"),
            median_value: row.get("median_value"),
            transaction_count: row.get("transaction_count"),
            size: row.get("size"),
            merkle_root: row.get("merkle_root"),
            difficulty: row.get("difficulty"),
            nonce: row.get("nonce"),
            weight: row.get("weight"),
            miner: row.get("miner"),
            network_hashrate: row.get("network_hashrate"),
            total_sent_today: row.get("total_sent_today"),
            blockchain_size: row.get("blockchain_size"),
        }))
    } else {
        None
    }
}

pub fn start_server(pool: Pool<Postgres>) -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .manage(pool)
        .mount("/", routes![latest_block_metrics])
        .mount("/", routes![latest_15_blocks])
        .mount("/", routes![block_by_height])
}