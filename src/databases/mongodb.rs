use crate::{DatabasePool, Session, SessionError, SessionStore};
use async_trait::async_trait;
use chrono::Utc;
use mongodb::{bson::{doc, Document}, Client, error::Error};
use serde::{Deserialize, Serialize};

///Redis's Session Helper type for the DatabasePool.
pub type SessionMongoSession = Session<SessionMongoPool>;
///Redis's Session Store Helper type for the DatabasePool.
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

///Redis's Pool type for the DatabasePool. Needs a redis Client.
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
    async fn initiate(&self, table_name: &str) -> Result<(), SessionError> {
        let tmp = MongoSessionData::default();
        match &self.client.default_database() {
            Some(db) => {
                let col = db.collection::<MongoSessionData>(&table_name);

                let _ = &col.insert_one(&tmp, None).await?;
                let _ = col.find_one_and_delete(tmp.to_document(), None).await?;
            },
            None => {}
        }
        Ok(())
    }

    // TODO
    async fn delete_by_expiry(&self, table_name: &str) -> Result<Vec<String>, SessionError> {
        let mut ids: Vec<String> = Vec::new();
        match &self.client.default_database() {
            Some(db) => {
                let now = Utc::now().timestamp();
                let filter = doc!{"expires":
                    {"$lte": now}
                };
                let result = db.collection::<MongoSessionData>(&table_name)
                    .find(filter.clone(), None).await?; // add filter for expiration

                for item in result.deserialize_current().iter() {
                    if !&item.id.is_empty() {
                        ids.push(item.id.clone());
                    };
                }
                db.collection::<MongoSessionData>(&table_name)
                    .delete_many(filter, None).await?;
            },
            None => {}
        }
        Ok(ids)
    }

    async fn count(&self, table_name: &str) -> Result<i64, SessionError> {
        Ok(
            match &self.client.default_database() {
                Some(db) => {
                    db.collection::<MongoSessionData>(&table_name)
                        .estimated_document_count(None).await? as i64
                },
                None => 0
            }
        )
    }

    async fn store(
        &self,
        id: &str,
        session: &str,
        expires: i64,
        table_name: &str,
    ) -> Result<(), SessionError> {
        let ses = MongoSessionData {
            id: id.to_string(),
            expires: expires,
            session: session.to_string()
        };
        
        match &self.client.default_database() {
            Some(db) => {
                db.collection::<MongoSessionData>(&table_name).insert_one(
                    MongoSessionData {
                        id: id.to_string(),
                        expires: expires,
                        session: session.to_string()
                    }, None).await?;
            },
            None => {}
        }
        Ok(())
    }

    async fn load(&self, id: &str, table_name: &str) -> Result<Option<String>, SessionError> {
        Ok(
            match &self.client.default_database() {
                Some(db) => {
                    let filter = doc!{
                        "id": id,
                        "expires":
                            {"$gte": Utc::now().timestamp()}
                    };
                    match db.collection::<MongoSessionData>(&table_name)
                        .find_one(doc!{"id": id}, None).await.unwrap_or_default()
                    {
                        Some (result) => {
                            if result.session.is_empty() {
                                None
                            } else {
                                Some(result.session)
                            }
                        },
                        None => None
                    }
                },
                None => None
            }
        )
    }

    async fn delete_one_by_id(&self, id: &str, table_name: &str) -> Result<(), SessionError> {
        match &self.client.default_database() {
            Some(db) => {
                let _ = db.collection::<MongoSessionData>(&table_name)
                    .delete_one(doc!{"id": id}, None).await?;
            },
            None => {}
        }
        Ok(())
    }

    async fn exists(&self, id: &str, table_name: &str) -> Result<bool, SessionError> {
        Ok(
            match &self.client.default_database() {
                Some(db) => {
                    db.collection::<MongoSessionData>(&table_name)
                        .find_one(doc!{"id": id}, None).await?.is_some()
                },
                None => false
            }
        )
    }

    async fn delete_all(&self, table_name: &str) -> Result<(), SessionError> {
        match &self.client.default_database() {
            Some(db) => {
                let _ = db.collection::<MongoSessionData>(&table_name)
                    .drop(None).await?;
            },
            None => {}
        }
        Ok(())
    }

    async fn get_ids(&self, table_name: &str) -> Result<Vec<String>, SessionError> {
        let mut ids: Vec<String> = Vec::new();
        match &self.client.default_database() {
            Some(db) => {
                let filter = doc!{"expires":
                    {"$gte": Utc::now().timestamp()}
                };
                let result = db.collection::<MongoSessionData>(&table_name)
                    .find(filter, None).await?; // add filter for expiration

                for item in result.deserialize_current().iter() {
                    if !&item.id.is_empty() {
                        ids.push(item.id.clone());
                    };
                }
            },
            None => {}
        }
        Ok(ids)
    }

    fn auto_handles_expiry(&self) -> bool {
        false
    }
}