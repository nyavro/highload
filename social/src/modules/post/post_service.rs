use uuid::Uuid;
use crate::modules::post::{model::Post, repository::PostRepository, service_provider::{PostService, PostServiceError}};
use async_trait::async_trait; 

pub struct PostServiceImpl<R> 
where
    R: PostRepository {
    repository: R
} 

impl <R> PostServiceImpl<R>
where 
    R: PostRepository {    
    pub fn new(repository: R) -> Self {
        PostServiceImpl { 
            repository
        }
    }

    async fn fetch_from_db(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {
        let feed = self.repository.feed(user_id, limit, offset).await?;        
        Ok(feed)
    }            
}

#[async_trait]
impl <R> PostService for PostServiceImpl<R> 
where
    R: PostRepository + Send + Sync {    
    async fn create(&self, user_id: Uuid, text: &String) -> Result<Post, PostServiceError> {                
        Ok(self.repository.create(user_id, text).await?)
    }

    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<Post, PostServiceError> {
        Ok(self.repository.update(user_id, post_id, text).await?)
    }

    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError> {        
        Ok(self.repository.delete(user_id, post_id).await?)
    }
  
    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError> {         
        Ok(self.repository.get(post_id).await?)
    }

    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {                  
        Ok(self.fetch_from_db(user_id, limit, offset).await?)      
    }
}    
