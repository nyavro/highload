CREATE TYPE friendship_status AS ENUM ('pending', 'subscriber', 'accepted', 'blocked');

CREATE TABLE friends(
    initiator_id UUID NOT NULL,  
    friend_id UUID NOT NULL,
    status friendship_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (initiator_id, friend_id),
    CONSTRAINT fk_initiator 
        FOREIGN KEY (initiator_id) 
        REFERENCES users(id) 
        ON DELETE CASCADE,
    CONSTRAINT fk_friend 
        FOREIGN KEY (friend_id) 
        REFERENCES users(id) 
        ON DELETE CASCADE
);