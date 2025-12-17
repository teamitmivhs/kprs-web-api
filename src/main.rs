use actix_web::{App, HttpServer, middleware::from_fn, web};
use actix_cors::Cors;
use deadpool_redis::{Config as RedisConfig, Runtime as RedisRuntime};
use kprs_web_api::{
    data::{admin::init_admin_data, candidate::init_candidates_data, vote::init_votes_count, voter::init_voters_data},
    db::init_db,
    middleware::middleware,
    routes::{
        admin::{admin_check_api, admin_login_api, admin_reset_api, admin_token_api, admin_votes_api},
        voter::{voter_check_api, voter_get_api, voter_logout_api, voter_vote_api},
    },
    util::log_something
};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Setup dotenv
    dotenvy::dotenv().unwrap();

    // Setup SurrealDB
    init_db().await;

    // Setup Static Data
    init_voters_data().await;
    init_votes_count().await;
    init_candidates_data().await;
    init_admin_data().await;

    // Setup Redis
    let redis_url: String = std::env::var("REDIS_URL").unwrap();

    let redis_configuration: RedisConfig = RedisConfig {
        url: Some(redis_url),
        connection: None,
        ..Default::default()
    };

    let redis_pool = redis_configuration
        .create_pool(Some(RedisRuntime::Tokio1))
        .unwrap();

    // Setup WebSocker

    // Setup HTTP Server
    log_something("Setup", "Starting...");
    HttpServer::new(move || {
        App::new()
            // State
            .app_data(web::Data::new(redis_pool.clone()))

            // Middleware
            .wrap(from_fn(middleware))
            .wrap(Cors::permissive())

            // Voter related API
            .service(voter_get_api)
            .service(voter_vote_api)
            .service(voter_logout_api)
            .service(voter_check_api)

            // Admin related API
            .service(admin_login_api)
            .service(admin_reset_api)
            .service(admin_token_api)
            .service(admin_votes_api)
            .service(admin_check_api)

            // WebSocket live connectio
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
