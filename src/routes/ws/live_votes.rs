use std::{collections::HashMap, sync::Arc};

use actix_ws::handle;
use futures_util::StreamExt;
use actix_web::{HttpRequest, HttpResponse, get, web};
use surrealdb::Uuid;
use tokio::sync::RwLock;

use crate::{data::live_clients::get_live_clients};

#[get("/ws/votes")]
pub async fn ws_handler(req: HttpRequest, body: web::Payload) -> actix_web::Result<HttpResponse> {
      let (response, mut session, mut msg_stream) = handle(&req, body)?;

      let client_id: String = Uuid::new_v4().to_string();
      {
            let live_clients: Arc<RwLock<HashMap<String, actix_ws::Session>>> = get_live_clients();
            let mut locked_write_live_clients = live_clients.write().await;
            locked_write_live_clients.insert(client_id.clone(), session.clone());
      }

      actix_web::rt::spawn(async move {
            while let Some(Ok(msg)) = msg_stream.next().await {
                  match msg {
                        actix_ws::Message::Ping(bytes) => {
                              let _ = session.pong(&bytes).await;
                        }
                        _ => ()
                  }
            }

            let live_clients: Arc<RwLock<HashMap<String, actix_ws::Session>>> = get_live_clients();
            let mut locked_write_live_clients = live_clients.write().await;
            locked_write_live_clients.remove(&client_id);
      });

      Ok(response)
}
