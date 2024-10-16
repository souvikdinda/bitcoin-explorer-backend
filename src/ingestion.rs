use tokio::time::{sleep, Duration};
use sqlx::Pool;
use sqlx::Postgres;
use crate::bitcoin;
use std::env;
use std::path::PathBuf;
use bitcoincore_rpc::RpcApi;


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

pub async fn start_ingestion(pool: Pool<Postgres>) {
    let client = bitcoin::get_client();

    loop {
        if let Ok(block_height) = bitcoin::fetch_block_height(&client) {
            if let Err(e) = crate::db::insert_block_height(&pool, block_height).await {
                eprintln!("Failed to insert block height: {:?}", e);
            }

            if let Ok(block_data) = bitcoin::fetch_block_details(&client, block_height).await {
                let block_hash = &block_data.block_hash;
                let transaction_count = block_data.transaction_count;
                let size = block_data.size;
                let weight = block_data.weight;
                let difficulty = block_data.difficulty;
                let merkle_root = &block_data.merkle_root;
                let nonce = block_data.nonce;
                
                let total_sent_today = fetch_total_sent_today(&client);
                let network_hashrate = fetch_network_hashrate(&client);
                let blockchain_size = fetch_blockchain_size();
                
                let miner = &block_data.miner;
                let btc = block_data.btc;
                let value = block_data.value;
                let average_value = block_data.average_value;
                let median_value = block_data.median_value;

                if let Err(e) = crate::db::insert_metrics(
                    &pool, 
                    block_height, 
                    block_hash, 
                    transaction_count, 
                    block_data.value_today, 
                    total_sent_today, 
                    network_hashrate, 
                    blockchain_size, 
                    size, 
                    weight, 
                    difficulty, 
                    merkle_root, 
                    nonce, 
                    miner,
                    btc,  
                    value, 
                    average_value, 
                    median_value
                ).await {
                    eprintln!("Failed to insert metrics: {:?}", e);
                }
            } else {
                eprintln!("Failed to fetch block details for height: {}", block_height);
            }
        } else {
            eprintln!("Failed to fetch block height");
        }

        sleep(Duration::from_secs(300)).await;
    }
}
