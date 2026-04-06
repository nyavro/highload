box.cfg{
    listen = 3301
}

box.schema.space.create('test_space')

box.space.test_space:create_index('primary', {type = 'hash', parts = {1, 'unsigned'}})