use log::error;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Error;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn init(config: PgPoolOptions, url: &str) -> Result<Database, Error> {
        Ok(Database {
            pool: config.connect(url).await?,
        })
    }

    pub async fn get<T>(&self, query: &str) -> Option<Vec<T>>
    where
        T: for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
    {
        let final_query = sqlx::query_as::<_, T>(query);
        let res: Result<Vec<T>, Error> = final_query.fetch_all(&self.pool).await;

        return match res {
            Ok(output) => match output.len() {
                0 => None,
                _ => Some(output),
            },
            Err(error) => {
                error!("Error: {:?}", error);
                None
            }
        };
    }

    pub async fn get_one<T>(&self, query: &str) -> Option<T>
    where
        T: for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
    {
        let rows: Option<Vec<T>> = self.get::<T>(query).await;
        
        return match rows {
            Some(mut rows) => match rows.len() {
                0 => None,
                _ => Some(rows.pop().unwrap()),
            },
            None => None,
        };
    }
}
