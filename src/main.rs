//! PostgreSQL Library Benchmark - Utility Runner
//!
//! This binary provides utilities for setting up and testing the benchmark environment.

use anyhow::Result;
use pg_benchmark::DATABASE_URL;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("PostgreSQL Library Benchmark Suite");
    println!("===================================");
    println!();
    println!("Libraries being benchmarked:");
    println!("  - tokio-postgres (async low-level driver)");
    println!("  - sqlx (compile-time verified SQL)");
    println!("  - sea-orm (async ORM)");
    println!("  - diesel (sync ORM)");
    println!("  - clorinde (generated type-safe queries)");
    println!();
    println!("Database URL: {}", DATABASE_URL);
    println!();
    println!("To run benchmarks:");
    println!("  cargo bench");
    println!();
    println!("To run specific benchmark groups:");
    println!("  cargo bench -- insert");
    println!("  cargo bench -- select");
    println!("  cargo bench -- join");
    println!("  cargo bench -- heavy");
    println!();
    
    // Test database connectivity
    println!("Testing database connection...");
    match test_connection().await {
        Ok(_) => println!("Database connection successful!"),
        Err(e) => println!("Database connection failed: {}", e),
    }
    
    Ok(())
}

async fn test_connection() -> Result<()> {
    let (client, connection) = tokio_postgres::connect(DATABASE_URL, tokio_postgres::NoTls).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    
    let row = client.query_one("SELECT COUNT(*) as count FROM users", &[]).await?;
    let count: i64 = row.get("count");
    println!("  Users in database: {}", count);
    
    let row = client.query_one("SELECT COUNT(*) as count FROM posts", &[]).await?;
    let count: i64 = row.get("count");
    println!("  Posts in database: {}", count);
    
    Ok(())
}
