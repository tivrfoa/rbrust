use actix_web::{http::KeepAlive, web, App, HttpServer};
use deadpool_postgres::{Config, PoolConfig, Runtime};
use std::env;
use tokio_postgres::NoTls;

mod db;
use db::*;

mod controller;
use controller::*;

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

    let http_port = env::var("HTTP_PORT").unwrap_or("8080".into());
    println!("Running at port: {http_port}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
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
