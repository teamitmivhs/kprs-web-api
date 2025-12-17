use actix_web::{HttpRequest, HttpResponse, post};

use crate::util::verify_admin_token;

#[post("/admin/check")]
pub async fn post(req: HttpRequest) -> HttpResponse {
      // Get the admin token from request cookies
      let cookie_admin_token = req.cookie("admin_session_token");
      let cookie_admin_token = match cookie_admin_token {
          Some(data) => data.value().to_string(),
          None => {
              return HttpResponse::Unauthorized().finish();
          }
      };

      // Check the admin token
      match verify_admin_token(cookie_admin_token).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(response) => response
      }
}
