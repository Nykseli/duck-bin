use sqlx::SqlitePool;

#[derive(Clone)]
pub struct DataPool {
    pub pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub password: String,
}

#[derive(Debug)]
pub struct Content {
    /// Owner of the content
    pub user_id: i64,
    /// unique content id
    pub content_id: String,
    /// raw plaintext content
    pub content: String,
}
