use fred::prelude::Pool;
use uuid::Uuid;
use crate::modules::post::{model::Post};
use fred::prelude::{SortedSetsInterface};
use fred::prelude::*;
use fred::error::Error;
use std::sync::Arc;
use async_trait::async_trait; 
use mockall::automock;

const DEFAULT_FEED_SIZE: i64 = 100;

#[automock]
#[async_trait]
pub trait PostCache {        
    async fn update_followers_feeds(&self, followers_ids: Vec<Uuid>, post: &Post) -> Result<(), Error>;    
    async fn get_user_feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<String>, Error>;
    async fn save_user_feed(&self, user_id: Uuid, posts: &Vec<Post>) -> Result<(), Error>;

    async fn get_posts_by_ids(&self, ids: Vec<String>) -> Result<Vec<Post>, Error>;        
    async fn save_post(&self, post: &Post) -> Result<(), Error>;
    async fn save_posts(&self, posts: &Vec<Post>) -> Result<(), Error>;
    async fn delete_post(&self, post_id: &Uuid) -> Result<(), Error>;
    async fn get_post(&self, post_id: &Uuid) -> Result<Option<Post>, Error>;

    async fn mark_feed_exists(&self, user_id: Uuid) -> Result<(), Error>;
    async fn check_feed_exists(&self, user_id: Uuid) -> Result<bool, Error>;
}

pub struct PostCacheImpl {
    pool: Arc<Pool>
} 

impl PostCacheImpl {
    pub fn new(pool: Arc<Pool>) -> Self {
        PostCacheImpl { pool }
    }
}

#[async_trait]
impl PostCache for PostCacheImpl {
    
    async fn save_post(&self, post: &Post) -> Result<(), Error> {        
        let item_key = format!("post:{}", post.id);            
        self.pool.next().hset(
            "posts_data_store", //TODO
            (item_key, serde_json::to_string(&post).expect("Failed to serialize post"))
        ).await         
    }   

    async fn save_posts(&self, posts: &Vec<Post>) -> Result<(), Error> {
        let entries: Vec<(String, String)> = posts
            .iter()
            .map(|post| {
                let item_key = format!("post:{}", post.id);
                let json = serde_json::to_string(post).expect("Failed to serialize post");
                (item_key, json)
            })
            .collect();
        if entries.is_empty() {
            return Ok(());
        }        
        self.pool.next().hset("posts_data_store", entries).await
    }

    async fn get_post(&self, post_id: &Uuid) -> Result<Option<Post>, Error> {            
        let item_key = format!("post:{}", post_id);            
        self.pool.next().hget::<Option<String>, _, _>(
            "posts_data_store", //TODO
            item_key
        ).await
            .map(|maybe_json|
                maybe_json.map(|post_json| serde_json::from_str(&post_json).expect("Failed to deserialize post"))
            )
    }   

    async fn delete_post(&self, post_id: &Uuid) -> Result<(), Error> {
        let item_key = format!("post:{}", post_id);            
        self.pool.next().hdel(
            "posts_data_store", //TODO
            item_key
        ).await
    }

    async fn update_followers_feeds(&self, followers_ids: Vec<Uuid>, post: &Post) -> Result<(), Error> {      
        let pipeline = self.pool.next().pipeline();        
        for follower_id in &followers_ids {
            let cache_key = format!("highload/post/feed/ids/{}", follower_id);                                                        
            let _ = pipeline.zadd::<(), _, _>(cache_key, None, None, false, false, (post.timestamp.timestamp() as f64, post.id.to_string())).await;
        }
        pipeline.last().await
    }

    async fn get_user_feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<String>, Error> {
        let cache_key = format!("highload/post/feed/ids/{}", user_id);
        let start = offset.unwrap_or(0) as i64;
        let stop = start + (limit.unwrap_or(DEFAULT_FEED_SIZE) as i64) - 1;
        self.pool.next().zrevrange::<Vec<String>, _>(cache_key, start, stop, false).await
    }

    async fn save_user_feed(&self, user_id: Uuid, posts: &Vec<Post>) -> Result<(), Error> {
        let entries: Vec<(f64, String)> = posts
            .iter()
            .map(|p| (p.timestamp.timestamp() as f64, p.id.to_string()))
            .collect();

        if entries.is_empty() {
            return Ok(());
        }
        let cache_key = format!("highload/post/feed/ids/{}", user_id);
        self.pool.next().zadd(
            cache_key,
            None, 
            None,
            false,
            false,
            entries
        ).await
    }       

    async fn get_posts_by_ids(&self, ids: Vec<String>) -> Result<Vec<Post>, Error> {
        if ids.is_empty() {
            return Ok(vec!())
        } 
        let posts_data_store = "highload/post/post_by_id".to_string();
        self.pool.next().hmget::<Vec<Option<String>>, _, _>(posts_data_store, ids).await
            .map(|json_posts| 
                json_posts
                    .into_iter()
                    .flatten() 
                    .filter_map(|s| serde_json::from_str(&s).ok())
                    .collect()
            )        
    }

    async fn mark_feed_exists(&self, user_id: Uuid) -> Result<(), Error> {
        let feed_existence_marker_key = format!("highload/post/feed/exists/{}", user_id);        
        self.pool.next().set(
            &feed_existence_marker_key, 
            "1", 
            Some(Expiration::EX(3600)), 
            None, 
            false
        ).await  
    }    

    async fn check_feed_exists(&self, user_id: Uuid) -> Result<bool, Error> {
        let feed_existence_marker_key = format!("highload/post/feed/exists/{}", user_id);        
        self.pool.next().exists::<i64, _>(&feed_existence_marker_key).await.map(|count| count > 0)
    }     
}