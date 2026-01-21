use uuid::Uuid;
use deadpool_postgres::{Object};
use thiserror::Error;
use crate::modules::post::{repository::{PostRepositoryError, PostRepositoryImpl, PostRepository}, model::Post};

#[derive(Error, Debug)]
pub enum PostServiceError {
    #[error("Database error: {0}")]
    Database(#[from] PostRepositoryError)
}

pub trait PostService {
    async fn create<'a>(&self, user_id: Uuid, text: &String) -> Result<Uuid, PostServiceError>;
    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<(), PostServiceError>;
    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError>;
    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError>;
    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError>;
}

pub struct PostServiceImpl {
    repository: PostRepositoryImpl
}

impl PostServiceImpl {
    pub fn new(client: Object) -> PostServiceImpl {
        PostServiceImpl { repository: PostRepositoryImpl::new(client) }
    }
}

impl PostService for PostServiceImpl {    

    async fn create<'a>(&self, user_id: Uuid, text: &String) -> Result<Uuid, PostServiceError> {        
        let uuid = self.repository.create(user_id, text).await?;
        Ok(uuid)
    }

    async fn update(&self, user_id: Uuid, post_id: Uuid, text: &String) -> Result<(), PostServiceError> {
        let result = self.repository.update(user_id, post_id, text).await?;
        Ok(result)
    }

    async fn delete(&self, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError> {
        let result = self.repository.delete(user_id, post_id).await?;
        Ok(result)
    }

    async fn get(&self, post_id: Uuid) -> Result<Post, PostServiceError> {
        let post = self.repository.get(post_id).await?;
        Ok(post)
    }

    async fn feed(&self, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {
        let feed = self.repository.feed(user_id, limit, offset).await?;        
        Ok(feed)    
    }
}

