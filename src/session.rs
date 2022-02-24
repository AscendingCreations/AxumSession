use crate::{AxumSessionData, AxumSessionID, AxumSessionStore};
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    http::{self, StatusCode},
};
use serde::de::DeserializeOwned;
use serde::Serialize;

///This is the Session that is generated when a user is routed to a page that Needs one
/// It is used to Save and load session data similar to how it is done on python.
#[derive(Debug, Clone)]
pub struct AxumSession {
    pub(crate) store: AxumSessionStore,
    pub(crate) id: AxumSessionID,
}

/// this auto pulls a SqlxSession from the extensions when added by the Session managers call
/// if for some reason the Session Manager did not run this will Error.
#[async_trait]
impl<B> FromRequest<B> for AxumSession
where
    B: Send,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let extensions = req.extensions().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Can't extract AxumSession: extensions has been taken by another extractor",
        ))?;
        extensions.get::<AxumSession>().cloned().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Can't extract AxumSession. Is `AxumSessionLayer` enabled?",
        ))
    }
}

impl AxumSession {
    ///Runs a Closure that can return Data from the users SessionData Hashmap.
    pub async fn tap<T: DeserializeOwned>(
        &self,
        func: impl FnOnce(&mut AxumSessionData) -> Option<T>,
    ) -> Option<T> {
        let store_rg = self.store.inner.read().await;

        let mut instance = store_rg
            .get(&self.id.0.to_string())
            .expect("Session data unexpectedly missing")
            .lock()
            .await;

        func(&mut instance)
    }

    ///Sets the Entire Session to be Cleaned on next load.
    pub async fn destroy(&self) {
        self.tap(|sess| {
            sess.destroy = true;
            Some(1)
        })
        .await;
    }

    ///Used to get data stored within SessionDatas hashmap from a key value.
    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.tap(|sess| {
            let string = sess.data.get(key)?;
            serde_json::from_str(string).ok()
        })
        .await
    }

    /// Used to Set data to SessionData via a Key and the Value to Set.
    pub async fn set(&self, key: &str, value: impl Serialize) {
        let value = serde_json::to_string(&value).unwrap_or_else(|_| "".to_string());

        self.tap(|sess| {
            if sess.data.get(key) != Some(&value) {
                sess.data.insert(key.to_string(), value);
            }
            Some(1)
        })
        .await;
    }

    ///used to remove a key and its data from SessionData's Hashmap
    pub async fn remove(&self, key: &str) {
        self.tap(|sess| sess.data.remove(key)).await;
    }

    /// Will instantly clear all data from SessionData's Hashmap
    pub async fn clear_all(&self) {
        let store_rg = self.store.inner.read().await;

        let mut sess = store_rg
            .get(&self.id.0.to_string())
            .expect("Session data unexpectedly missing")
            .lock()
            .await;

        sess.data.clear();
        if self.store.is_persistent() {
            self.store.clear_store().await.unwrap();
        }
    }

    /// Returns a Count of all Sessions currently within the Session Store.
    /// If is_persistant then returns all database saved sessions.
    /// if not is_persistant returns count of sessions in memory store.
    pub async fn count(&self) -> i64 {
        if self.store.is_persistent() {
            self.store.count().await.unwrap_or(0i64)
        } else {
            self.store.inner.read().await.len()
        }
    }
}
