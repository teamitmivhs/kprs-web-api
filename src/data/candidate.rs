use tokio::sync::OnceCell;

use crate::{db::get_all_candidates, util::{log_error, log_something}};


pub static CANDIDATES_DATA: OnceCell<Vec<String>> = OnceCell::const_new();

pub async fn get_candidates_data<'a>() -> &'a Vec<String> {
      let data = CANDIDATES_DATA.get_or_init(async || {
            // Get the candidate data
            let db_all_candidates = get_all_candidates().await;
            let db_all_candidates = match db_all_candidates {
                  Ok(data) => data,
                  Err(err) => {
                        log_error("StaticData", format!("There's an error when trying to get static data from database. Error: {}", err.to_string()).as_str());
                        return Vec::new();
                  }
            };


            // Create a variable that can hold the data
            let mut candidates_data: Vec<String> = Vec::new();

            // Iterate each candidate in database
            for db_candidate in db_all_candidates {
                  candidates_data.push(db_candidate.name);
            }


            // Log the success message
            log_something("StaticData", "Static candidates data successfully initialized.");

            // Return the result
            candidates_data
      }).await;

      return &data;
}
