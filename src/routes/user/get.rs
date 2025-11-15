use actix_web::{post, HttpResponse, Responder};

#[post("/user/get")]
pub async fn post() -> impl Responder {
      HttpResponse::Ok()
}