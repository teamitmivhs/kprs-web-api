use actix_web::{HttpResponse, cookie::Cookie, post};
use time::Duration;

#[post("/voter/logout")]
pub async fn post() -> HttpResponse {
      let clear_cookie = Cookie::build("voter_token", "")
            .path("/")
            .secure(true)
            .http_only(true)
            .max_age(Duration::seconds(0))
            .finish();

      HttpResponse::Ok().cookie(clear_cookie).finish()
}
