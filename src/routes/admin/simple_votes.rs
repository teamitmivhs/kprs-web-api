use std::{collections::HashMap, sync::Arc};

use actix_web::{HttpRequest, HttpResponse, get};
use tokio::sync::RwLock;

use crate::{data::vote::get_votes_count, db::Campus, util::verify_admin_token};


#[get("/admin/votes/simple")]
pub async fn post(req: HttpRequest) -> HttpResponse {
      // Get the admin token from request cookies
      let cookie_admin_token = req.cookie("admin_session_token");
      let cookie_admin_token = match cookie_admin_token {
          Some(data) => data.value().to_string(),
          None => {
              return HttpResponse::Unauthorized().finish();
          }
      };

      // Verify the admin token
      match verify_admin_token(cookie_admin_token.as_str()).await {
            Ok(_) => (),
            Err(response) => return response
      };

      // Get the static votes data
      let static_votes_data: Arc<HashMap<Campus, RwLock<HashMap<String, String>>>> = get_votes_count();


      // Map the result to each candidates
      let mut vote_result: HashMap<Campus, HashMap<String, usize>> = HashMap::new();
      for static_votes_data_per_campus in static_votes_data.iter() {
            let locked_static_votes_data = static_votes_data_per_campus.1.read().await;

            let mut vote_per_campus_result: HashMap<String, usize> = HashMap::new();
            for (_, candidate_name) in locked_static_votes_data.clone() {
                  vote_per_campus_result
                        .entry(candidate_name.clone())
                        .and_modify(|counter| {
                              *counter += 1;
                        })
                        .or_insert(1);
            }

            vote_result.insert(static_votes_data_per_campus.0.clone(), vote_per_campus_result);
      }


      HttpResponse::Ok()
            .json(vote_result)
}
