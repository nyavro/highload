use uuid::Uuid;
use deadpool_postgres::{Object};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PostServiceError {
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),
    
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug)]
pub struct Post {
    pub id: Uuid,            
    pub text: String,
    pub author_user_id: Uuid
}


pub async fn create<'a>(client: Object, user_id: Uuid, text: &String) -> Result<Uuid, PostServiceError> {
    let res = client.query_one(
        "INSERT INTO posts (user_id, text) VALUES ($1, $2) RETURNING id", 
        &[&user_id, text]
    ).await?;
    let id: Uuid = res.get(0);    
    Ok(id)
}

pub async fn update(client: Object, user_id: Uuid, post_id: Uuid, text: &String) -> Result<(), PostServiceError> {
    let rows_affected = client.execute(
        "UPDATE posts SET text=$1,updated_at=NOW() WHERE user_id=$2 AND id=$3", 
        &[text, &user_id, &post_id]
    ).await?;    
    if rows_affected > 0 {
        Ok(())
    } else {
        Err(PostServiceError::Internal("Not updated".to_string()))
    }
}

pub async fn delete(client: Object, user_id: Uuid, post_id: Uuid) -> Result<(), PostServiceError> {
    let rows_affected = client.execute(
        "DELETE FROM posts WHERE user_id=$1 AND id=$2", 
        &[&user_id, &post_id]
    ).await?;    
    if rows_affected > 0 {
        Ok(())
    } else {
        Err(PostServiceError::Internal("Not updated".to_string()))
    }
}

pub async fn get(client: Object, post_id: Uuid) -> Result<Post, PostServiceError> {
    let res = client.query_one(
        "SELECT text,user_id FROM posts WHERE id=$1", 
        &[&post_id]
    ).await?;    
    let text: String = res.get(0);    
    let author_user_id: Uuid = res.get(1);    
    Ok(Post {id: post_id, text, author_user_id})
}

pub async fn feed(client: Object, user_id: Uuid, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Post>, PostServiceError> {
    let res = client.query(
        "SELECT p.text,p.user_id,p.id 
            FROM ((SELECT friend_id AS f_id FROM friends WHERE initiator_id=$1 AND NOT status='blocked') UNION (SELECT initiator_id AS f_id FROM friends WHERE friend_id=$1 AND NOT status='blocked')) q 
            JOIN posts p ON q.f_id = p.user_id ORDER BY p.created_at DESC LIMIT $2 OFFSET $3", 
        &[&user_id, &limit, &offset]
    ).await?;        
    Ok(
        res.iter().map(|row| {
            let text: String = row.get(0);    
            let author_user_id: Uuid = row.get(1);    
            let id: Uuid = row.get(2);
            Post {id, text, author_user_id}
        }).collect()
    )    
}