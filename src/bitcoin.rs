use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::env;

pub fn get_client() -> Client {
    let rpc_url = env::var("BITCOIN_RPC_URL").expect("BITCOIN_RPC_URL must be set in .env");
    let rpc_user = env::var("BITCOIN_RPC_USER").expect("BITCOIN_RPC_USER must be set in .env");
    let rpc_password = env::var("BITCOIN_RPC_PASSWORD").expect("BITCOIN_RPC_PASSWORD must be set in .env");

    Client::new(&rpc_url, Auth::UserPass(rpc_user, rpc_password)).unwrap()
}

pub fn fetch_block_height(client: &Client) -> Result<i64, bitcoincore_rpc::Error> {
    client.get_block_count().map(|count| count as i64)
}

pub fn fetch_block_details(client: &Client, block_height: i64) -> Result<(String, i32), bitcoincore_rpc::Error> {
    let block_hash = client.get_block_hash(block_height as u64)?;
    let block = client.get_block(&block_hash)?;
    let transaction_count = block.txdata.len() as i32;

    Ok((block_hash.to_string(), transaction_count))
}
