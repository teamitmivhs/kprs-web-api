use std::{collections::HashMap, sync::Arc};

use actix_web::{HttpRequest, HttpResponse, get};
use tokio::sync::RwLock;

use crate::{data::vote::get_votes_count, util::verify_admin_token};


#[get("/admin/votes")]
pub async fn get(req: HttpRequest) -> HttpResponse {
      // Get the admin token from request cookies
      let cookie_admin_token = req.cookie("admin_session_token");
      let cookie_admin_token = match cookie_admin_token {
          Some(data) => data.value().to_string(),
          None => {
              return HttpResponse::Unauthorized().finish();
          }
      };

      // Verify the admin token
      match verify_admin_token(cookie_admin_token).await {
            Ok(_) => (),
            Err(response) => return response
      };

      // Get the static votes data
      let static_votes_data: Arc<RwLock<HashMap<String, String>>> = get_votes_count();
      let locked_static_votes_data = static_votes_data.read().await;


      HttpResponse::Ok()
            .json(locked_static_votes_data.clone())
}
