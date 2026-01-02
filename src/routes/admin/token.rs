use std::{collections::HashMap, sync::Arc};
use actix_web::{HttpRequest, HttpResponse, get, web::{self, Json}};
use deadpool_redis::{Pool as RedisPool};
use strum::IntoEnumIterator;
use tokio::sync::RwLock;

use crate::{data::voter::get_voters_data, db::{Campus, Voter}, rdb::{RedisVoterType, get_voters_data_redis}, util::{log_error, verify_admin_token}};


#[get("/admin/token")]
pub async fn get(req: HttpRequest, redis_pool: web::Data<RedisPool>) -> HttpResponse {
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
      }


      // Get the token data from Redis
      let redis_voter_tokens: Result<HashMap<String, RedisVoterType>, HttpResponse>  = get_voters_data_redis(&redis_pool).await;
      let redis_voter_tokens: HashMap<String, RedisVoterType> = match redis_voter_tokens {
            Ok(data) => data,
            Err(err) => {
                  return err;
            }
      };


      // Get the token data from static
      let static_voter_tokens: Arc<RwLock<HashMap<String, Voter>>> = get_voters_data();
      let locked_static_voter_tokens = static_voter_tokens.read().await.clone();


      // Map all of the result into a single variable
      let mut result_voters_token: HashMap<Campus, HashMap<String, String>> = HashMap::new();

      for campus_name in Campus::iter() {
            result_voters_token.insert(campus_name, HashMap::new());
      }

      for static_voter_token in locked_static_voter_tokens.iter() {
            let result_voters_token_per_campus = result_voters_token.get_mut(&static_voter_token.1.campus);
            let result_voters_token_per_campus = match result_voters_token_per_campus {
                  Some(data) => data,
                  None => {
                        log_error("GetToken", "There's a voter data in static data where the campus is not in the enum!");
                        continue;
                  }
            };
            result_voters_token_per_campus.insert(static_voter_token.0.clone(), static_voter_token.1.token.clone());
      }

      for dynamic_voter_token in redis_voter_tokens.iter() {
            let result_voters_token_by_campus = result_voters_token.get_mut(&dynamic_voter_token.1.campus);
            let result_voters_token_by_campus = match result_voters_token_by_campus {
                  Some(data) => data,
                  None => {
                        log_error("GetToken", "There's a voter data in redis where the campus is not in the enum!");
                        continue;
                  }
            };

            result_voters_token_by_campus.entry(dynamic_voter_token.0.clone())
                .and_modify(|result_voter_token| {
                      *result_voter_token = dynamic_voter_token.1.token.clone();
                });
      }


      // Return the token data
      let mut response = HttpResponse::Ok();

      response.json(Json(result_voters_token))
}
