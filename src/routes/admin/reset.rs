use std::collections::HashMap;

use actix_web::{HttpRequest, HttpResponse, post, web};
use deadpool_redis::{self, Pool as RedisPool, PoolError};
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{data::{vote::get_votes_count, voter::get_voters_data}, db::{Vote, get_all_votes, remove_vote}, util::{generate_token, log_error, log_something}};

#[derive(Deserialize)]
struct ResetBodyRequestType {
      voter_fullname: String
}

#[derive(Serialize)]
struct ResetBodyResponseType {
      new_token: String
}


#[post("/admin/reset")]
pub async fn post(body: web::Json<ResetBodyRequestType>, req: HttpRequest, redis_pool: web::Data<RedisPool>) -> HttpResponse {
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


      // Get the voter fullname
      let reset_body_data = body.into_inner();
      let target_voter_fullname = reset_body_data.voter_fullname;


      // Verify the voter is exists
      let users_data = get_voters_data();
      let locked_users_data = users_data.read().await;
      if !locked_users_data.contains_key(&target_voter_fullname) {
            log_something("PostReset", format!("An admin just wanting to reset a user that doesn't exists: {}", target_voter_fullname).as_str());
            return HttpResponse::NotFound().finish();
      }


      // Generate new token
      let new_voter_token: String = generate_token();


      // Add the token of the voter to the Redis database
      let redis_connection_result: Result<deadpool_redis::Connection, PoolError>  = redis_pool.get().await;
      let mut redis_connection: deadpool_redis::Connection = match redis_connection_result {
            Ok(connection) => connection,
            Err(err) => {
                  log_error("PostReset", format!("There's an error when trying to get admin redis pool. Error: {}", err.to_string()).as_str());
                  return HttpResponse::InternalServerError().finish();
            }
      };


      let insert_result: Result<(), RedisError> = redis_connection.hset("voter_token_reset", target_voter_fullname.clone(), new_voter_token.clone()).await;
      match insert_result {
            Ok(_) => (),
            Err(err) => {
                  log_error("PostReset", format!("There's an error when trying to reset a voter token to Redis. Error: {}", err.to_string()).as_str());
                  return HttpResponse::InternalServerError().finish();
            }
      }


      // Get the votes data to get who this user voting
      let db_all_votes = match get_all_votes().await {
            Ok(data) => data,
            Err(err) => {
                  log_error("PostReset", format!("There's an error when getting all votes. Error: {}", err.to_string()).as_str());
                  return HttpResponse::InternalServerError().finish();
            }
      };
      let possible_voted_candidate: Option<&Vote> = db_all_votes.iter().find(|data| data.voter_name == target_voter_fullname);


      // Reset the vote from database
      let remove_vote_result = remove_vote(target_voter_fullname.clone()).await;
      match remove_vote_result {
            Ok(_) => {
                  log_something("PostReset", format!("Successfully remove a vote from {}", target_voter_fullname).as_str());
            },
            Err(err) => {
                  log_error("PostReset", format!("Failed remove a vote from {}. Error: {}", target_voter_fullname, err.to_string()).as_str());
                  return HttpResponse::InternalServerError().finish();
            }
      }


      // Reset the vote from static(?) data
      match possible_voted_candidate {
            Some(voted_candidate) => {
                  let static_votes_data: &RwLock<HashMap<String, String>> = get_votes_count().await;
                  let mut locked_static_votes_data = static_votes_data.write().await;
                  locked_static_votes_data.remove(&voted_candidate.voter_name);
            },
            None => ()
      }


      // Sends OK! with the data!
      HttpResponse::Ok()
            .content_type("application/json")
            .json(ResetBodyResponseType {
                  new_token: new_voter_token
            })
}
