use tokio::time::{sleep, Duration};
use sqlx::Pool;
use sqlx::Postgres;
use crate::bitcoin;
use reqwest;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;
use bitcoincore_rpc::RpcApi;
// use std::collections::HashSet;
// use bitcoincore_rpc::bitcoin::address::Address;
// use bitcoin::network::constants::Network; 
// use bitcoin::Network;

#[derive(Deserialize)]
struct PriceResponse {
    bitcoin: PriceData,
}

#[derive(Deserialize)]
struct PriceData {
    usd: f64,
}

fn get_bitcoin_data_dir() -> PathBuf {
    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());

    #[cfg(target_os = "linux")]
    let data_dir = format!("{}/.bitcoin", home_dir);

    #[cfg(target_os = "macos")]
    let data_dir = format!("{}/Library/Application Support/Bitcoin", home_dir);

    #[cfg(target_os = "windows")]
    let data_dir = env::var("APPDATA").map(|appdata| format!("{}/Bitcoin", appdata)).unwrap_or_else(|_| ".".to_string());

    PathBuf::from(data_dir)
}

async fn fetch_market_price() -> Result<f64, reqwest::Error> {
    let response = reqwest::get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
        .await?
        .json::<PriceResponse>()
        .await?;
    
    Ok(response.bitcoin.usd)
}

fn fetch_total_sent_today(client: &bitcoincore_rpc::Client) -> f64 {
    let mut total_sent = 0.0;

    let current_height = match client.get_block_count() {
        Ok(height) => height,
        Err(_) => return 0.0, 
    };

    let blocks_per_day = 144; 
    let start_height = if current_height > blocks_per_day {
        current_height - blocks_per_day
    } else {
        0
    };

    // Iterate over the blocks in the last 24 hours
    for height in start_height..=current_height {
        if let Ok(block_hash) = client.get_block_hash(height) {
            if let Ok(block) = client.get_block(&block_hash) {
                for transaction in block.txdata {
                    let tx_total: f64 = transaction.output.iter()
                        .map(|vout| vout.value.to_btc()) 
                        .sum();
                    total_sent += tx_total;
                }
            }
        }
    }

    total_sent
}


fn fetch_network_hashrate(client: &bitcoincore_rpc::Client) -> f64 {
    match client.get_network_hash_ps(None, None) {
        Ok(hashrate) => hashrate as f64,
        Err(_) => 0.0,
    }
}

fn fetch_blockchain_size() -> f64 {
    let blocks_path = get_bitcoin_data_dir().join("blocks");
    
    match std::fs::metadata(blocks_path) {
        Ok(metadata) => metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0),
        Err(_) => 0.0,
    }
}


// fn fetch_unique_addresses_24hr(client: &bitcoincore_rpc::Client) -> i32 {
//     let mut unique_addresses: HashSet<String> = HashSet::new();

//     let current_height = match client.get_block_count() {
//         Ok(height) => height,
//         Err(_) => return 0,
//     };

//     let blocks_per_day = 144; // Approximate number of blocks per day (6 blocks per hour)
//     let start_height = if current_height > blocks_per_day {
//         current_height - blocks_per_day
//     } else {
//         0
//     };

//     // Set the network manually (e.g., mainnet, testnet)
//     let network = Network::Bitcoin; // You can change this to Network::Testnet if needed

//     for height in start_height..=current_height {
//         if let Ok(block_hash) = client.get_block_hash(height) {
//             if let Ok(block) = client.get_block(&block_hash) {
//                 for transaction in block.txdata {
//                     for output in transaction.output {
//                         // Attempt to extract address from the script_pubkey
//                         if let Some(address) = Address::from_script(&output.script_pubkey, network) {
//                             unique_addresses.insert(address.to_string());
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     unique_addresses.len() as i32
// }

pub async fn start_ingestion(pool: Pool<Postgres>) {
    let client = bitcoin::get_client();

    loop {
        if let Ok(block_height) = bitcoin::fetch_block_height(&client) {
            println!("Fetched block height: {}", block_height);
            if let Err(e) = crate::db::insert_block_height(&pool, block_height).await {
                eprintln!("Failed to insert block height: {:?}", e);
            }

            if let Ok((block_hash, transaction_count)) = bitcoin::fetch_block_details(&client, block_height) {
                println!("Fetched block details: Hash = {}, Transactions = {}", block_hash, transaction_count);

                if let Ok(market_price) = fetch_market_price().await {
                    println!("Fetched market price: {}", market_price);
                    let total_sent_today = fetch_total_sent_today(&client);
                    println!("Total sent today: {}", total_sent_today);
                    let network_hashrate = fetch_network_hashrate(&client);
                    println!("Network hashrate: {}", network_hashrate);
                    let blockchain_size = fetch_blockchain_size();
                    println!("Blockchain size: {}", blockchain_size);
                    // let unique_addresses_24hr = fetch_unique_addresses_24hr(&client);

                    if let Err(e) = crate::db::insert_metrics(
                        &pool, 
                        block_height, 
                        &block_hash, 
                        transaction_count, 
                        market_price, 
                        total_sent_today, 
                        network_hashrate, 
                        blockchain_size, 
                        // unique_addresses_24hr
                    ).await {
                        eprintln!("Failed to insert metrics: {:?}", e);
                    }
                } else {
                    eprintln!("Failed to fetch market price");
                }
            } else {
                eprintln!("Failed to fetch block details for height: {}", block_height);
            }
        } else {
            eprintln!("Failed to fetch block height");
        }

        sleep(Duration::from_secs(30)).await;
    }
}
