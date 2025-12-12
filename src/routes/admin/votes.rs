use std::collections::HashMap;

use actix_web::{HttpRequest, HttpResponse, get};
use serde::Serialize;
use tokio::sync::RwLock;

use crate::{data::vote::get_votes_count, util::log_error};

#[derive(Serialize)]
struct GetBodyRequestType {
      votes_data: HashMap<String, String>
}

#[get("/admin/votes")]
pub async fn get(req: HttpRequest) -> HttpResponse {
      // Verify the admin token from cookies
      let admin_token_cookie = req.cookie("admin_token");
      let admin_token_cookie = match admin_token_cookie {
            Some(cookie) => cookie.value().to_string(),
            None => {
                  return HttpResponse::NotFound().finish();
            }
      };

      let valid_admin_token = std::env::var("ADMIN_TOKEN");
      let valid_admin_token = match valid_admin_token {
            Ok(data) => data,
            Err(err) => {
                  log_error("PostReset", format!("There's an error when trying to get admin token from ENV. Error: {}", err.to_string()).as_str());
                  return HttpResponse::InternalServerError().finish();
            }
      };


      if admin_token_cookie != valid_admin_token {
            return HttpResponse::Unauthorized().finish();
      }


      // Get the static votes data
      let static_votes_data: &RwLock<HashMap<String, String>> = get_votes_count().await;
      let locked_static_votes_data = static_votes_data.read().await;


      HttpResponse::Ok()
            .json(GetBodyRequestType {
                  votes_data: locked_static_votes_data.clone()
            })
}
