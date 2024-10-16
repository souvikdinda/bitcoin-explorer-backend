use sqlx::{migrate::MigrateDatabase, Error, Pool, Postgres};
use tokio::time::{sleep, Duration};

pub async fn init_db(database_url: &str) -> Result<Pool<Postgres>, Error> {
    let mut attempts = 0;
    let max_attempts = 10;

    let pool = loop {
        match Pool::<Postgres>::connect(database_url).await {
            Ok(pool) => {
                println!("Connected to the database.");
                break pool;
            }
            Err(e) => {
                eprintln!("Failed to connect to database: {:?}", e);
                attempts += 1;
                if attempts >= max_attempts {
                    eprintln!("Reached maximum attempts to connect to the database.");
                    return Err(Error::Configuration("Unable to connect to the database after several attempts".into()));
                }
                let delay = Duration::from_secs(2_u64.pow(attempts));
                println!("Retrying in {:?} seconds...", delay.as_secs());
                sleep(delay).await;
            }
        }
    };

    if !Postgres::database_exists(database_url).await.unwrap_or(false) {
        match Postgres::create_database(database_url).await {
            Ok(_) => println!("Database created successfully."),
            Err(e) => eprintln!("Failed to create database: {:?}", e),
        }
    } else {
        println!("Database already exists.");
    }

    create_tables(&pool).await?;

    Ok(pool)
}

async fn create_tables(pool: &Pool<Postgres>) -> Result<(), Error> {
    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS block_height (
            id SERIAL PRIMARY KEY,
            height BIGINT NOT NULL,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        "#
    )
    .execute(pool)
    .await {
        Ok(_) => println!("block_height table created or already exists."),
        Err(e) => eprintln!("Failed to create block_height table: {:?}", e),
    };

    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS metrics (
            id SERIAL PRIMARY KEY,
            block_height BIGINT NOT NULL,
            block_hash TEXT,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            transaction_count INTEGER,
            market_price FLOAT8,
            total_sent_today FLOAT8,
            network_hashrate FLOAT8,
            blockchain_size FLOAT8,
            size BIGINT,
            weight BIGINT,
            difficulty FLOAT8,
            merkle_root TEXT,
            nonce BIGINT,
            miner TEXT,
            btc FLOAT8,               
            value FLOAT8,             
            average_value FLOAT8,     
            median_value FLOAT8
        );
        "#
    )
    .execute(pool)
    .await
    {
        Ok(_) => println!("metrics table created or already exists."),
        Err(e) => eprintln!("Failed to create metrics table: {:?}", e),
    };

    Ok(())
}

pub async fn insert_block_height(pool: &Pool<Postgres>, height: i64) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO block_height (height) VALUES ($1)"
    )
    .bind(height)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_metrics(
    pool: &Pool<Postgres>,
    block_height: i64,
    block_hash: &str,
    transaction_count: i32,
    market_price: f64,
    total_sent_today: f64,
    network_hashrate: f64,
    blockchain_size: f64,
    size: i64,
    weight: i64,
    difficulty: f64,
    merkle_root: &str,
    nonce: i64,
    miner: &str,
    btc: f64,               
    value: f64,             
    average_value: f64,     
    median_value: f64,     
) -> Result<(), Error> {
    sqlx::query(
        r#"
        INSERT INTO metrics 
            (block_height, block_hash, transaction_count, market_price, total_sent_today, network_hashrate, 
            blockchain_size, size, weight, difficulty, merkle_root, nonce, miner, 
            btc, value, average_value, median_value) 
        VALUES 
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        "#
    )
    .bind(block_height)
    .bind(block_hash)
    .bind(transaction_count)
    .bind(market_price)
    .bind(total_sent_today)
    .bind(network_hashrate)
    .bind(blockchain_size)
    .bind(size)
    .bind(weight)
    .bind(difficulty)
    .bind(merkle_root)
    .bind(nonce)
    .bind(miner)
    .bind(btc)                
    .bind(value)              
    .bind(average_value)      
    .bind(median_value)  
    .execute(pool)
    .await?;
    Ok(())
}
