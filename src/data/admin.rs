use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::{db::{Admin, get_all_admins}, util::{log_error, log_something}};


pub static ADMIN_DATA: Lazy<Arc<RwLock<HashMap<String, Admin>>>> = Lazy::new(|| {
      Arc::new(RwLock::new(HashMap::new()))
});

pub async fn update_admin_data() {
      // Get the admin data from database
      let admin_data = get_all_admins().await;
      let admin_data: Vec<Admin> = match admin_data {
            Ok(data) => data,
            Err(err) => {
                  log_error("StaticData", format!("There's an error when trying to get static data from database. Error: {}", err.to_string()).as_str());
                  return;
            }
      };


      // Map all of the admin data
      let mut locked_write_admin_data = ADMIN_DATA.write().await;
      locked_write_admin_data.clear();
      for data in admin_data {
            locked_write_admin_data.insert(data.admin_id.clone(), data);
      }


      log_something("StaticData", "Static users data successfully updated!");
}

pub fn get_all_admin_data() -> Arc<RwLock<HashMap<String, Admin>>> {
      return ADMIN_DATA.clone()
}

pub async fn init_admin_data() {
      update_admin_data().await;
}
