CREATE TABLE dialogs(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_id UUID NOT NULL,  
    to_id UUID NOT NULL,  
    text VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,    
    CONSTRAINT fk_from_user
        FOREIGN KEY (from_id) 
        REFERENCES users(id) 
        ON DELETE CASCADE,
    CONSTRAINT fk_to_user
        FOREIGN KEY (from_id) 
        REFERENCES users(id) 
        ON DELETE CASCADE
);