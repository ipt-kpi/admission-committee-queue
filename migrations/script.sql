CREATE TABLE IF NOT EXISTS enrollee (
    id BIGINT PRIMARY KEY,
    username VARCHAR(255),
    name VARCHAR(255),
    patronymic VARCHAR(255),
    last_name VARCHAR(255),
    phone_number VARCHAR(13),
    banned BOOLEAN
);

CREATE TABLE IF NOT EXISTS queue (
    enrollee BIGINT REFERENCES enrollee (id),
    date DATE,
    time TIME,
    processed BOOLEAN,
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

DROP FUNCTION is_enrollee_valid(last_name_t VARCHAR, name_t VARCHAR, patronymic_t VARCHAR);
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
    id BIGINT,
    date DATE,
    time_t TIME
) RETURNS bool
AS $$
    DECLARE
        exists bool;
BEGIN
    exists := exists(SELECT 1 FROM queue WHERE enrollee = id);
    INSERT INTO queue(enrollee, date, time)
    VALUES (id, date, time_t)
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

CREATE TABLE IF NOT EXISTS refresh_sessions (
    id SERIAL PRIMARY KEY,
    user_id SERIAL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token UUID NOT NULL DEFAULT uuid_generate_v4(),
    fingerprint VARCHAR(200) NOT NULL,
    expires_in BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);