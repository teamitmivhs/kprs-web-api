use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

pub static LIVE_CLIENTS: Lazy<Arc<RwLock<HashMap<String, actix_ws::Session>>>> = Lazy::new(|| {
      Arc::new(RwLock::new(HashMap::new()))
});

pub fn get_live_clients() -> Arc<RwLock<HashMap<String, actix_ws::Session>>> {
      LIVE_CLIENTS.clone()
}
