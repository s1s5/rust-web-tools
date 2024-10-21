use sea_orm::{ConnectionTrait, DatabaseConnection};
use std::sync::Arc;

use async_graphql::{dataloader::DataLoader, Context};

type Result<T> = anyhow::Result<T>;

pub struct Database<C> {
    pub connection: Arc<C>,
}

impl Database<DatabaseConnection> {
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
}

impl<C> Database<C>
where
    C: ConnectionTrait,
{
    pub fn new(connection: C) -> DataLoader<Self> {
        DataLoader::new(
            Self {
                connection: Arc::new(connection),
            },
            tokio::task::spawn,
        )
    }

    #[inline]
    pub fn get_connection(&self) -> &C {
        &self.connection
    }
}

pub fn get_data_loader_from_ctx<'a>(
    ctx: &Context<'a>,
) -> &'a DataLoader<Database<DatabaseConnection>> {
    ctx.data_unchecked::<DataLoader<Database<DatabaseConnection>>>()
}

pub fn get_db_from_ctx<'a>(ctx: &Context<'a>) -> &'a DatabaseConnection {
    get_data_loader_from_ctx(ctx).loader().get_connection()
}
