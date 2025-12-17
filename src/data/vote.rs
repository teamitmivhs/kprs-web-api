use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use crate::{db::{Vote, get_all_votes}, util::{log_error, log_something}};

pub static VOTES_COUNT: Lazy<Arc<RwLock<HashMap<String, String>>>> = Lazy::new(|| {
      Arc::new(RwLock::new(HashMap::new()))
});

pub async fn update_votes_data() {
      // Get the votes data
      let db_all_votes = get_all_votes().await;
      let db_all_votes: Vec<Vote> = match db_all_votes {
            Ok(data) => data,
            Err(err) => {
                  log_error("StaticData", format!("There's an error when trying to get all votes from postgres. Error: {}", err.to_string()).as_str());
                  return;
            }
      };

      // Map all of the votes data
      let mut locked_write_static_votes_data = VOTES_COUNT.write().await;
      locked_write_static_votes_data.clear();
      for vote_data in db_all_votes {
            locked_write_static_votes_data.insert(vote_data.voter_name, vote_data.candidate_name);
      }

      // Log the success message
      log_something("StaticData", "Static votes data successfully initialized.");
}


pub fn get_votes_count() -> Arc<RwLock<HashMap<String, String>>> {
      VOTES_COUNT.clone()
}

pub async fn init_votes_count() {
      update_votes_data().await;
}
