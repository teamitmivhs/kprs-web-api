use std::collections::HashMap;

use actix_web::{HttpRequest, HttpResponse, get, web};
use deadpool_redis::Pool as RedisPool;
use strum::IntoEnumIterator;

use crate::{data::candidate::get_candidates_data, db::{Campus, Candidate}, util::{log_error, verify_admin_token, verify_voter_token}};


#[get("/candidate")]
pub async fn get(req: HttpRequest, redis_pool: web::Data<RedisPool>) -> HttpResponse {
      // Get admin or voter token from request cookie
      let admin_or_voter_token = req.cookie("admin_token").or(req.cookie("voter_token"));
      let admin_or_voter_token: String = match admin_or_voter_token {
            Some(token) => token.value().to_string(),
            None => {
                  return HttpResponse::Unauthorized().finish();
            }
      };


      // Verify admin or voter token
      let is_verified: bool = verify_admin_token(admin_or_voter_token.as_str()).await.map(|_| true)
                  .unwrap_or(verify_voter_token(admin_or_voter_token.as_str(), &redis_pool).await.map(|_| true)
                  .unwrap_or(false));

      if !is_verified {
            return HttpResponse::Unauthorized().finish();
      }


      // Get the candidate data
      let mut result_candidate_data: HashMap<Campus, Vec<Candidate>> = HashMap::new();
      let static_candidate_data = get_candidates_data().await;
      for campus in Campus::iter() {
            result_candidate_data.insert(campus, Vec::new());
      }

      for candidate_data in static_candidate_data {
            let result_candidate_data_by_campus = result_candidate_data.get_mut(&candidate_data.campus);
            let result_candidate_data_by_campus = match result_candidate_data_by_campus {
                  Some(data) => data,
                  None => {
                        log_error("GetCandidate", "All of the campus is not initialized");
                        return HttpResponse::InternalServerError().finish();
                  }
            };

            result_candidate_data_by_campus.push(candidate_data.clone());
      }


      // Return the candidates data
      HttpResponse::Ok().json(result_candidate_data)
}
