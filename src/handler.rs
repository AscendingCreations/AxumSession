use crate::{DatabasePool, SessionData, SessionError, SessionStore};
#[cfg(feature = "key-store")]
use fastbloom_rs::Deletable;
use std::fmt::Debug;
use time::OffsetDateTime;
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

        let current_time = OffsetDateTime::now_utc();

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
                save_due_sessions(&session_store, current_time).await;
            }

            session_store
                .inner
                .retain(|_k, v| v.autoremove > current_time);

            session_store.timers.write().await.last_expiry_sweep =
                OffsetDateTime::now_utc() + session_store.config.memory.purge_update;

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
                    break;
                }
            };

            #[cfg(not(feature = "key-store"))]
            if let Err(err) = session_store.cleanup().await {
                tracing::error!("Session Database Cleaning Failed with error: {}", err);
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
                OffsetDateTime::now_utc() + session_store.config.database.purge_database_update;

            tracing::trace!("Session Database Cleaning Finished");
        }

        tracing::trace!("Session Manager is Sleeping");
        sleep(session_store.config.thread_sleep_duration.unsigned_abs()).await;
    }

    Ok(())
}

/// Saves every in-memory session that is due for unloading (`autoremove` passed)
/// and not yet expired to the persistent store, stopping at the first failed save.
///
/// The due sessions are cloned out first so no DashMap shard lock is held across
/// the database await: a request writing a locked shard blocks its worker thread
/// synchronously, which can stall the I/O driver the await itself needs and
/// deadlock the runtime.
pub(crate) async fn save_due_sessions<T>(
    session_store: &SessionStore<T>,
    current_time: OffsetDateTime,
) where
    T: DatabasePool + Clone + Debug + Sync + Send + 'static,
{
    let due: Vec<SessionData> = session_store
        .inner
        .iter()
        .filter(|r| r.autoremove < current_time && !r.expired())
        .map(|r| r.value().clone())
        .collect();

    for session in &due {
        if let Err(err) = session_store.store_session(session).await {
            tracing::debug!("Session Failed to save to Database with error: {}", err);
            break;
        } else {
            tracing::debug!("Session id {}: was saved to the database.", session.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::save_due_sessions;
    use crate::{DatabaseError, DatabasePool, SessionConfig, SessionData, SessionStore};
    use async_trait::async_trait;
    use std::sync::mpsc;
    use std::time::Duration as StdDuration;
    use time::{Duration, OffsetDateTime};
    use tokio::sync::watch;

    /// A persistent store whose `store` awaits a timer before succeeding, and
    /// publishes the id it is saving so the test can write that session's shard
    /// while the save is suspended.
    #[derive(Clone, Debug)]
    struct SlowPool {
        saving: watch::Sender<Option<String>>,
    }

    #[async_trait]
    impl DatabasePool for SlowPool {
        async fn initiate(&self, _table_name: &str) -> Result<(), DatabaseError> {
            Ok(())
        }
        async fn count(&self, _table_name: &str) -> Result<i64, DatabaseError> {
            Ok(0)
        }
        async fn store(
            &self,
            id: &str,
            _session: &str,
            _expires: i64,
            _table_name: &str,
        ) -> Result<(), DatabaseError> {
            self.saving.send_replace(Some(id.to_owned()));
            tokio::time::sleep(StdDuration::from_millis(50)).await;
            Ok(())
        }
        async fn load(
            &self,
            _id: &str,
            _table_name: &str,
        ) -> Result<Option<String>, DatabaseError> {
            Ok(None)
        }
        async fn delete_one_by_id(
            &self,
            _id: &str,
            _table_name: &str,
        ) -> Result<(), DatabaseError> {
            Ok(())
        }
        async fn exists(&self, _id: &str, _table_name: &str) -> Result<bool, DatabaseError> {
            Ok(false)
        }
        async fn delete_by_expiry(&self, _table_name: &str) -> Result<Vec<String>, DatabaseError> {
            Ok(Vec::new())
        }
        async fn delete_all(&self, _table_name: &str) -> Result<(), DatabaseError> {
            Ok(())
        }
        async fn get_ids(&self, _table_name: &str) -> Result<Vec<String>, DatabaseError> {
            Ok(Vec::new())
        }
        fn auto_handles_expiry(&self) -> bool {
            false
        }
    }

    /// Runs the sweep's save step on a ONE-worker runtime while a second task
    /// writes the very session being saved. If the save held that session's
    /// DashMap shard lock across its `store` await, the writer would block the
    /// only worker, the save's timer could never fire, and the runtime would hang;
    /// the watchdog on this (non-runtime) thread turns that hang into a failure.
    #[test]
    fn save_due_sessions_does_not_hold_a_shard_lock_across_the_store_await() {
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .expect("build runtime");
            runtime.block_on(async {
                let (saving, mut saving_rx) = watch::channel(None);
                let config = SessionConfig::default();
                let store = SessionStore::new(Some(SlowPool { saving }), config.clone())
                    .await
                    .expect("build session store");

                let now = OffsetDateTime::now_utc();
                for n in 0..4 {
                    let mut session = SessionData::new(format!("session-{n}"), true, &config);
                    session.autoremove = now - Duration::minutes(1);
                    session.expires = now + Duration::hours(1);
                    store.inner.insert(session.id.clone(), session);
                }

                let sweeper = store.clone();
                let sweep = tokio::spawn(async move { save_due_sessions(&sweeper, now).await });

                let writer = store.clone();
                let write = tokio::spawn(async move {
                    let id = saving_rx
                        .wait_for(Option::is_some)
                        .await
                        .expect("saving channel open")
                        .clone()
                        .expect("an id is being saved");
                    let session = SessionData::new(id.clone(), true, &config);
                    writer.inner.insert(id, session);
                });

                sweep.await.expect("sweep task");
                write.await.expect("writer task");
            });
            done_tx.send(()).expect("report completion");
        });

        assert!(
            done_rx.recv_timeout(StdDuration::from_secs(5)).is_ok(),
            "the save step deadlocked against a concurrent write to the shard it was saving"
        );
    }
}
