use sqlx::MySqlPool;

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: MySqlPool,
}
