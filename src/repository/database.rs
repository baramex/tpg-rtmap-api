use std::any::TypeId;
use std::ops::Add;
use std::time::SystemTime;

use async_trait::async_trait;
use log::error;
use sqlx::postgres::{PgPool, PgPoolOptions, PgQueryResult, PgRow, PgConnectOptions};
use sqlx::Error;

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

    pub async fn insert_many<T>(&self, data: &Vec<T>) -> Result<PgQueryResult, Error>
    where
        T: serde::Serialize + Table,
    {
        /*
        46s -> 1000
        16s -> 500
        700ms -> 100
        200ms -> 50
        65ms -> 25
        17ms -> 10 <-- best
        9ms -> 5
        4ms -> 1
        */
        let parts: std::slice::Chunks<'_, T> = data.chunks(10);

        for chunk in parts {
            let start: SystemTime = SystemTime::now();
            println!("Chunk size: {}, formatting...", chunk.len());

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

            println!("Binding params: {}...", params.len());
            for value in params {
                if TypeId::of::<i32>() == value.type_id() {
                    let n: i32 = *value.downcast::<i32>().unwrap();
                    println!("Binding i32: {}", n);
                    final_query = final_query.bind(n);
                } else if TypeId::of::<i16>() == value.type_id() {
                    let n: i16 = *value.downcast::<i16>().unwrap();
                    println!("Binding i16: {}", n);
                    final_query = final_query.bind(n);
                } else if TypeId::of::<i8>() == value.type_id() {
                    let n: i8 = *value.downcast::<i8>().unwrap();
                    println!("Binding i8: {}", n);
                    final_query = final_query.bind(n);
                } else if TypeId::of::<f64>() == value.type_id() {
                    let n: f64 = *value.downcast::<f64>().unwrap();
                    println!("Binding f64: {}", n);
                    final_query = final_query.bind(n);
                } else if TypeId::of::<String>() == value.type_id() {
                    let n: String = *value.downcast::<String>().unwrap();
                    println!("Binding String: {}", n);
                    final_query = final_query.bind(n);
                }
            }

            println!("Executing query...");
            final_query.execute(&self.pool).await?;
            println!(
                "Inserted {} rows in {} ms",
                chunk.len(),
                start.elapsed().unwrap().as_millis()
            );
        }

        Ok(PgQueryResult::default())
    }
}
