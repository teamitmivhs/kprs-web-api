use std::{collections::HashMap, sync::Arc};

use actix_web::{HttpRequest, HttpResponse, get};
use tokio::sync::RwLock;

use crate::{data::vote::get_votes_count, db::Campus, util::{log_error, verify_admin_token}};


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
      let mut result: HashMap<Campus, HashMap<String, String>> = HashMap::new();

      let all_static_votes_data: Arc<HashMap<Campus, RwLock<HashMap<String, String>>>> = get_votes_count();

      for votes_data in all_static_votes_data.iter() {
            let static_votes_data: Option<&RwLock<HashMap<String, String>>> = all_static_votes_data.get(votes_data.0);
            let static_votes_data = match static_votes_data {
                  Some(data) => data,
                  None => {
                        log_error("PostVote", "The static votes count hasn't initialized yet.");
                        return HttpResponse::InternalServerError().finish();
                  }
            };
            let locked_static_votes_data = static_votes_data.read().await;

            result.insert(votes_data.0.clone(), locked_static_votes_data.clone());
      }



      HttpResponse::Ok()
            .json(result)
}
