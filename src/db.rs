use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;


static SURREAL_DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);

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
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Voter {
    pub token: String,
    pub name: String,
}

pub async fn get_all_users() -> surrealdb::Result<Vec<Voter>> {
      SURREAL_DB.select::<Vec<Voter>>("voters").await
}

pub async fn get_user_by_token(token: String) -> surrealdb::Result<Option<Voter>> {
      let result = SURREAL_DB.query("SELECT * FROM voters WHERE token = $token")
            .bind(("token", token))
            .await?
            .take::<Vec<Voter>>(0)?;

      Ok(result.get(0).map(|voter_data| voter_data.clone()))
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Candidate {
      pub name: String
}

pub async fn get_all_candidates() -> surrealdb::Result<Vec<Candidate>> {
      SURREAL_DB.select::<Vec<Candidate>>("candidates").await
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vote {
      pub voter_name: String,
      pub candidate_name: String
}

pub async fn get_all_votes() -> surrealdb::Result<Vec<Vote>> {
      SURREAL_DB.select::<Vec<Vote>>("votes").await
}

pub async fn insert_vote(voter_name: String, candidate_name: String) -> surrealdb::Result<()> {
      SURREAL_DB.insert::<Vec<Vote>>("votes")
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
      SURREAL_DB.query("DELETE FROM votes WHERE voter_name = $voter_name")
            .bind(("voter_name", voter_name))
            .await?;

      Ok(())
}
