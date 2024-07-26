CREATE TABLE user_sessions (
    user_id INTEGER,
    session_id TEXT UNIQUE
);

CREATE TABLE users (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);

INSERT INTO users (name, password) VALUES ("duck", "foobar");
