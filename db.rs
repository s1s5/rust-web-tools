use sea_orm::DatabaseConnection;
use std::sync::Arc;

use async_graphql::{dataloader::DataLoader, Context};

type Result<T> = anyhow::Result<T>;

pub struct Database {
    pub connection: Arc<DatabaseConnection>,
}

impl Database {
    pub async fn new_from_env() -> Result<DataLoader<Self>> {
        Ok(DataLoader::new(
            Self {
                connection: Arc::new(
                    sea_orm::Database::connect(std::env::var("DATABASE_URL")?).await?,
                ),
            },
            tokio::task::spawn,
        ))
    }

    #[inline]
    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}

pub fn get_data_loader_from_ctx<'a>(ctx: &Context<'a>) -> &'a DataLoader<Database> {
    ctx.data_unchecked::<DataLoader<Database>>()
}

pub fn get_db_from_ctx<'a>(ctx: &Context<'a>) -> &'a DatabaseConnection {
    get_data_loader_from_ctx(ctx).loader().get_connection()
}
