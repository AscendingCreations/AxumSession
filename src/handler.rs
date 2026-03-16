use crate::{DatabasePool, SessionError, SessionStore};
use chrono::Utc;
#[cfg(feature = "key-store")]
use fastbloom_rs::Deletable;
use std::fmt::Debug;
use tokio::time::sleep;

pub async fn runner<T>(
    session_store: SessionStore<T>,
    kill_as_duplicate: bool,
) -> Result<(), SessionError>
where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    if kill_as_duplicate {
        tracing::trace!("Killed Session Manager Duplicate.");
        return Ok(());
    }

    tracing::trace!("Session Manager is starting loop");

    loop {
        let (last_sweep, last_database_sweep) = {
            let timers = session_store.timers.read().await;
            (timers.last_expiry_sweep, timers.last_database_expiry_sweep)
        };

        let current_time = Utc::now();

        if last_sweep <= current_time && !session_store.config.memory.memory_lifespan.is_zero() {
            tracing::trace!("Session Memory Cleaning Started");

            // Only unload these from filter if the Client is None as this means no database.
            // Otherwise only unload from the filter if removed from the Database.
            #[cfg(feature = "key-store")]
            if session_store.is_persistent()
                && session_store.auto_handles_expiry()
                && session_store.config.memory.use_bloom_filters
            {
                let mut filter = session_store.filter.write().await;
                session_store
                    .inner
                    .iter()
                    .filter(|r| r.autoremove < current_time)
                    .for_each(|r| filter.remove(r.key().as_bytes()));
            }

            session_store
                .inner
                .retain(|_k, v| v.autoremove > current_time);

            session_store.timers.write().await.last_expiry_sweep =
                Utc::now() + session_store.config.memory.purge_update;

            tracing::trace!("Session Memory Cleaning Finished");
        }

        // Throttle by database lifespan - e.g. sweep every 6 hours
        if last_database_sweep <= current_time && session_store.is_persistent() {
            tracing::trace!("Session Database Cleaning Started");

            //Remove any old keys that expired and Remove them from our loaded filter.
            #[cfg(feature = "key-store")]
            let expired = match session_store.cleanup().await {
                Ok(v) => v,
                Err(err) => {
                    tracing::error!("Session Database Cleaning Failed",);
                    continue;
                }
            };

            #[cfg(not(feature = "key-store"))]
            if session_store.cleanup().await.is_err() {
                tracing::error!("Session Database Cleaning Failed",);
                continue;
            }

            #[cfg(feature = "key-store")]
            if !session_store.auto_handles_expiry() {
                let mut filter = session_store.filter.write().await;
                expired.iter().for_each(|id| filter.remove(id.as_bytes()));
            }

            session_store
                .timers
                .write()
                .await
                .last_database_expiry_sweep =
                Utc::now() + session_store.config.database.purge_database_update;

            tracing::trace!("Session Database Cleaning Finished");
        }

        tracing::trace!("Session Manager is Sleeping");
        sleep(
            session_store
                .config
                .thread_sleep_duration
                .to_std()
                .unwrap_or_default(),
        )
        .await;
    }
}
