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
    Accepted,
    RequestSent,
    AlreadyExists
}
pub enum FriendshipEndResult {
    Subscribed,
    Blocked,
    NotInFriendship
}

pub async fn add_friend(mut client: Object, initiator_user_id: Uuid, user_id: Uuid) -> Result<FriendshipCreateResult, FriendServiceError> {         
    if initiator_user_id == user_id {
        return Err(FriendServiceError::IllegalState("Cannot add self as friend".to_string()));
    }
    let tx = client.transaction().await?;
    let rows_affected = tx.execute(
    "UPDATE friends SET status = 'accepted', updated_at = NOW() WHERE initiator_id = $1 AND friend_id = $2 AND status = 'pending'",
    &[&user_id, &initiator_user_id]
    ).await?;
    if rows_affected > 0 {    
        info!("Friendship request accepted");
        tx.commit().await;
        return Ok(FriendshipCreateResult::Accepted);
    }
    info!("Adding friendship request");
    let insert_res = tx.execute(
        "INSERT INTO friends (initiator_id, friend_id, status)
        VALUES ($1, $2, 'pending')
        ON CONFLICT (initiator_id, friend_id) DO NOTHING",
        &[&initiator_user_id, &user_id]
    ).await?;
    tx.commit().await?;
    if insert_res > 0 {
        info!("Friendship request added");
        Ok(FriendshipCreateResult::RequestSent)
    } else {
        info!("Friendship request already exists");
        Ok(FriendshipCreateResult::AlreadyExists)
    }
}

async fn turn_friend_to_subscriber(client: Object, initiator_user_id: Uuid, user_id: Uuid) -> Result<FriendshipEndResult, FriendServiceError> {
    let rows_affected = client.execute(
        "UPDATE friends SET status = 'subscriber', initiator_id = $2, updated_at = NOW() WHERE ((initiator_id = $1 AND friend_id = $2) OR (initiator_id = $2 AND friend_id = $1) AND status='accepted'", 
        &[&initiator_user_id, &user_id]
    ).await?;
    if rows_affected > 0 {
        return Ok(FriendshipEndResult::Subscribed);
    }
    return Ok(FriendshipEndResult::NotInFriendship);
}

async fn delete_and_block(client: Object, initiator_user_id: Uuid, user_id: Uuid) -> Result<FriendshipEndResult, FriendServiceError> {
    let rows_affected = client.execute(
        "UPDATE friends SET status = 'blocked', initiator_id = $1, updated_at = NOW() WHERE ((initiator_id = $1 AND friend_id = $2) OR (initiator_id = $2 AND friend_id = $1) AND NOT (status = 'blocked')", 
        &[&initiator_user_id, &user_id]
    ).await?;
    if rows_affected > 0 {
        return Ok(FriendshipEndResult::Blocked);
    }
    return Ok(FriendshipEndResult::NotInFriendship);
}

pub async fn delete_friend(client: Object, initiator_user_id: Uuid, user_id: Uuid, block: bool) -> Result<FriendshipEndResult, FriendServiceError> {         
    if block {
        delete_and_block(client, initiator_user_id, user_id).await
    } else {
        turn_friend_to_subscriber(client, initiator_user_id, user_id).await
    }    
}