use std::ops::Add;

use async_trait::async_trait;
use log::error;
use sqlx::postgres::{PgPool, PgPoolOptions, PgQueryResult, PgRow};
use sqlx::Error;

#[async_trait]
pub trait Table {
    const TABLE_NAME: &'static str;

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error>;
    fn format(&self) -> String;
    fn keys() -> String;
}

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

    pub async fn query(&self, query: &str) -> Result<PgQueryResult, Error> {
        return sqlx::query(query).execute(&self.pool).await;
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

    pub async fn insert_one<T>(&self, data: T) -> Result<PgQueryResult, Error>
    where
        T: serde::Serialize + Table,
    {
        let query = format!("INSERT INTO {} {} VALUES {}", T::TABLE_NAME, T::keys(), data.format());

        let final_query = sqlx::query(&query);

        return final_query.execute(&self.pool).await;
    }

    pub async fn insert_many<T>(&self, data: &Vec<T>) -> Result<PgQueryResult, Error>
    where
        T: serde::Serialize + Table,
    {
        let parts: std::slice::Chunks<'_, T> = data.chunks(1000);

        for chunk in parts {
            let mut params: String = String::new();
            for d in chunk {
                params = params.add(&(d.format() + ","));
            }
            params.pop();

            let query: String = format!("INSERT INTO {} {} VALUES {}", T::TABLE_NAME, T::keys(), params);

            let final_query = sqlx::query(&query);
            
            final_query.execute(&self.pool).await?;
        }

        Ok(PgQueryResult::default())
    }
}
