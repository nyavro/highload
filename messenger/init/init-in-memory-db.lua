box.cfg{
    listen = 3301,
}

box.once('init_v1', function()
    local dialogs = box.schema.space.create('dialogs', {
        if_not_exists = true,
        format = {
            {name = 'owner_id',   type = 'string'},
            {name = 'from_id',    type = 'string'},
            {name = 'to_id',      type = 'string'},
            {name = 'message_id', type = 'string'},
            {name = 'text',       type = 'string'},
            {name = 'created_at', type = 'number'},
            {name = 'updated_at', type = 'number'},
        }
    })
    dialogs:create_index('primary', {
        parts = {
            {field = 'owner_id',   type = 'string'},
            {field = 'to_id',      type = 'string'},
            {field = 'message_id', type = 'string'}
        },
        if_not_exists = true
    })
    dialogs:create_index('owner_to', {
        parts = {
            {field = 'owner_id', type = 'string'},
            {field = 'to_id',    type = 'string'}
        },
        unique = false,
        if_not_exists = true
    })
end)

function send_message(owner_id, from_id, to_id, message_id, text)
    local now = os.time()
    box.space.dialogs:insert{
        owner_id, 
        from_id, 
        to_id, 
        message_id, 
        text, 
        now, 
        now
    }
    return message_id
end

function list_dialogs(owner_id, to_id, limit, offset)
    local result = {}
    local tuples = box.space.dialogs.index.owner_to:select({owner_id, to_id})
    table.sort(tuples, function(a, b) return a[6] > b[6] end)    
    offset = offset or 0
    limit = limit or #tuples
    
    for i = offset + 1, math.min(offset + limit, #tuples) do
        local tuple = tuples[i]
        table.insert(result, {
            tuple[2],
            tuple[3],
            tuple[5]
        })
    end
    
    return result
end
