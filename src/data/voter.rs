use dashmap::DashMap;
use tokio::sync::OnceCell;

use crate::{db::get_all_users, util::{log_error, log_something}};


pub static USERS_DATA: OnceCell<DashMap<String, String>> = OnceCell::const_new();

pub async fn get_voters_data<'a>() -> &'a DashMap<String, String> {
      let data = USERS_DATA.get_or_init(async || {
            // Get the user data
            let db_all_users = get_all_users().await;
            let db_all_users = match db_all_users {
                  Ok(data) => data,
                  Err(err) => {
                        log_error("StaticData", format!("There's an error when trying to get all voters from postgres. Error: {}", err.to_string()).as_str());
                        return DashMap::new();
                  }
            };


            // Create a variable that can hold the data
            let users_data: DashMap<String, String> = DashMap::new();

            // Iterate each users in database
            for db_user in db_all_users {
                  users_data.insert(db_user.name, db_user.token);
            }


            // Log the success message
            log_something("StaticData", "Static users data successfully initialized.");

            // Return the result
            users_data
      }).await;

      return &data;
}
