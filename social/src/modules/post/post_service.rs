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

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use uuid::Uuid;
    use crate::modules::post::{model::Post, post_cache::MockPostCache, repository::MockPostRepository};
    use crate::modules::friend::{repository::MockFriendRepository};    
    use chrono::Utc;

    #[tokio::test]
    async fn test_create_post_success() {
        let mut mock_repo = MockPostRepository::new();
        let mut mock_friends = MockFriendRepository::new();
        let mut mock_cache = MockPostCache::new();
        let u_id = Uuid::new_v4(); 
        let post_id = Uuid::new_v4();       
        mock_repo.expect_create()
            .with(eq(u_id), eq("Hello world".to_string()))
            .times(1)
            .returning(move |_, _| Ok(Post { id: post_id.clone(), text: "Hello world".to_string(), author_user_id: Uuid::new_v4(), timestamp: Utc::now()}));
        mock_friends.expect_get_followers_ids()
            .with(eq(u_id))
            .times(1)
            .returning(|_| Ok(vec![Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap()]));

        mock_cache.expect_save_post().returning(|_| Ok(()));
        mock_cache.expect_update_followers_feeds().returning(|_, _| Ok(()));        

        let result = PostServiceImpl::new(mock_repo, mock_friends, mock_cache).create(u_id, &"Hello world".to_string()).await;
        assert!(result.is_ok());
        let id = result.ok().unwrap();
        assert_eq!(id, post_id);
    }
}

