use actix_web::{App, HttpServer, middleware::from_fn, web};
use deadpool_redis::{Config as RedisConfig, Runtime as RedisRuntime};
use kprs_web_api::{
    data::{candidate::get_candidates_data, vote::get_votes_count, voter::get_voters_data}, db::init_db, middleware::middleware, routes::{
        admin::{admin_reset_api, admin_token_api, admin_votes_api},
        voter::{voter_get_api, voter_vote_api},
    }, util::log_something
};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Setup dotenv
    dotenvy::dotenv().unwrap();

    // Setup SurrealDB
    init_db().await;

    // Setup Static Data
    get_voters_data().await;
    get_votes_count().await;
    get_candidates_data().await;

    // Get the database URL from environment variable
    let redis_url: String = std::env::var("REDIS_URL").unwrap();

    // Setup Redis
    let redis_configuration: RedisConfig = RedisConfig {
        url: Some(redis_url),
        connection: None,
        ..Default::default()
    };

    let redis_pool = redis_configuration
        .create_pool(Some(RedisRuntime::Tokio1))
        .unwrap();

    // Setup HTTP Server
    log_something("Setup", "Starting...");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis_pool.clone()))
            .wrap(from_fn(middleware))
            .service(voter_get_api)
            .service(voter_vote_api)
            .service(admin_reset_api)
            .service(admin_token_api)
            .service(admin_votes_api)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
