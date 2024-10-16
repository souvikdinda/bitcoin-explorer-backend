use serde::Serialize;

#[derive(Serialize, Default)] 
pub struct BlockMetrics {
    pub block_height: i64,
    pub block_hash: String,
    pub btc: f64,                  // Total Bitcoin outputs
    pub value: f64,                // Historical value
    pub value_today: f64,          // Current Bitcoin value
    pub average_value: f64,        // Average transaction value
    pub median_value: f64,         // Median transaction value
    pub transaction_count: i32,    // Number of transactions
    pub size: i64,                 // Block size
    pub weight: i64,               // Block weight
    pub difficulty: f64,           // Block difficulty
    pub merkle_root: String,       // Merkle root of block
    pub nonce: i64,                // Block nonce
    pub miner: String,             // Miner who mined the block
    pub network_hashrate: f64,     // Network hashrate
    pub total_sent_today: f64,     // Total Bitcoin sent today
    pub blockchain_size: f64,      // Blockchain size
}
