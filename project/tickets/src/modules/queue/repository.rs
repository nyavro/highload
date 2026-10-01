use fred::clients::Pool;
use fred::prelude::*;
use std::{error::Error, sync::Arc};
use uuid::Uuid;

pub struct RedisQueueRepository {
    pool: Arc<Pool>,
}

impl RedisQueueRepository {
    pub fn new(pool: Arc<Pool>) -> Self {
        Self { pool }
    }

    pub async fn is_session_active(&self, event_id: Uuid, user_id: Uuid) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let active_key = format!("active_sessions:event:{}", event_id);
        let exists: bool = self.pool.hexists(active_key, user_id.to_string()).await?;
        Ok(exists)
    }

    // Returns the user's position in the queue (1-based)
    pub async fn join_queue(&self, event_id: Uuid, user_id: Uuid, timestamp_ms: i64) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let queue_key = format!("queue:event:{}", event_id);
        let user_str = user_id.to_string();
        let score = timestamp_ms as f64;
        let _: i64 = self.pool.zadd(&queue_key, None, None, false, false, (score, user_str.clone())).await?;
        
        let rank: Option<i64> = self.pool.zrank(&queue_key, user_str, false).await?;        
        Ok(rank.unwrap_or(0) + 1) 
    }

    // Promotes first N users from queue to list of active sessions with automatic eviction of expired ones
    pub async fn promote_queue(&self, event_id: Uuid, limit: i64, session_ttl_sec: i64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let queue_key = format!("queue:event:{}", event_id);
        let active_key = format!("active_sessions:event:{}", event_id);
        let now_timestamp = chrono::Utc::now().timestamp();
        
        // Atomic operation to promote users from the queue to active sessions while cleaning up expired sessions.
        // 1. Checks for expired sessions in the active_sessions hash and removes them.
        // 2. Counts how many active sessions are currently present.
        // 3. If there are available slots (limit - current_active), it promotes users
        let lua_script = r#"
            local queue_key = KEYS[1]
            local active_key = KEYS[2]
            local limit = tonumber(ARGV[1])
            local now_time = tonumber(ARGV[2])
            local ttl_sec = tonumber(ARGV[3])

            -- step 1: Clean up expired sessions
            local all_active = redis.call("HGETALL", active_key)
            for i = 1, #all_active, 2 do
                local user_id = all_active[i]
                local expire_at = tonumber(all_active[i+1])
                if expire_at and expire_at < now_time then
                    redis.call("HDEL", active_key, user_id)
                end
            end

            -- step 2: Calculate available slots
            local current_active = redis.call("HLEN", active_key)
            local slots_available = limit - current_active
            if slots_available <= 0 then
                return 0
            end

            -- step 3: Promote users from queue to active sessions
            local waiters = redis.call("ZRANGE", queue_key, 0, slots_available - 1)
            if #waiters == 0 then
                return 0
            end

            local promoted_count = 0
            for _, user_id in ipairs(waiters) do
                local expire_at_str = tostring(now_time + ttl_sec)
                redis.call("HSET", active_key, user_id, expire_at_str)
                redis.call("ZREM", queue_key, user_id)
                promoted_count = promoted_count + 1
            end

            return promoted_count
        "#;

        let keys: Vec<Key> = vec![queue_key.into(), active_key.into()];        
        let args: Vec<Value> = vec![
            Value::String(limit.to_string().into()),
            Value::String(now_timestamp.to_string().into()),
            Value::String(session_ttl_sec.to_string().into()),
        ]; 
        let promoted_count: i32 = self.pool.eval(lua_script, keys, args).await?;        
        Ok(promoted_count as usize)
    }
    
    pub async fn get_active_sessions(&self, event_id: Uuid) -> i64 {
        let active_key = format!("active_sessions:event:{}", event_id);    
        self.pool.hlen(active_key).await.unwrap_or(0)
    }
    
    pub async fn move_user_to_active(&self, event_id: Uuid, user_id: Uuid) -> i32 {
        let active_key_str = format!("active_sessions:event:{}", event_id);
        // Метод Fast Path: выдает сессию на 120 секунд прямо из Middleware
        let expire_at = chrono::Utc::now().timestamp() + 120;                
        self.pool.hset(active_key_str, (user_id.to_string(), expire_at.to_string())).await.unwrap_or(0)
    }

    // Checks user status
    /// Returns Ok(None), if the user is already admitted (in active_sessions), 
    /// and Ok(Some(position)), if they are still waiting in the queue ZSET.
    pub async fn get_queue_status(&self, event_id: Uuid, user_id: Uuid) -> Result<Option<i64>, Box<dyn Error + Send + Sync>> {
        // 1. First, we check if the user has already been admitted by the worker   
        let active = self.is_session_active(event_id, user_id).await?;
        if active {
            return Ok(None);
        }        
        let queue_key = format!("queue:event:{}", event_id);
        let rank: Option<i64> = self.pool.zrank(&queue_key, user_id.to_string(), false).await?;
        match rank {
            Some(r) => Ok(Some(r + 1)), // Returns the position (index + 1)
            None => {
                // If they are not in either active or queue — they either disconnected or 
                // are requesting status without joining the queue. We'll treat this as position 0 (need to join the queue).
                Ok(Some(0))
            }
        }
    }
}
