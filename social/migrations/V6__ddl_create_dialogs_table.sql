CREATE TABLE dialogs(
    owner_id UUID NOT NULL,
    from_id UUID NOT NULL,  
    to_id UUID NOT NULL,     
    message_id UUID NOT NULL,
    text VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (owner_id, to_id, message_id)
);