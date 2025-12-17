use actix_web::{HttpResponse, cookie::Cookie, post, web};
use serde::Deserialize;
use rand::{Rng, distr::Alphanumeric};
use time::Duration;

use crate::{data::admin::get_all_admin_data, db::set_admin_session_token, util::log_error};

pub static TOKEN_LEN:usize = 50;

#[derive(Deserialize)]
struct AdminLoginData {
      admin_id: String,
      admin_password: String
}


#[post("/admin/login")]
pub async fn post(data: web::Json<AdminLoginData>) -> HttpResponse {
      // Get Admin ID and Admin Password
      let data = data.into_inner();

      // Get static admin data
      let static_admin_data = get_all_admin_data();
      {
            // Read static admin data
            let locked_static_admin_data = static_admin_data.read().await;
            let target_admin_data = locked_static_admin_data.get(&data.admin_id);

            // Check if the Admin ID exists in the static admin data
            let target_admin_data = match target_admin_data {
                  Some(data) => data,
                  None => {
                        return HttpResponse::Unauthorized().finish();
                  }
            };

            // Check if the the Admin Password correct
            if target_admin_data.admin_password != data.admin_password {
                  return HttpResponse::Unauthorized().finish();
            }
      }

      // Create admin cookie
      let mut rng = rand::rng();
      let admin_session_token: String = (0..TOKEN_LEN)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect::<String>();

      {
            // Update static admin data
            let mut write_locked_static_admin_data = static_admin_data.write().await;
            write_locked_static_admin_data.entry(data.admin_id.clone()).and_modify(|data| {
                  data.admin_session_token = Some(admin_session_token.clone().to_string());
            });
      }

      // Update from database
      match set_admin_session_token(data.admin_id.as_str(), admin_session_token.as_str()).await {
            Ok(_) => (),
            Err(err) => {
                  log_error("AdminLogin", format!("There's an error when trying to set admin session token. Error: {}", err.to_string()).as_str());
            }
      }

      // Create admin session token cookie
      let admin_session_token_cookie = Cookie::build("admin_session_token", admin_session_token.as_str())
            .path("/")
            .secure(true)
            .http_only(true)
            .max_age(Duration::days(2))
            .finish();

      HttpResponse::Ok().cookie(admin_session_token_cookie).finish()
}
