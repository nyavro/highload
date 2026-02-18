CREATE TABLE friends(
    user_id UUID NOT NULL,  
    friend_id UUID NOT NULL,    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, friend_id),
    CONSTRAINT fk_user
        FOREIGN KEY (user_id) 
        REFERENCES users(id) 
        ON DELETE CASCADE,
    CONSTRAINT fk_friend 
        FOREIGN KEY (friend_id) 
        REFERENCES users(id) 
        ON DELETE CASCADE
);