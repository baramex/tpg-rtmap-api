use std::ops::Add;

use async_trait::async_trait;
use log::error;
use sqlx::postgres::{PgPool, PgPoolOptions, PgQueryResult, PgRow, PgValue};
use sqlx::{Error, Value};

#[async_trait]
pub trait Table {
    const TABLE_NAME: &'static str;

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error>;
    fn format(&self) -> String;
    fn keys() -> String;
    fn values(&self) -> Vec<String>; // TODO: change string: support numbers, dates, text etc (database types)
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
        let parts: std::slice::Chunks<'_, T> = data.chunks(100);

        for chunk in parts {
            println!("Chunk size: {}, formatting...", chunk.len());

            let mut i = 1;
            let mut str: String = String::new();
            let mut params: Vec<String> = Vec::new();
            for d in chunk {
                str = str.add("(");
                for v in d.values() {
                    str = str.add(&format!("${},", i));
                    params.push(v);
                    i += 1;
                }
                str.pop();
                str = str.add("),");
            }
            str.pop();

            let query: String = format!("INSERT INTO {} {} VALUES {} ON CONFLICT DO NOTHING", T::TABLE_NAME, T::keys(), str);
            println!("Query: {}", query);

            let mut final_query = sqlx::query(&query);

            println!("Binding params: {}...", params.len());
            for p in params {
                final_query = final_query.bind(p);
            }

            println!("Executing query...");
            final_query.execute(&self.pool).await?;
            println!("Inserted {} rows", chunk.len());
        }

        Ok(PgQueryResult::default())
    }
}
