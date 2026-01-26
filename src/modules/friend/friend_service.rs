use uuid::Uuid;
use deadpool_postgres::{Object};
use thiserror::Error;
use log::info;

#[derive(Error, Debug)]
pub enum FriendServiceError {
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),
    
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Illegal state: {0}")]
    IllegalState(String),
}

pub enum FriendshipCreateResult {
    Mutual,
    Subscribed,
    AlreadyExists
}
pub enum FriendshipEndResult {
    Unsubscribed,
    Removed,
    NotInFriendship
}

pub async fn add_friend(mut client: Object, initiator_user_id: Uuid, user_id: Uuid) -> Result<FriendshipCreateResult, FriendServiceError> {         
    if initiator_user_id == user_id {
        return Err(FriendServiceError::IllegalState("Cannot add self as friend".to_string()));
    }
    let tx = client.transaction().await?;
    let res = tx.query_one(
        "SELECT count(*) FROM friends WHERE user_id = $2 AND friend_id = $1",
        &[&initiator_user_id, &user_id]
    ).await?;
    let count: i32 = res.get(0);    
    let rows_affected = tx.execute(
        "INSERT INTO friends (user_id, friend_id, status) VALUES($1, $2) ON CONFLICT (user_id, friend_id) DO NOTHING",
        &[&user_id, &initiator_user_id]
    ).await?;
    tx.commit().await?;
    if rows_affected > 0 {
        if count > 0 {
            info!("Friendship request accepted");
            return Ok(FriendshipCreateResult::Mutual);
        } else {
            info!("Subscribed to {}", user_id);
            return Ok(FriendshipCreateResult::Subscribed);
        }        
    }        
    else {
        info!("Friendship request already exists");
        return Ok(FriendshipCreateResult::AlreadyExists);
    }    
}

async fn unsubscribe(mut client: Object, initiator_user_id: Uuid, user_id: Uuid) -> Result<FriendshipEndResult, FriendServiceError> {
    let tx = client.transaction().await?;
    let rows_affected = tx.execute(
        "DELETE FROM friends WHERE user_id = $1 AND friend_id = $2", 
        &[&initiator_user_id, &user_id]
    ).await?;
    if rows_affected > 0 {
        return Ok(FriendshipEndResult::Unsubscribed);
    }
    return Ok(FriendshipEndResult::NotInFriendship);
}

async fn delete_and_block(client: Object, initiator_user_id: Uuid, user_id: Uuid) -> Result<FriendshipEndResult, FriendServiceError> {
    let rows_affected = client.execute(
        "DELETE FROM friends WHERE user_id = $1 AND friend_id = $2 OR user_id = $2 AND friend_id = $1", 
        &[&initiator_user_id, &user_id]
    ).await?;
    if rows_affected > 0 {
        return Ok(FriendshipEndResult::Removed);
    }
    return Ok(FriendshipEndResult::NotInFriendship);
}

pub async fn delete_friend(client: Object, initiator_user_id: Uuid, user_id: Uuid, block: bool) -> Result<FriendshipEndResult, FriendServiceError> {         
    if block {
        delete_and_block(client, initiator_user_id, user_id).await
    } else {
        unsubscribe(client, initiator_user_id, user_id).await
    }    
}