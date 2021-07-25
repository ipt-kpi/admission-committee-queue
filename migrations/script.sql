CREATE TABLE IF NOT EXISTS enrollee (
    id SERIAL PRIMARY KEY,
    chat_id BIGINT UNIQUE,
    username VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    patronymic VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    phone_number VARCHAR(13) NOT NULL,
    banned BOOLEAN NOT NULL DEFAULT FALSE,
    notification BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TYPE status AS ENUM ('wait', 'processed', 'absent');

CREATE TABLE IF NOT EXISTS queue (
    enrollee INTEGER REFERENCES enrollee(id) PRIMARY KEY,
    date DATE NOT NULL,
    time TIME NOT NULL,
    status status DEFAULT 'wait'::status,
    UNIQUE (date, time)
);

CREATE TABLE IF NOT EXISTS parsed_names(name VARCHAR(255));

CREATE TABLE IF NOT EXISTS teloxide_dialogues (
    chat_id BIGINT PRIMARY KEY,
    dialogue BYTEA NOT NULL
);

CREATE OR REPLACE FUNCTION get_relevant_time(
    start_date date,
    start_time time,
    max_enrollee integer,
    wait_time interval
) RETURNS SETOF time
AS $$
    DECLARE start_timestamp timestamp := start_date + start_time;
BEGIN
    RETURN QUERY
        SELECT time::time FROM generate_series(
            start_timestamp,
            start_timestamp + (wait_time * (max_enrollee - 1)),
            wait_time
        ) AS time WHERE time::time NOT IN (SELECT time FROM queue WHERE date = start_date);
END $$  LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_intervals(
    start_date date,
    start_time time,
    max_enrollee integer,
    wait_time interval
) RETURNS SETOF text
AS $$
BEGIN
    RETURN QUERY
        SELECT concat(intervals.interval_time, ':00-', intervals.interval_time + 1, ':00') FROM (
            SELECT COUNT(*), date_part('hour', time) as interval_time FROM get_relevant_time(
                start_date,
                start_time,
                max_enrollee,
                wait_time
            ) AS time
            GROUP BY interval_time
            ORDER BY interval_time
        ) as intervals WHERE intervals.count > 0;
END $$  LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION is_enrollee_valid(
    last_name_t VARCHAR(255),
    name_t VARCHAR(255),
    patronymic_t VARCHAR(255)
) RETURNS bool
AS $$
    DECLARE
        name_t VARCHAR(1) := left(name_t, 1);
        patronymic_t VARCHAR(1) := left(patronymic_t, 1);
BEGIN
    RETURN (
        SELECT NOT
            exists(SELECT 1 FROM enrollee WHERE left(name, 1) = name_t AND left(patronymic, 1) = patronymic_t AND last_name = last_name_t)
            AND
            exists(SELECT 1 FROM parsed_names WHERE name = concat_ws(' ', last_name_t, name_t || '.', patronymic_t || '.'))
    );
END $$  LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION register_in_queue(
    chat_id_t BIGINT,
    date DATE,
    time_t TIME
) RETURNS bool
AS $$
DECLARE
    exists bool = exists(SELECT 1 FROM queue JOIN enrollee e on e.id = queue.enrollee WHERE chat_id = chat_id_t);
    enrollee_id INTEGER;
BEGIN
    SELECT id INTO enrollee_id FROM enrollee WHERE chat_id = chat_id_t;
    INSERT INTO queue(enrollee, date, time)
    VALUES (enrollee_id, date, time_t)
    ON CONFLICT(enrollee) DO UPDATE SET date = excluded.date, time = excluded.time;
    RETURN exists;
END $$  LANGUAGE plpgsql;

CREATE TYPE role AS ENUM ('user', 'admin');

CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(16) NOT NULL,
    email VARCHAR(64) NOT NULL,
    password VARCHAR(60) NOT NULL,
    role role DEFAULT 'user'::role
);

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

SET TIMEZONE='Europe/Kiev';

CREATE TABLE IF NOT EXISTS refresh_sessions (
    id SERIAL PRIMARY KEY,
    user_id SERIAL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token UUID NOT NULL DEFAULT uuid_generate_v4(),
    fingerprint VARCHAR(200) NOT NULL,
    expires_in BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION notify_status() RETURNS TRIGGER AS $$
DECLARE
    record RECORD;
    count INTEGER;
BEGIN
    IF CURRENT_DATE = NEW.date AND OLD.status = 'wait' THEN
        FOR record IN
            SELECT * FROM queue WHERE date = CURRENT_DATE AND status = 'wait'
        LOOP
            IF record.time >= NEW.time THEN
                SELECT COUNT(*) INTO count FROM queue WHERE enrollee != record.enrollee AND date = record.date AND status = 'wait' AND time < record.time;
                IF (SELECT notification FROM enrollee WHERE id = record.enrollee) OR count = 5 OR count = 1 OR count = 0 THEN
                    PERFORM pg_notify('queue_status', row_to_json(row(record.enrollee, count))::text);
                END IF;
            END IF;
        END LOOP;
    END IF;
        RETURN NULL;
    END
$$ LANGUAGE plpgsql;

CREATE TRIGGER queue_notify
AFTER INSERT OR UPDATE OR DELETE ON queue
    FOR EACH ROW EXECUTE PROCEDURE notify_status();


--import names
COPY parsed_names(name) FROM 'path' CSV;

--export queue
COPY (
    SELECT last_name, name, patronymic, date, time, phone_number, username, status, id, chat_id
    FROM queue JOIN enrollee e on e.chat_id = queue.enrollee
) TO 'path' DELIMITER ',' CSV HEADER;
