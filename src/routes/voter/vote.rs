use std::{collections::HashMap, sync::Arc};

use actix_web::{HttpRequest, HttpResponse, post, web};
use deadpool_redis::{self, Pool as RedisPool};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::{
    data::{candidate::get_candidates_data, vote::get_votes_count},
    db::{Campus, Voter, insert_vote},
    util::{log_error, log_something, verify_voter_token},
};

#[derive(Deserialize)]
struct VoteBodyRequest {
    candidate_fullname: String,
}

#[post("/voter/vote")]
pub async fn post(
    body: web::Json<VoteBodyRequest>,
    req: HttpRequest,
    redis_pool: web::Data<RedisPool>,
) -> HttpResponse {
    // Get the user token from request cookies
    let cookie_user_token = req.cookie("voter_token");
    let cookie_user_token = match cookie_user_token {
        Some(data) => data.value().to_string(),
        None => {
            return HttpResponse::Unauthorized().finish();
        }
    };


    // Verify the token from checking into the Redis database
    let target_voter_data: Voter = match verify_voter_token(cookie_user_token.as_str(), &redis_pool).await {
          Ok(voter) => voter,
          Err(response) => {
                return response;
          }
    };

    // Get the candidate info from request data
    let request_body = body.into_inner();
    let target_candidate_fullname: String = request_body.candidate_fullname;
    let target_voter_fullname: &String = &target_voter_data.name;

    // Verify candidate name
    let candidates_data = get_candidates_data().await;
    let target_candidate_data = candidates_data.iter().find(|data| data.name == target_candidate_fullname);
    let target_candidate_data = match target_candidate_data {
          Some(data) => data,
          None => {
            log_something(
                  "PostVote",
                  format!(
                  "{} has votes {} that is currently not registered",
                  target_voter_fullname, target_candidate_fullname
                  )
                  .as_str(),
            );
            return HttpResponse::BadRequest().finish();
          }
    };

    if target_candidate_data.campus != target_voter_data.campus {
          return HttpResponse::Unauthorized().finish();
    }

    // Get the static vote
    let static_votes_data: Arc<HashMap<Campus, RwLock<HashMap<String, String>>>> = get_votes_count();
    let static_votes_data: Option<&RwLock<HashMap<String, String>>> = static_votes_data.get(&target_candidate_data.campus);
    let static_votes_data: &RwLock<HashMap<String, String>> = match static_votes_data {
          Some(data) => data,
          None => {
                log_error("PostVote", "The static votes count hasn't initialized yet.");
                return HttpResponse::InternalServerError().finish();
          }
    };

    let mut locked_static_votes_data = static_votes_data.write().await;

    // Verify that the haven't voted yet.
    if locked_static_votes_data.contains_key(target_voter_fullname) {
          return HttpResponse::Conflict().finish();
    }


    // Create vote record into the SurrealDB
    let vote_record = insert_vote(
        target_voter_fullname.clone(),
        target_candidate_fullname.clone(),
        target_candidate_data.campus
    )
    .await;

    match vote_record {
        Ok(_) => {
            log_something(
                "PostVote",
                format!(
                    "{} has successfully votes {}",
                    target_voter_fullname, target_candidate_fullname
                )
                .as_str(),
            );
        }
        Err(err) => {
            log_error("PostVote", format!("There's an error when trying to update vote record into the database. Error: {}", err.to_string()).as_str());
            return HttpResponse::InternalServerError().finish();
        }
    }


    // Put the vote data inside the static data
    locked_static_votes_data.insert(target_voter_fullname.clone(), target_candidate_fullname);


    // Return OK
    HttpResponse::Ok().finish()
}
