use std::collections::HashMap;

use actix_web::{HttpRequest, HttpResponse, post, web};
use deadpool_redis::{self, Pool as RedisPool, PoolError};
use redis::{AsyncCommands, RedisError};
use serde::Deserialize;

use crate::{
    data::{candidate::get_candidates_data, vote::get_votes_count, voter::get_voters_data},
    db::insert_vote,
    util::{log_error, log_something},
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
    let redis_connection_result: Result<deadpool_redis::Connection, PoolError> =
        redis_pool.get().await;
    let mut redis_connection: deadpool_redis::Connection = match redis_connection_result {
        Ok(connection) => connection,
        Err(err) => {
            log_error(
                "PostReset",
                format!(
                    "There's an error when trying to get admin redis pool. Error: {}",
                    err.to_string()
                )
                .as_str(),
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    let redis_user_token_result: Result<HashMap<String, String>, RedisError> =
        redis_connection.hgetall("voter_token_reset").await;
    let redis_user_tokens: HashMap<String, String> = match redis_user_token_result {
        Ok(data) => data,
        Err(err) => {
            log_error("PostVote", err.to_string().as_str());
            return HttpResponse::InternalServerError().finish();
        }
    };
    let redis_user_name: Option<String> = redis_user_tokens
        .iter()
        .find(|(_, v)| v == &&cookie_user_token)
        .map(|user_data| user_data.0.clone());


    // Verify the token from checking into the redis database
    let static_voters_data = get_voters_data();
    let locked_static_voters_data = static_voters_data.read().await;
    let data_user_fullname = locked_static_voters_data
        .iter()
        .find(|data| data.1 == &cookie_user_token);
    let data_user_fullname: Option<String> = match data_user_fullname {
        Some(data) => Some(data.0.clone()),
        None => None,
    };


    // Verify the token using this step:
    // 1. Positive if the token is inside Redis
    // 2. Negative if the token is inside of the voter inside Hashmap and not inside Redis
    // 3. Negative if the token is inside the Redis
    let redis_user_token_by_data_user_name = match &data_user_fullname {
        Some(fullname) => redis_user_tokens.get(fullname),
        None => None,
    };

    if redis_user_name.is_none()
        && (redis_user_token_by_data_user_name.is_some() || data_user_fullname.is_none())
    {
        return HttpResponse::Unauthorized().finish();
    }

    let target_voter_fullname = match redis_user_name.or(data_user_fullname) {
            Some(data) => data,
            None => {
                  log_error("PostVote", "There's no voter data from either Static data and Redis data but the condition passes!");
                  return HttpResponse::Unauthorized().finish();
            }
    };

    // Get the candidate info from request data
    let request_body = body.into_inner();
    let target_candidate_fullname: String = request_body.candidate_fullname;

    // Verify candidate name
    let candidates_data = get_candidates_data().await;
    if !candidates_data.contains(&target_candidate_fullname) {
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

    // Get the static vote
    let static_votes_data = get_votes_count().await;
    let mut locked_static_votes_data = static_votes_data.write().await;

    // Verify that the haven't voted yet.
    if locked_static_votes_data.contains_key(&target_voter_fullname) {
          return HttpResponse::Conflict().finish();
    }


    // Create vote record into the SurrealDB
    let vote_record = insert_vote(
        target_voter_fullname.clone(),
        target_candidate_fullname.clone(),
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
    locked_static_votes_data.insert(target_voter_fullname, target_candidate_fullname);


    // Return OK
    HttpResponse::Ok().finish()
}
