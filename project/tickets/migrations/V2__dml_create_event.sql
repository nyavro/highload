INSERT INTO events (id, title, description, start_at)
VALUES ('00000000-0000-0000-0000-000000000004', 'Сhampions league final', 'Test event', NOW() + INTERVAL '30 days')
ON CONFLICT (id) DO NOTHING;

-- 40 * 20 * 50 = 40 000 seats.
INSERT INTO seats (id, event_id, sector, row_number, seat_number, price_cents)
SELECT 
    gen_random_uuid() as id,
    '00000000-0000-0000-0000-000000000004'::UUID as event_id,
    'Sector ' || sector_num as sector,
    row_num as row_number,
    seat_num as seat_number,
    CASE 
        WHEN sector_num <= 2 THEN 15000 
        WHEN sector_num <= 6 THEN 7500  
        ELSE 3500                       
    END as price_cents
FROM 
    generate_series(1, 10) as sector_num, -- 40 sectors
    generate_series(1, 20) as row_num,    -- 20 rows
    generate_series(1, 50) as seat_num    -- 50 seats per row
ON CONFLICT (event_id, sector, row_number, seat_number) DO NOTHING;