use sqlx::{migrate::MigrateDatabase, Error, Pool, Postgres};

pub async fn init_db(database_url: &str) -> Result<Pool<Postgres>, Error> {
     if !Postgres::database_exists(database_url).await.unwrap_or(false) {
        Postgres::create_database(database_url).await?;
    } else {
        println!("Database already exists");
    }
    let pool = Pool::<Postgres>::connect(database_url).await?;

    create_tables(&pool).await?;

    Ok(pool)
}

async fn create_tables(pool: &Pool<Postgres>) -> Result<(), Error> {
    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS block_height (
            id SERIAL PRIMARY KEY,
            height INTEGER NOT NULL,
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
            block_height INTEGER NOT NULL,
            block_hash TEXT,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            transaction_count INTEGER,
            market_price FLOAT8,
            total_sent_today FLOAT8,
            network_hashrate FLOAT8,
            blockchain_size FLOAT8
        );
        "#
    )
    .execute(pool)
    .await {
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
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO metrics (block_height, block_hash, transaction_count, market_price, total_sent_today, network_hashrate, blockchain_size) VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(block_height)
    .bind(block_hash)
    .bind(transaction_count)
    .bind(market_price)
    .bind(total_sent_today)
    .bind(network_hashrate)
    .bind(blockchain_size)
    .execute(pool)
    .await?;
    Ok(())
}