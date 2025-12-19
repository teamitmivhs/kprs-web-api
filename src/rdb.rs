use std::collections::HashMap;

use actix_web::HttpResponse;
use deadpool_redis::{PoolError, Pool as RedisPool, Connection as RedisConnection};
use redis::AsyncCommands;

use crate::util::log_error;



pub async fn get_voters_data_redis(redis_pool: &RedisPool) -> Result<HashMap<String, String>, HttpResponse> {

      let redis_connection_result: Result<RedisConnection, PoolError>  = redis_pool.get().await;
      let mut redis_connection: RedisConnection = match redis_connection_result {
            Ok(connection) => connection,
            Err(err) => {
                  log_error("PostReset", format!("There's an error when trying to get redis pool. Error: {}", err.to_string()).as_str());
                  return Err(HttpResponse::InternalServerError().finish());
            }
      };

      let redis_voter_tokens: Result<HashMap<String, String>, redis::RedisError>  = redis_connection.hgetall("voter_token_reset").await;
      let redis_voter_tokens: HashMap<String, String> = match redis_voter_tokens {
            Ok(data) => data,
            Err(err) => {
                  log_error("GetToken", format!("There's an error when trying to get redis voter. Error: {}", err.to_string()).as_str());
                  return Err(HttpResponse::InternalServerError().finish());
            }
      };

      Ok(redis_voter_tokens)
}
