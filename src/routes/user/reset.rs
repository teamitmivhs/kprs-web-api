use actix_web::{post, HttpResponse, Responder};

#[post("/user/reset")]
pub async fn post() -> impl Responder {
      HttpResponse::Ok()
}