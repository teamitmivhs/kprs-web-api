use std::collections::HashMap;

use actix_web::HttpResponse;
use deadpool_redis::{Pool as RedisPool, PoolError};
use redis::{AsyncCommands, RedisError};
use time::{OffsetDateTime, macros::{format_description, offset}};
use rand::Rng;

use crate::{data::{admin::get_all_admin_data, voter::get_voters_data}, db::{Admin, Voter}};

static DATETIME_FMT: &[time::format_description::FormatItem<'static>] = format_description!("[hour]:[minute]:[second]");

pub fn get_time() -> String {
      let utc = OffsetDateTime::now_utc();
      let result_time = utc.to_offset(offset!(+7)).format(DATETIME_FMT);

      match result_time {
            Ok(data) => data.to_string(),
            Err(err) => {
                  log_error("Util", format!("There's an error when get the current time. Error: {}", err.to_string()).as_str());
                  String::from("--:--:--")
            }
      }
}

pub fn log_something(scope_title: &str, message: &str) {
      println!("[{}] [{}] {}", get_time(), scope_title, message);
}

pub fn log_error(scope_title: &str, message: &str) {
      println!("[{}] [ERROR] [{}] {}", get_time(), scope_title, message);
}

static TOKEN_LENGTH: usize = 5;

pub fn generate_token() -> String {
      let mut result: String = String::new();
      let mut rng = rand::rng();

      // Iterate for each characters
      // for i in 65..(65+52) {
      //       if i > 25 { i += 6; }

      // }
      for i in 0..=(TOKEN_LENGTH) {
            let mut random_index = rng.random_range(65..(65+52));
            if random_index > 90 { random_index += 6; }

            if let Some(data) = char::from_u32(random_index) {
                  result.insert(i, data);
            }
            else {
                  result.insert(i, 'A');
            }
      }

      result
}

pub async fn verify_voter_token(cookie_user_token: &str, redis_pool: &RedisPool) -> Result<Voter, HttpResponse> {
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
              return Err(HttpResponse::InternalServerError().finish());
          }
      };

      let redis_user_token_result: Result<HashMap<String, String>, RedisError> =
          redis_connection.hgetall("voter_token_reset").await;
      let redis_user_tokens: HashMap<String, String> = match redis_user_token_result {
          Ok(data) => data,
          Err(err) => {
              log_error("PostVote", err.to_string().as_str());
              return Err(HttpResponse::InternalServerError().finish());
          }
      };
      let redis_user_name_by_cookie_token: Option<String> = redis_user_tokens
          .iter()
          .find(|(_, v)| v == &&cookie_user_token)
          .map(|user_data| user_data.0.clone());


      // Verify the token from checking into the redis database
      let static_voters_data = get_voters_data();
      let locked_static_voters_data = static_voters_data.read().await;
      let static_voter_data_maybe = locked_static_voters_data
          .iter()
          .find(|data| data.1.token == cookie_user_token);
      let static_voter_name: Option<String> = match static_voter_data_maybe {
          Some(data) => Some(data.0.clone()),
          None => None,
      };


      // Verify the token using this step:
      // 1. Positive if the token is inside Redis
      // 2. Negative if the token is inside of the voter inside Hashmap and not inside Redis
      // 3. Negative if the token is inside the Redis
      let redis_user_token_by_data_user_name = match &static_voter_name {
          Some(fullname) => redis_user_tokens.get(fullname),
          None => None,
      };

      if redis_user_name_by_cookie_token.is_none()
          && (redis_user_token_by_data_user_name.is_some() || static_voter_name.is_none())
      {
          return Err(HttpResponse::Unauthorized().finish());
      }

      let target_voter_data: Option<&Voter> = match redis_user_name_by_cookie_token.or(static_voter_name) {
              Some(data) => locked_static_voters_data.get(&data),
              None => {
                    log_error("PostVote", "There's no voter data from either Static data and Redis data but the condition passes!");
                    return Err(HttpResponse::Unauthorized().finish());
              }
      };
      let target_voter_data: &Voter = match target_voter_data {
            Some(data) => data,
            None => {
                  log_error("PostVote", "There's no voter data in static variables but it seems that the data is exists in Redis!");
                  return Err(HttpResponse::InternalServerError().finish());
            }
      };

      Ok(target_voter_data.clone())
}

pub async fn verify_admin_token(admin_token: impl Into<String>) -> Result<Admin, HttpResponse> {
      // Get the static admin token
      let admin_token = Some(admin_token.into());
      let static_admin_data = get_all_admin_data();
      let locked_static_admin_data = static_admin_data.read().await;
      let admin_data: Option<&Admin> = locked_static_admin_data.iter().find(|data| data.1.admin_session_token == admin_token).map(|data| data.1);

      match admin_data {
            Some(data) => Ok(data.clone()),
            None => Err(HttpResponse::InternalServerError().finish())
      }
}
