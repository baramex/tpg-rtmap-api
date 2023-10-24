use std::any::TypeId;
use std::ops::Add;

use async_trait::async_trait;
use log::error;
use sqlx::postgres::{PgPool, PgPoolOptions, PgQueryResult, PgRow, PgConnectOptions, PgArguments};
use sqlx::{Error, Postgres};
use sqlx::query::QueryAs;

#[async_trait]
pub trait Table {
    const TABLE_NAME: &'static str;

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error>;
    fn keys() -> String;
    fn values(&self) -> Vec<Box<dyn std::any::Any>>;
}

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn init(config: PgPoolOptions, connect_options: PgConnectOptions) -> Result<Database, Error> {
        Ok(Database {
            pool: config.connect_with(connect_options).await?,
        })
    }

    pub async fn query(&self, query: &str) -> Result<PgQueryResult, Error> {
        return sqlx::query(query).execute(&self.pool).await;
    }

    pub async fn get_many<T>(&self, query: QueryAs<'_, Postgres, T, PgArguments>) -> Option<Vec<T>>
    where
        T: for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
    {
        let res: Result<Vec<T>, Error> = query.fetch_all(&self.pool).await;

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

    pub async fn get_one<T>(&self, query: QueryAs<'_, Postgres, T, PgArguments>) -> Option<T>
    where
        T: for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
    {
        let res: Result<T, Error> = query.fetch_one(&self.pool).await;

        return match res {
            Ok(output) => Some(output),
            Err(error) => {
                error!("Error: {:?}", error);
                None
            }
        };
    }

    pub async fn insert_many<T>(&self, data: &Vec<T>) -> Result<PgQueryResult, Error>
    where
        T: serde::Serialize + Table,
    {
        let parts: std::slice::Chunks<'_, T> = data.chunks(10);

        for chunk in parts {
            let mut i = 1;
            let mut str: String = String::new();
            let mut params: Vec<Box<dyn std::any::Any>> = Vec::new();
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

            let query: String = format!(
                "INSERT INTO {} {} VALUES {} ON CONFLICT DO NOTHING",
                T::TABLE_NAME,
                T::keys(),
                str
            );

            let mut final_query = sqlx::query(&query);

            for value in params {
                if TypeId::of::<i32>() == value.type_id() {
                    let n: i32 = *value.downcast::<i32>().unwrap();
                    final_query = final_query.bind(n);
                } else if TypeId::of::<i16>() == value.type_id() {
                    let n: i16 = *value.downcast::<i16>().unwrap();
                    final_query = final_query.bind(n);
                } else if TypeId::of::<f64>() == value.type_id() {
                    let n: f64 = *value.downcast::<f64>().unwrap();
                    final_query = final_query.bind(n);
                } else if TypeId::of::<String>() == value.type_id() {
                    let n: String = *value.downcast::<String>().unwrap();
                    final_query = final_query.bind(n);
                }
            }

            final_query.execute(&self.pool).await?;
        }

        Ok(PgQueryResult::default())
    }
}
