use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::env;
use crate::models::BlockMetrics;
use bitcoin::consensus::encode::serialize;
use std::cmp::Ordering;
use serde::Deserialize;
use chrono::{DateTime, Utc};
use serde_json;

pub fn get_client() -> Client {
    let rpc_url = env::var("BITCOIN_RPC_URL").expect("BITCOIN_RPC_URL must be set in .env");
    let rpc_user = env::var("BITCOIN_RPC_USER").expect("BITCOIN_RPC_USER must be set in .env");
    let rpc_password = env::var("BITCOIN_RPC_PASSWORD").expect("BITCOIN_RPC_PASSWORD must be set in .env");

    Client::new(&rpc_url, Auth::UserPass(rpc_user, rpc_password)).unwrap()
}

#[derive(Deserialize)]
struct PriceResponse {
    bitcoin: PriceData,
}

#[derive(Deserialize)]
struct PriceData {
    usd: f64,
}

async fn fetch_market_price() -> Result<f64, reqwest::Error> {
    let response = reqwest::get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
        .await?
        .json::<PriceResponse>()
        .await?;
    
    Ok(response.bitcoin.usd)
}

async fn fetch_historical_price(timestamp: i64) -> Result<f64, reqwest::Error> {
    let datetime: DateTime<Utc> = DateTime::from_timestamp(timestamp, 0)
        .expect("Invalid timestamp provided"); 

    let date_str = datetime.format("%d-%m-%Y").to_string();
    let url = format!("https://api.coingecko.com/api/v3/coins/bitcoin/history?date={}", date_str);
    let response = reqwest::get(&url).await?.json::<serde_json::Value>().await?;

    if let Some(price) = response["market_data"]["current_price"]["usd"].as_f64() {
        Ok(price)
    } else {
        Ok(50_000.0) 
    }
}

fn calculate_difficulty(bits: u32) -> f64 {
    let max_target: f64 = (0xFFFF as f64) * 2.0f64.powi(208); 
    let target = bits_to_target(bits);
    max_target / target
}

fn bits_to_target(bits: u32) -> f64 {
    let exponent = ((bits >> 24) & 0xff) as i32;
    let mantissa = (bits & 0xffffff) as f64;
    mantissa * 2.0f64.powi(8 * (exponent - 3))
}

pub fn fetch_block_height(client: &Client) -> Result<i64, bitcoincore_rpc::Error> {
    client.get_block_count().map(|count| count as i64)
}

pub async fn fetch_block_details(client: &Client, block_height: i64) -> Result<BlockMetrics, bitcoincore_rpc::Error> {
    let block_hash = client.get_block_hash(block_height as u64)?;
    let block = client.get_block(&block_hash)?;

    let transaction_count = block.txdata.len() as i32;

    let block_timestamp = block.header.time as i64;
    let historical_price = fetch_historical_price(block_timestamp).await.unwrap_or(50_000.0);
    let current_price = fetch_market_price().await.unwrap_or(60_000.0);


    let mut output_values: Vec<f64> = Vec::new();
    let total_outputs: f64 = block.txdata
        .iter()
        .map(|tx| {
            let tx_output_value = tx.output.iter().map(|out| out.value.to_btc()).sum::<f64>();
            output_values.push(tx_output_value); 
            tx_output_value
        })
        .sum();

    let size = serialize(&block).len() as i64;
    let weight = size * 4;
    let difficulty = calculate_difficulty(block.header.bits.to_consensus());
    

    output_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let median_value = if output_values.len() % 2 == 0 {
        (output_values[output_values.len() / 2 - 1] + output_values[output_values.len() / 2]) / 2.0
    } else {
        output_values[output_values.len() / 2]
    };

    let miner = if let Some(coinbase_tx) = block.txdata.get(0) {
        if let Some(output) = coinbase_tx.output.get(0) {
            output.script_pubkey.to_string()
        } else {
            "Unknown".to_string()
        }
    } else {
        "Unknown".to_string()
    };

    Ok(BlockMetrics {
        block_height,
        block_hash: block_hash.to_string(),
        transaction_count,
        btc: total_outputs,
        value: total_outputs * historical_price,
        value_today: total_outputs * current_price,
        average_value: total_outputs / transaction_count as f64,
        median_value,
        size,
        difficulty,
        nonce: block.header.nonce as i64,
        merkle_root: block.header.merkle_root.to_string(),
        weight,
        miner,

        ..Default::default()
    })
}