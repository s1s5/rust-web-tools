use async_graphql::{dataloader::DataLoader, Context};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
type Result<T> = anyhow::Result<T>;

pub struct Database {
    pub connection: Arc<DatabaseConnection>,
}

impl Database {
    pub async fn new_from_env() -> Result<Self> {
        let db = sea_orm::Database::connect(std::env::var("DATABASE_URL").unwrap())
            .await
            .expect("Could not connect to database");
        Ok(Self {
            connection: Arc::new(db),
        })
    }

    pub async fn new(connection: Arc<DatabaseConnection>) -> Self {
        Database { connection }
    }

    #[inline]
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}

pub fn get_data_loader_from_ctx(ctx: Context<'_>) -> &'_ DataLoader<Database> {
    ctx.data_unchecked::<DataLoader<Database>>()
}

pub fn get_db_from_ctx(ctx: Context<'_>) -> &'_ DatabaseConnection {
    get_data_loader_from_ctx(ctx).loader().get_connection()
}
