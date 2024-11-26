use async_graphql::{dataloader::DataLoader, Context};
use sea_orm::{
    prelude::Expr, sea_query::CaseStatement, ActiveModelTrait, ColumnTrait, Condition,
    ConnectionTrait, DatabaseConnection, EntityTrait, Iterable, PrimaryKeyToColumn, QueryFilter,
    UpdateMany,
};
use std::sync::Arc;

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

/// batch update
/// bulk_update(&[a], &[Column::Name]).exec(conn).await?;
pub fn bulk_update<A>(
    objects: &[A],
    fields: &[<A::Entity as EntityTrait>::Column],
) -> UpdateMany<A::Entity>
where
    A: ActiveModelTrait,
{
    let mut cond_list = vec![];
    let mut filter_cond = Condition::any();
    for obj in objects.iter() {
        let mut cond = Condition::all();
        for pk in <A::Entity as EntityTrait>::PrimaryKey::iter() {
            cond = cond.add(pk.into_column().eq(obj.get(pk.into_column()).unwrap()));
        }
        cond_list.push(cond.clone());
        filter_cond = filter_cond.add(cond);
    }

    let mut qs = <A::Entity as EntityTrait>::update_many().filter(filter_cond);
    for f in fields {
        let case_statement = objects
            .iter()
            .zip(cond_list.iter())
            .fold(CaseStatement::new(), |a, (e, c)| {
                a.case(c.clone(), Expr::val(e.get(*f).unwrap()))
            });
        qs = qs.col_expr(*f, case_statement.into())
    }

    qs
}
