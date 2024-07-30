CREATE TABLE content (
    -- user who owns the content
    user_id INTEGER NOT NULL,
    -- unique content id
    content_id TEXT UNIQUE NOT NULL,
    -- when content is created
    created DATETIME NOT NULL,
    -- when content should expire, NULL if never
    expires DATETIME,
    -- content page title
    title TEXT UNIQUE NOT NULL,
    -- raw plaintext content
    content TEXT NOT NULL
);
