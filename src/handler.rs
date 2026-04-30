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
        tracing::trace!(
            "Killed Session Manager Duplicate on thread ID: {:?}.",
            std::thread::current().id()
        );
        return Ok(());
    } else {
        tracing::trace!("Session Manager is starting loop");
    }

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

            // We do this to ensure that if Types that never got updated due to the newer timers that on memory unload
            // we ensure they get synced before unloading them from memory but only if the database/cookie has not expired.
            if session_store.is_persistent() {
                for session in session_store
                    .inner
                    .iter()
                    .filter(|r| r.autoremove < current_time)
                {
                    if !session.expired() {
                        if let Err(err) = session_store.store_session(&session).await {
                            tracing::debug!(
                                "Session Failed to save to Database with error: {}",
                                err
                            );
                            session_store.timers.write().await.last_expiry_sweep =
                                Utc::now() + session_store.config.memory.purge_update;
                            break;
                        } else {
                            tracing::debug!(
                                "Session id {}: was saved to the database.",
                                session.id
                            );
                        }
                    }
                }
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
                    tracing::error!("Session Database Cleaning Failed with error: {}", err);
                    session_store
                        .timers
                        .write()
                        .await
                        .last_database_expiry_sweep =
                        Utc::now() + session_store.config.database.purge_database_update;
                    break;
                }
            };

            #[cfg(not(feature = "key-store"))]
            if let Err(err) = session_store.cleanup().await {
                tracing::error!("Session Database Cleaning Failed with error: {}", err);
                session_store
                    .timers
                    .write()
                    .await
                    .last_database_expiry_sweep =
                    Utc::now() + session_store.config.database.purge_database_update;
                break;
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

    Ok(())
}
