use axum_session::DatabaseError;
use redis::aio::ConnectionLike;
use redis_pool::connection::RedisPoolConnection;

pub async fn scan_keys<C>(
    con: &mut RedisPoolConnection<C>,
    pattern: &str,
) -> Result<Vec<String>, DatabaseError>
where
    C: ConnectionLike + Send + Sync + 'static,
{
    // SCAN works like KEYS but it is safe to use in production.
    // Instead of blocking the server, it will only return a small
    // amount of keys per iteration.
    // https://redis.io/commands/scan

    let mut keys: Vec<String> = Vec::new();
    let mut cursor: i32 = 0;

    loop {
        let (new_cursor, new_keys): (i32, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .query_async(con)
            .await
            .map_err(|err| DatabaseError::GenericSelectError(err.to_string()))?;

        keys.extend(new_keys);

        cursor = new_cursor;
        if cursor == 0 {
            break;
        }
    }

    Ok(keys)
}
