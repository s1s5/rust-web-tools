use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, FromQueryResult, PaginatorTrait, QuerySelect, Select,
};
type Result<T> = std::result::Result<T, DbErr>;

pub struct Edge<T> {
    pub node: T,
    pub cursor: u64,
}
pub struct Connection<T> {
    pub edges: Vec<Edge<T>>,
    pub has_previous_page: bool,
    pub has_next_page: bool,
}

pub enum Range {
    Forward((u64, u64)),
    Backward((u64, Option<u64>)),
}

impl Range {
    // pub fn default() -> Range {
    //     Range::Forward((20, 0))
    // }

    pub fn new(
        first: Option<u64>,
        last: Option<u64>,
        after: Option<u64>,
        before: Option<u64>,
    ) -> anyhow::Result<Range> {
        if let Some(first) = first {
            if last.is_some() {
                Err(anyhow::anyhow!(
                    "first and last must not be set at the same time",
                ))
            } else if before.is_some() {
                Err(anyhow::anyhow!(
                    "first or before must not be set at the same time",
                ))
            } else {
                Ok(Range::Forward((first, after.unwrap_or(0))))
            }
        } else if let Some(last) = last {
            if after.is_some() {
                Err(anyhow::anyhow!(
                    "last or after must not be set at the same time",
                ))
            } else {
                Ok(Range::Backward((last, before)))
            }
        } else {
            Err(anyhow::anyhow!("first or last must be set"))
        }
    }

    async fn forward<E: EntityTrait>(
        &self,
        db: &DatabaseConnection,
        qs: Select<E>,
        first: u64,
        after: u64,
    ) -> Result<Connection<E::Model>> {
        let mut records = qs.offset(after).limit(first + 1).all(db).await?;
        let has_next_page = if records.len() > (first as usize) {
            records.pop();
            true
        } else {
            false
        };
        Ok(Connection {
            edges: records
                .into_iter()
                .enumerate()
                .map(|x| Edge {
                    node: x.1,
                    cursor: (x.0 as u64) + after,
                })
                .collect(),
            has_previous_page: after > 0,
            has_next_page,
        })
    }

    async fn backward<'db, E, M>(
        &self,
        db: &'db DatabaseConnection,
        qs: Select<E>,
        last: u64,
        before: Option<u64>,
    ) -> Result<Connection<M>>
    where
        E: EntityTrait<Model = M>,
        M: FromQueryResult + Sized + Send + Sync + 'db,
    {
        let count = qs.clone().paginate(db, 1).num_items().await?;
        let before = before.map(|x| std::cmp::min(x, count)).unwrap_or(count);
        let end_cursor = std::cmp::max(0, before - last - 1);
        let records = qs.offset(end_cursor).limit(last).all(db).await?;
        Ok(Connection {
            edges: records
                .into_iter()
                .enumerate()
                .map(|x| Edge {
                    node: x.1,
                    cursor: (x.0 as u64) + end_cursor,
                })
                .collect(),
            has_previous_page: end_cursor > 0,
            has_next_page: before < count,
        })
    }

    pub async fn get_connection<'db, E, M>(
        &self,
        db: &'db DatabaseConnection,
        qs: Select<E>,
    ) -> Result<Connection<M>>
    where
        E: EntityTrait<Model = M>,
        M: FromQueryResult + Sized + Send + Sync + 'db,
    {
        match self {
            Self::Forward((first, after)) => self.forward(db, qs, *first, *after).await,
            Self::Backward((last, before)) => self.backward(db, qs, *last, *before).await,
        }
    }
}
