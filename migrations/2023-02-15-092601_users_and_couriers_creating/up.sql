CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    uuid UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    first_name TEXT NOT NULL,
    address TEXT,
    phone_number TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    role TEXT NOT NULL,
    is_blocked BOOLEAN DEFAULT false NOT NULL,
    is_deleted BOOLEAN DEFAULT false NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT USERS_ROLE_CHECK CHECK (role in ('USER', 'COURIER', 'ADMIN', 'ANALYST'))
);

CREATE TABLE couriers (
    user_uuid UUID  NOT NULL PRIMARY KEY,
    is_free BOOLEAN NOT NULL DEFAULT TRUE,
    rating FLOAT NOT NULL DEFAULT 5.0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT FK_USER
        FOREIGN KEY(user_uuid)
            REFERENCES users(uuid),
    CONSTRAINT CHECK_RATING
        check (rating between 0.0 and 5.0)
    
);

CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER set_timestamp_users
BEFORE UPDATE ON couriers
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

CREATE OR REPLACE TRIGGER set_timestamp_couriers
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();