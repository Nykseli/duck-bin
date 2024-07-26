CREATE TABLE users (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);

INSERT INTO users (name, password) VALUES ("duck", "foobar");
