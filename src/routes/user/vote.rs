use actix_web::{HttpResponse, Responder, post};

#[post("/user/vote")]
pub async fn post() -> impl Responder {
      HttpResponse::Ok()
}