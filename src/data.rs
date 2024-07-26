use sqlx::SqlitePool;

#[derive(Clone)]
pub struct DataPool {
    pub pool: SqlitePool,
}

#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub password: String,
}
