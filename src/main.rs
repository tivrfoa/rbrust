use actix_web::{http::KeepAlive, web, App, HttpServer};
use deadpool_postgres::{Config, PoolConfig, Runtime};
use std::env;
use std::{sync::Arc};
use tokio_postgres::NoTls;

mod db;
use db::*;

mod controller;
use controller::*;

mod jobs;
use jobs::*;

#[tokio::main]
async fn main() -> AsyncVoidResult {
    let mut cfg = Config::new();
    cfg.host = Some(
        env::var("DB_HOST")
            .unwrap_or("localhost".into())
            .to_string(),
    );
    cfg.port = Some(5432);
    cfg.dbname = Some("rinhadb".to_string());
    cfg.user = Some("root".to_string());
    cfg.password = Some("1234".to_string());

    let pool_size = env::var("POOL_SIZE")
        .unwrap_or("125".to_string())
        .parse::<usize>()
        .unwrap();

    cfg.pool = PoolConfig::new(pool_size).into();
    println!("creating postgres pool...");
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    println!("postgres pool succesfully created");

    tokio::spawn(async move { db_warmup().await });

    let pool_async = pool.clone();
    tokio::spawn(async move { db_clean_warmup(pool_async).await });

    let pool_async = pool.clone();
    let queue = Arc::new(AppQueue::new());
    let queue_async = queue.clone();
    tokio::spawn(async move { db_flush_queue(pool_async, queue_async).await });

    let http_port = env::var("HTTP_PORT").unwrap_or("8080".into());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(queue.clone()))
            .service(criar_pessoa)
            .service(consultar_pessoa)
            .service(buscar_pessoas)
            .service(contar_pessoas)
    })
    .keep_alive(KeepAlive::Os)
    .bind(format!("0.0.0.0:{http_port}"))?
    .run()
    .await?;

    Ok(())
}
