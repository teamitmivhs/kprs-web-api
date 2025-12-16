use std::collections::HashMap;

use tokio::sync::{OnceCell, RwLock};

use crate::{db::{Admin, get_all_admins}, util::log_error};


pub static ADMIN_DATA: OnceCell<RwLock<HashMap<String, Admin>>> = OnceCell::const_new();

pub async fn get_all_admin_data<'a>() -> &'a RwLock<HashMap<String, Admin>> {
      let result: &RwLock<HashMap<String, Admin>> = ADMIN_DATA.get_or_init(|| async {
            // Get the admin data from database
            let admin_data = get_all_admins().await;
            let admin_data: Vec<Admin> = match admin_data {
                  Ok(data) => data,
                  Err(err) => {
                        log_error("StaticData", format!("There's an error when trying to get static data from database. Error: {}", err.to_string()).as_str());
                        return RwLock::new(HashMap::new());
                  }
            };


            // Map all of the admin data
            let mut mapped_admin_data: HashMap<String, Admin> = HashMap::new();
            for data in admin_data {
                  mapped_admin_data.insert(data.admin_id.clone(), data);
            }


            // Return the data
            RwLock::new(mapped_admin_data)
      }).await;

      return result;
}
