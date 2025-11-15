use kprs_web_api::{routes::user::{user_get_api, user_reset_api, user_vote_api}, util::log_something};

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    log_something("Setup", "Starting...");
    HttpServer::new(|| {
        App::new()
            .service(user_get_api)
            .service(user_reset_api)
            .service(user_vote_api)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
