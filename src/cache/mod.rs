// Redis cache layer.
//
// Implements a cache-through pattern:
//   1. Check Redis for a cached value
//   2. On miss: fetch from Sanity, serialize to JSON, store in Redis with TTL
//   3. Return the value
//
// Uses `fred` — an async Redis client with proper TLS support.

use fred::prelude::*;
use fred::clients::Client;

/// Cache key generators. Consistent naming enables targeted invalidation.
pub mod keys {
    pub fn posts_page(page: u32) -> String {
        format!("blog:posts:page:{}", page)
    }

    pub fn post(slug: &str) -> String {
        format!("blog:post:{}", slug)
    }

    pub fn tag(slug: &str) -> String {
        format!("blog:tag:{}", slug)
    }

    #[allow(dead_code)]
    pub fn post_count() -> String {
        "blog:post_count".to_string()
    }

    #[allow(dead_code)]
    pub fn post_count_tag(tag: &str) -> String {
        format!("blog:post_count:tag:{}", tag)
    }

    pub fn reaction(post_slug: &str, emoji: &str) -> String {
        format!("blog:react:{}:{}", post_slug, emoji)
    }
}

/// Retrieve a value from Redis. Returns `None` if the key doesn't exist.
pub async fn get<T: FromValue>(
    client: &Client,
    key: &str,
) -> Result<Option<T>, String> {
    let result: Option<T> = client
        .get(key)
        .await
        .map_err(|e| format!("Redis GET error: {}", e))?;
    Ok(result)
}

/// Store a string value in Redis with a TTL in seconds.
pub async fn set(
    client: &Client,
    key: &str,
    value: &str,
    ttl_seconds: u64,
) -> Result<(), String> {
    let _: () = client
        .set(key, value, Some(Expiration::EX(ttl_seconds as i64)), None, false)
        .await
        .map_err(|e| format!("Redis SET error: {}", e))?;
    Ok(())
}

/// Atomically increment a counter by `amount`. Creates the key at 0 if it doesn't exist.
pub async fn increment(
    client: &Client,
    key: &str,
    amount: i64,
) -> Result<i64, String> {
    let new_value: i64 = client
        .incr_by(key, amount)
        .await
        .map_err(|e| format!("Redis INCR error: {}", e))?;
    Ok(new_value)
}

/// Retrieve and deserialize a JSON value from Redis.
pub async fn get_json<T: for<'de> serde::Deserialize<'de>>(
    client: &Client,
    key: &str,
) -> Result<Option<T>, String> {
    let json_str: Option<String> = get(client, key).await?;
    match json_str {
        Some(json) => {
            let value: T =
                serde_json::from_str(&json).map_err(|e| format!("JSON deserialize error: {}", e))?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

/// Serialize a value to JSON and store it in Redis with a TTL.
pub async fn set_json<T: serde::Serialize>(
    client: &Client,
    key: &str,
    value: &T,
    ttl_seconds: u64,
) -> Result<(), String> {
    let json = serde_json::to_string(value).map_err(|e| format!("JSON serialize error: {}", e))?;
    set(client, key, &json, ttl_seconds).await
}
