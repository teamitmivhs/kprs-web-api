use std::{collections::HashMap, sync::Arc};

use actix_web::{HttpRequest, HttpResponse, post, web};
use deadpool_redis::{self, Pool as RedisPool};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{data::{vote::get_votes_count, voter::get_voters_data}, db::{Campus, Vote, Voter, get_all_votes, remove_vote}, rdb::set_voters_data_redis, util::{generate_token, log_error, log_something, verify_admin_token}};

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
      let admin_token_cookie = req.cookie("admin_session_token");
      let admin_token_cookie = match admin_token_cookie {
            Some(cookie) => cookie.value().to_string(),
            None => {
                  return HttpResponse::Unauthorized().finish();
            }
      };

      let admin_data = verify_admin_token(admin_token_cookie.as_str()).await;
      match admin_data {
            Ok(data) => data,
            Err(err) => {
                  return err;
            }
      };


      // Get the voter fullname
      let reset_body_data = body.into_inner();
      let target_voter_fullname = reset_body_data.voter_fullname;


      // Verify the voter is exists
      let users_data = get_voters_data();
      let locked_users_data = users_data.read().await;
      let voter_data: Option<&Voter> = locked_users_data.get(&target_voter_fullname);
      let voter_data: &Voter = match voter_data {
            Some(data) => data,
            None => {
                  log_something("PostReset", format!("An admin just wanting to reset a user that doesn't exists: {}", target_voter_fullname).as_str());
                  return HttpResponse::NotFound().finish();
            }
      };


      // Generate new token
      let new_voter_token: String = generate_token();


      // Add the token of the voter to the Redis database
      match set_voters_data_redis(&redis_pool, target_voter_fullname.as_str(), new_voter_token.as_str(), &voter_data.campus).await {
            Ok(_) => (),
            Err(err) => {
                  return err;
            }
      }


      // Get the votes data to get who this user voting
      let db_all_votes = match get_all_votes(None).await {
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
                  let static_votes_data: Arc<HashMap<Campus, RwLock<HashMap<String, String>>>> = get_votes_count();
                  let static_votes_data: Option<&RwLock<HashMap<String, String>>> = static_votes_data.get(&voted_candidate.campus);
                  let static_votes_data: &RwLock<HashMap<String, String>> = match static_votes_data {
                        Some(data) => data,
                        None => {
                              log_error("PostVote", "The static votes count hasn't initialized yet.");
                              return HttpResponse::InternalServerError().finish();
                        }
                  };

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
