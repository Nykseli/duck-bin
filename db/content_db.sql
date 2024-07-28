CREATE TABLE content (
    -- user who owns the content
    user_id INTEGER NOT NULL,
    -- unique content id
    content_id TEXT UNIQUE NOT NULL,
    -- raw plaintext content
    content TEXT NOT NULL
);
