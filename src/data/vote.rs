use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use strum::IntoEnumIterator;
use tokio::sync::RwLock;
use crate::{db::{Campus, Vote, get_all_votes}, util::{log_error, log_something}};

pub static VOTES_COUNT: Lazy<Arc<HashMap<Campus, RwLock<HashMap<String, String>>>>> = Lazy::new(|| {
      let mut hashmap_result: HashMap<Campus, RwLock<HashMap<String, String>>> = HashMap::new();

      for campus in Campus::iter() {
            hashmap_result.insert(campus, RwLock::new(HashMap::new()));
      }

      Arc::new(hashmap_result)
});

pub async fn update_votes_data() {
      // Get the votes data
      let mut db_all_votes_data: HashMap<Campus, Vec<Vote>> = HashMap::new();

      // Iterate for each campus
      for campus in Campus::iter() {
            // Get all of the data of that particular campus
            let result = get_all_votes(Some(campus)).await;
            let result: Vec<Vote> = match result {
                  Ok(data) => data,
                  Err(err) => {
                        log_error("StaticData", format!("There's an error when trying to get all votes from postgres. Error: {}", err.to_string()).as_str());
                        return;
                  }
            };

            // Add the result for that particular campus as the key
            db_all_votes_data.insert(campus, result);
      }


      // Map all of the votes data for each campus
      for all_votes_data in db_all_votes_data {
            // get the static data for current campus
            let static_votes_data = VOTES_COUNT.get(&all_votes_data.0);
            let static_votes_data = match static_votes_data {
                  Some(data) => data,
                  None => {
                        log_error("StaticData", "There's an error where campus enum as key is not aggregated yet.");
                        return;
                  }
            };

            // Get the locked write state of static votes data
            let mut locked_write_static_votes_data = static_votes_data.write().await;
            locked_write_static_votes_data.clear();

            // Iterate for each voter data in current campus
            for votes_data_per_campus in all_votes_data.1.iter() {
                  locked_write_static_votes_data.insert(votes_data_per_campus.voter_name.clone(), votes_data_per_campus.candidate_name.clone());
            }
      }

      // Log the success message
      log_something("StaticData", "Static votes data successfully initialized.");
}


pub fn get_votes_count() -> Arc<HashMap<Campus, RwLock<HashMap<String, String>>>> {
      VOTES_COUNT.clone()
}

pub async fn init_votes_count() {
      update_votes_data().await;
}
