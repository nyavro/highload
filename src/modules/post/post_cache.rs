use fred::prelude::Pool;
use uuid::Uuid;
use crate::modules::post::{model::Post};
use fred::prelude::{SortedSetsInterface};
use fred::prelude::*;
use fred::error::Error;
use std::sync::Arc;
use async_trait::async_trait; 
use mockall::automock;

const DEFAULT_FEED_SIZE: i64 = 1000;
const POST_CACHE_TTL_SECONDS: i64 = 86400;

#[automock]
#[async_trait]
pub trait FeedCache {
    async fn update_followers_feeds(&self, followers_ids: Vec<Uuid>, post: &Post) -> Result<(), Error>;    
    async fn get_user_feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<String>, Error>;
    async fn save_user_feed(&self, user_id: Uuid, posts: &Vec<Post>) -> Result<(), Error>;
}

#[automock]
#[async_trait]
pub trait UserPostCache {
    async fn get_posts_by_ids(&self, ids: Vec<String>) -> Result<Vec<Post>, Error>;        
    async fn save_post(&self, post: &Post) -> Result<(), Error>;
    async fn save_posts(&self, posts: &Vec<Post>) -> Result<(), Error>;
    async fn delete_post(&self, post_id: &Uuid) -> Result<(), Error>;
    async fn get_post(&self, post_id: &Uuid) -> Result<Option<Post>, Error>;
}

#[automock]
#[async_trait]
pub trait MarkCache {
    async fn mark_feed_exists(&self, user_id: Uuid) -> Result<(), Error>;
    async fn check_feed_exists(&self, user_id: Uuid) -> Result<bool, Error>;
}

#[async_trait]
pub trait PostCache: FeedCache + UserPostCache + MarkCache {}

impl<T> PostCache for T where T: FeedCache + UserPostCache + MarkCache {}

pub struct PostCacheImpl {
    pool: Arc<Pool>
} 

impl PostCacheImpl {
    pub fn new(pool: Arc<Pool>) -> Self {
        PostCacheImpl { pool }
    }
}


impl PostCacheImpl {
    fn get_feed_key(&self, user_id: &Uuid) -> String {
        format!("highload/post/feed/ids/{}", user_id)
    }
}

#[async_trait]
impl FeedCache for PostCacheImpl {
   
    async fn update_followers_feeds(&self, followers_ids: Vec<Uuid>, post: &Post) -> Result<(), Error> {      
        let pipeline = self.pool.next().pipeline();        
        for follower_id in &followers_ids {            
            let _ = pipeline.zadd::<(), _, _>(self.get_feed_key(follower_id), None, None, false, false, (post.timestamp.timestamp() as f64, post.id.to_string())).await;
        }
        pipeline.last().await
    }

    async fn get_user_feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<String>, Error> {        
        let start = offset.unwrap_or(0) as i64;
        let stop = start + (limit.unwrap_or(DEFAULT_FEED_SIZE) as i64) - 1;
        self.pool.next().zrevrange::<Vec<String>, _>(self.get_feed_key(&user_id), start, stop, false).await
    }

    async fn save_user_feed(&self, user_id: Uuid, posts: &Vec<Post>) -> Result<(), Error> {
        let entries: Vec<(f64, String)> = posts
            .iter()
            .map(|p| (p.timestamp.timestamp() as f64, p.id.to_string()))
            .collect();

        if entries.is_empty() {
            return Ok(());
        }        
        self.pool.next().zadd(
            self.get_feed_key(&user_id),
            None, 
            None,
            false,
            false,
            entries
        ).await
    }  
}

impl PostCacheImpl {
    fn get_post_key(&self, post_id: &String) -> String {
        format!("highload/post:{}", post_id)
    }
}

#[async_trait]
impl UserPostCache for PostCacheImpl {
    
    async fn save_post(&self, post: &Post) -> Result<(), Error> {                                    
        self.pool.next().set(
            self.get_post_key(&post.id.to_string()),
            serde_json::to_string(&post).map_err(|_| Error::new(ErrorKind::Parse, "Serde error"))?,
            Some(Expiration::EX(POST_CACHE_TTL_SECONDS)),
            None,
            false
        ).await          
    }   

    async fn save_posts(&self, posts: &Vec<Post>) -> Result<(), Error> {
        let mut entries = Vec::with_capacity(posts.len());
        for post in posts {
            let json = serde_json::to_string(post)
                .map_err(|e| Error::new(fred::error::ErrorKind::Parse, e.to_string()))?;
            entries.push((self.get_post_key(&post.id.to_string()), json));
        }

        self.pool.next().mset(entries).await
    }

    async fn get_post(&self, post_id: &Uuid) -> Result<Option<Post>, Error> {            
        let item_key = self.get_post_key(&post_id.to_string());  
        let maybe_json: Option<String> = self.pool.next().get(item_key).await?;                      
        match maybe_json {
            Some(json) => {
                let post = serde_json::from_str(&json)
                    .map_err(|e| Error::new(fred::error::ErrorKind::Parse, e.to_string()))?;
                Ok(Some(post))
            }
            None => Ok(None),
        }
    }   

    async fn delete_post(&self, post_id: &Uuid) -> Result<(), Error> {        
        self.pool.next().del(self.get_post_key(&post_id.to_string())).await
    }

    async fn get_posts_by_ids(&self, ids: Vec<String>) -> Result<Vec<Post>, Error> {
        if ids.is_empty() {
            return Ok(vec!())
        }         
        let keys: Vec<String> = ids.iter().map(|id| self.get_post_key(id)).collect();
        let json_posts: Vec<Option<String>> = self.pool.next().mget(keys).await?;
        
        Ok(json_posts.into_iter()
            .flatten()
            .filter_map(|s| serde_json::from_str(&s).ok())
            .collect())
    }   
}

impl PostCacheImpl {
    fn get_mark_key(&self, user_id: &Uuid) -> String {
        format!("highload/post/feed/exists/{}", user_id)
    }
}

#[async_trait]
impl MarkCache for PostCacheImpl {
   
    async fn mark_feed_exists(&self, user_id: Uuid) -> Result<(), Error> {          
        self.pool.next().set(
            &self.get_mark_key(&user_id), 
            "1", 
            Some(Expiration::EX(3600)), 
            None, 
            false
        ).await  
    }    

    async fn check_feed_exists(&self, user_id: Uuid) -> Result<bool, Error> {        
        self.pool.next().exists::<i64, _>(&self.get_mark_key(&user_id)).await.map(|count| count > 0)
    }     
}