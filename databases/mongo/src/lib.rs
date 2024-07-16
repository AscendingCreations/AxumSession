#![doc = include_str!("../README.md")]
#![allow(dead_code)]
#![warn(clippy::all, nonstandard_style, future_incompatible)]
#![forbid(unsafe_code)]

use async_trait::async_trait;
use axum_session::{DatabaseError, DatabasePool, Session, SessionStore};
use chrono::Utc;
use mongodb::{
    bson::{doc, Document},
    Client,
};
use serde::{Deserialize, Serialize};

pub type SessionMongoSession = Session<SessionMongoPool>;
pub type SessionMongoSessionStore = SessionStore<SessionMongoPool>;

#[derive(Default, Debug, Serialize, Deserialize)]
struct MongoSessionData {
    id: String,
    expires: i64,
    session: String,
}
impl MongoSessionData {
    fn to_document(&self) -> Document {
        doc! {
            "id": &self.id,
            "expires": self.expires,
            "session": &self.session
        }
    }
}

///Mongodb's Pool type for the DatabasePool. Needs a mongodb Client.
#[derive(Debug, Clone)]
pub struct SessionMongoPool {
    client: Client,
}

impl From<Client> for SessionMongoPool {
    fn from(client: Client) -> Self {
        SessionMongoPool { client }
    }
}

#[async_trait]
impl DatabasePool for SessionMongoPool {
    // Make sure the collection exists in the database
    // by inserting a record then deleting it
    async fn initiate(&self, table_name: &str) -> Result<(), DatabaseError> {
        let tmp = MongoSessionData::default();
        match &self.client.default_database() {
            Some(db) => {
                let col = db.collection::<MongoSessionData>(table_name);

                let _ = &col
                    .insert_one(&tmp)
                    .await
                    .map_err(|err| DatabaseError::GenericInsertError(err.to_string()))?;
                let _ = col
                    .find_one_and_delete(tmp.to_document())
                    .await
                    .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;
            }
            None => {}
        }
        Ok(())
    }

    async fn delete_by_expiry(&self, table_name: &str) -> Result<Vec<String>, DatabaseError> {
        let mut ids: Vec<String> = Vec::new();
        match &self.client.default_database() {
            Some(db) => {
                let now = Utc::now().timestamp();
                let filter = doc! {"expires":
                    {"$lte": now}
                };
                let result = db
                    .collection::<MongoSessionData>(table_name)
                    .find(filter.clone())
                    .await
                    .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

                for item in result.deserialize_current().iter() {
                    if !&item.id.is_empty() {
                        ids.push(item.id.clone());
                    };
                }
                db.collection::<MongoSessionData>(table_name)
                    .delete_many(filter)
                    .await
                    .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;
            }
            None => {}
        }
        Ok(ids)
    }

    async fn count(&self, table_name: &str) -> Result<i64, DatabaseError> {
        Ok(match &self.client.default_database() {
            Some(db) => db
                .collection::<MongoSessionData>(table_name)
                .estimated_document_count()
                .await
                .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?
                as i64,
            None => 0,
        })
    }

    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        table_name: &str,
    ) -> Result<(), DatabaseError> {
        match &self.client.default_database() {
            Some(db) => {
                let filter = doc! {
                    "id": id
                };
                let update_data = doc! {"$set": {
                    "id": id.to_string(),
                    "expires": expires,
                    "session": session.to_string()
                }};

                db.collection::<MongoSessionData>(table_name)
                    .update_one(filter, update_data)
                    .upsert(true)
                    .await
                    .map_err(|err| DatabaseError::GenericInsertError(err.to_string()))?;
            }
            None => {}
        }
        Ok(())
    }

    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, DatabaseError> {
        Ok(match &self.client.default_database() {
            Some(db) => {
                let filter = doc! {
                    "id": id,
                    "expires":
                        {"$gte": Utc::now().timestamp()}
                };
                match db
                    .collection::<MongoSessionData>(table_name)
                    .find_one(filter)
                    .await
                    .unwrap_or_default()
                {
                    Some(result) => {
                        if result.session.is_empty() {
                            None
                        } else {
                            Some(result.session)
                        }
                    }
                    None => None,
                }
            }
            None => None,
        })
    }

    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), DatabaseError> {
        match &self.client.default_database() {
            Some(db) => {
                let _ = db
                    .collection::<MongoSessionData>(table_name)
                    .delete_one(doc! {"id": id})
                    .await
                    .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;
            }
            None => {}
        }
        Ok(())
    }

    async fn exists(&self, id: &str, table_name: &str) -> Result<bool, DatabaseError> {
        Ok(match &self.client.default_database() {
            Some(db) => db
                .collection::<MongoSessionData>(table_name)
                .find_one(doc! {"id": id})
                .await
                .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?
                .is_some(),
            None => false,
        })
    }

    async fn delete_all(&self, table_name: &str) -> Result<(), DatabaseError> {
        match &self.client.default_database() {
            Some(db) => {
                let _ = db
                    .collection::<MongoSessionData>(table_name)
                    .drop()
                    .await
                    .map_err(|err| DatabaseError::GenericDeleteError(err.to_string()))?;
            }
            None => {}
        }
        Ok(())
    }

    async fn get_ids(&self, table_name: &str) -> Result<Vec<String>, DatabaseError> {
        let mut ids: Vec<String> = Vec::new();
        match &self.client.default_database() {
            Some(db) => {
                let filter = doc! {"expires":
                    {"$gte": Utc::now().timestamp()}
                };
                let result = db
                    .collection::<MongoSessionData>(table_name)
                    .find(filter)
                    .await
                    .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?; // add filter for expiration

                for item in result.deserialize_current().iter() {
                    if !&item.id.is_empty() {
                        ids.push(item.id.clone());
                    };
                }
            }
            None => {}
        }
        Ok(ids)
    }

    fn auto_handles_expiry(&self) -> bool {
        false
    }
}
