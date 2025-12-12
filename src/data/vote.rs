use std::collections::HashMap;

use tokio::sync::{OnceCell, RwLock};
use crate::{db::{Vote, get_all_votes}, util::{log_error, log_something}};

pub static VOTES_COUNT: OnceCell<RwLock<HashMap<String, String>>> = OnceCell::const_new();

pub async fn get_votes_count<'a>() -> &'a RwLock<HashMap<String, String>> {
      let result: &RwLock<HashMap<String, String>> = VOTES_COUNT.get_or_init(async || {
            // Get the votes data
            let db_all_votes = get_all_votes().await;
            let db_all_votes: Vec<Vote> = match db_all_votes {
                  Ok(data) => data,
                  Err(err) => {
                        log_error("StaticData", format!("There's an error when trying to get all votes from postgres. Error: {}", err.to_string()).as_str());
                        return RwLock::new(HashMap::new());
                  }
            };

            // Map all of the votes data
            let mut mapped_votes_data: HashMap<String, String> = HashMap::new();
            for vote_data in db_all_votes {
                  mapped_votes_data.insert(vote_data.voter_name, vote_data.candidate_name);
            }

            // Log the success message
            log_something("StaticData", "Static votes data successfully initialized.");

            // Return the result
            RwLock::new(mapped_votes_data)
      }).await;

      return result;
}

pub async fn init_votes_count() {
      get_votes_count().await;
}
