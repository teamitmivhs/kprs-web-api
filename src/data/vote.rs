use std::sync::atomic::{AtomicUsize, Ordering};

use dashmap::DashMap;
use tokio::sync::OnceCell;
use crate::{db::{Vote, get_all_candidates, get_all_votes}, util::{log_error, log_something}};

pub static VOTES_COUNT: OnceCell<DashMap<String, AtomicUsize>> = OnceCell::const_new();

pub async fn get_votes_count<'a>() -> &'a DashMap<String, AtomicUsize> {
      let result: &DashMap<String, AtomicUsize> = VOTES_COUNT.get_or_init(async || {
            // Get the votes data
            let db_all_votes = get_all_votes().await;
            let db_all_votes: Vec<Vote> = match db_all_votes {
                  Ok(data) => data,
                  Err(err) => {
                        log_error("StaticData", format!("There's an error when trying to get all votes from postgres. Error: {}", err.to_string()).as_str());
                        return DashMap::new();
                  }
            };

            // Get the candidates data
            let db_all_candidates = get_all_candidates().await;
            let db_all_candidates = match db_all_candidates {
                  Ok(data) => data,
                  Err(err) => {
                        log_error("StaticData", format!("There's an error when trying to get all candidates from postgres. Error: {}", err.to_string()).as_str());
                        return DashMap::new();
                  }
            };

            // Create a variable that can hold the data
            let votes_count: DashMap<String, AtomicUsize> = DashMap::new();

            for db_candidate in db_all_candidates {
                  votes_count.insert(db_candidate.name, AtomicUsize::new(0));
            }


            // Iterate each votes in database
            for db_vote in db_all_votes {
                  if !votes_count.contains_key(&db_vote.candidate_name) {
                        log_error("GetStaticVotes", "There's an error where the candidate from the vote data is not exists on the candidates database");
                        panic!();
                  }

                  votes_count.entry(db_vote.candidate_name)
                        .and_modify(|data| {
                              data.fetch_add(1, Ordering::Relaxed);
                        });
            }


            // Log the success message
            log_something("StaticData", "Static votes data successfully initialized.");


            // Return the result
            votes_count
      }).await;

      return result;
}
