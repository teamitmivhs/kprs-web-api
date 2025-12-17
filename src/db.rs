use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::method::Stream;
use surrealdb::opt::auth::Root;
use tokio;
use futures::StreamExt;

use crate::data::admin::update_admin_data;
use crate::data::live_clients::get_live_clients;
use crate::data::vote::update_votes_data;
use crate::data::voter::update_voters_data;
use crate::util::{log_error, log_something};


static SURREAL_DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Campus {
      MM,
      PD
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Voter {
    pub token: String,
    pub name: String,
    pub class: String,
    pub campus: Campus
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Candidate {
      pub name: String,
      pub campus: Campus
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Vote {
      pub voter_name: String,
      pub candidate_name: String
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Admin {
      pub admin_id: String,
      pub admin_password: String,
      pub admin_session_token: Option<String>
}

pub async fn handle_live_changes() {

      async fn voter_changes() -> surrealdb::Result<()> {
            let mut live: Stream<Vec<Voter>> = SURREAL_DB.select::<Vec<Voter>>("voter").live().await?;


            'notification_loop: while let Some(result) = live.next().await {
                  match result {
                        Ok(_) => {
                              update_voters_data().await;
                        },
                        Err(err) => {
                              log_error("LiveUpdate", format!("There's an error when trying to get the notificaton. Error: {}", err.to_string()).as_str());
                              continue 'notification_loop;
                        }
                  };

                  log_something("LiveUpdate", "There's an update for voter data!");
            }

            Ok(())
      }

      async fn admin_changes() -> surrealdb::Result<()> {
            let mut live: Stream<Vec<Admin>> = SURREAL_DB.select::<Vec<Admin>>("admin").live().await?;


            'notification_loop: while let Some(result) = live.next().await {
                  match result {
                        Ok(_) => {
                              update_admin_data().await;
                        },
                        Err(err) => {
                              log_error("LiveUpdate", format!("There's an error when trying to get the notificaton. Error: {}", err.to_string()).as_str());
                              continue 'notification_loop;
                        }
                  };

                  log_something("LiveUpdate", "There's an update for admin data!");
            }

            Ok(())
      }

      async fn votes_changes() -> surrealdb::Result<()> {
            let mut live: Stream<Vec<Vote>> = SURREAL_DB.select::<Vec<Vote>>("vote").live().await?;

            'notification_loop: while let Some(result) = live.next().await {
                  match result {
                        Ok(notification) => {
                              update_votes_data().await;
                              let live_clients = get_live_clients();
                              let mut locked_write_live_clients = live_clients.write().await;

                              let mut message: String = String::from("v");

                              message += match notification.action {
                                    surrealdb::Action::Create => "-c:",
                                    surrealdb::Action::Delete => "-d:",
                                    surrealdb::Action::Update => "-u:",
                                    _ => ""
                              };

                              message += notification.data.voter_name.as_str();
                              message += ",";
                              message += notification.data.candidate_name.as_str();

                              for (_, session) in locked_write_live_clients.iter_mut() {
                                    let _ = session.text(message.as_str()).await;
                              }
                        },
                        Err(err) => {
                              log_error("LiveUpdate", format!("There's an error when trying to get the notificaton. Error: {}", err.to_string()).as_str());
                              continue 'notification_loop;
                        }
                  }

                  log_something("LiveUpdate", "There's an update for votes data!");
            }

            Ok(())
      }


      tokio::spawn(async {
            let result = voter_changes().await;
            match result {
                  Ok(_) => (),
                  Err(err) => {
                        log_error("LiveUpdate", format!("There's an error when using live select for voter database! Error: {}", err.to_string()).as_str());
                  }
            }
      });

      tokio::spawn(async {
            let result = admin_changes().await;
            match result {
                  Ok(_) => (),
                  Err(err) => {
                        log_error("LiveUpdate", format!("There's an error when using live select for admin database! Error: {}", err.to_string()).as_str());
                  }
            }
      });

      tokio::spawn(async {
            let result = votes_changes().await;
            match result {
                  Ok(_) => (),
                  Err(err) => {
                        log_error("LiveUpdate", format!("There's an error when using live select for votes database! Error: {}", err.to_string()).as_str());
                  }
            }
      });
}


pub async fn init_db() {
      SURREAL_DB.connect::<Ws>(std::env::var("DATABASE_URL").unwrap()).await.unwrap();

      SURREAL_DB.signin(Root {
            username: std::env::var("SURREAL_USERNAME").unwrap().as_str(),
            password: std::env::var("SURREAL_PASSWORD").unwrap().as_str(),
      }).await.unwrap();

      SURREAL_DB
            .use_ns(std::env::var("SURREAL_NS_NAME").unwrap())
            .use_db(std::env::var("SURREAL_DB_NAME").unwrap())
            .await.unwrap();

      handle_live_changes().await;
}



pub async fn get_all_users() -> surrealdb::Result<Vec<Voter>> {
      SURREAL_DB.select::<Vec<Voter>>("voter").await
}

pub async fn get_user_by_token(token: String) -> surrealdb::Result<Option<Voter>> {
      let result = SURREAL_DB.query("SELECT * FROM voter WHERE token = $token")
            .bind(("token", token))
            .await?
            .take::<Vec<Voter>>(0)?;

      Ok(result.get(0).map(|voter_data| voter_data.clone()))
}




pub async fn get_all_candidates() -> surrealdb::Result<Vec<Candidate>> {
      SURREAL_DB.select::<Vec<Candidate>>("candidate").await
}



pub async fn get_all_votes() -> surrealdb::Result<Vec<Vote>> {
      SURREAL_DB.select::<Vec<Vote>>("vote").await
}

pub async fn insert_vote(voter_name: String, candidate_name: String) -> surrealdb::Result<()> {
      SURREAL_DB.insert::<Vec<Vote>>("vote")
            .content(vec![
                  Vote {
                        voter_name: voter_name,
                        candidate_name: candidate_name
                  }
            ])
            .await?;

      Ok(())
}

pub async fn remove_vote(voter_name: String) -> surrealdb::Result<()> {
      SURREAL_DB.query("DELETE FROM vote WHERE voter_name = $voter_name")
            .bind(("voter_name", voter_name))
            .await?;

      Ok(())
}

pub async fn get_all_admins() -> surrealdb::Result<Vec<Admin>> {
      SURREAL_DB.select::<Vec<Admin>>("admin").await
}

pub async fn set_admin_session_token(admin_id: impl Into<String>, admin_session_token: impl Into<String>) -> surrealdb::Result<()> {
      SURREAL_DB.query("UPDATE admin SET admin_session_token = $admin_session_token WHERE admin_id = $admin_id")
            .bind(("admin_session_token", admin_session_token.into()))
            .bind(("admin_id", admin_id.into()))
            .await?;

      Ok(())
}
