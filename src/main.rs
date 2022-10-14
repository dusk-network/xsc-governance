#[macro_use]
extern crate dotenv_codegen;
/// Graphql configuration goes here. The schema, queries and mutations.
mod graphql;
/// Data structures which are sent over the wire and the database.
mod models;

use crate::graphql::{graphiql, graphql_handler, schema::QueryRoot};

use async_graphql::*;
use axum::{extract::Extension, routing::get, Router, Server};
use dotenv::dotenv;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use tracing::info;

pub type SqliteConnection = PooledConnection<SqliteConnectionManager>;

async fn sqlite() -> Result<Pool<SqliteConnectionManager>> {
    let manager = SqliteConnectionManager::file(dotenv!("SQLITE_DB"));
    let pool = r2d2::Pool::new(manager)?;

    Ok(pool)
}

#[tokio::main]
async fn main() {
    env_logger::init();

    dotenv().ok();

    let sqlite = sqlite().await.expect("cannot connect to sqlite");

    info!("Connected to sqlite at: {}", dotenv!("SQLITE_DB"));

    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(sqlite)
        .finish();

    let x = schema.execute("{ transfer(blockHeight: 1) }").await;
    println!("{}", x.data);

    let app = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .layer(Extension(schema));

    info!("GraphiQL IDE at: http://{}", dotenv!("GRAPHQL_ENDPOINT"));
    info!("Starting server at: {}", dotenv!("GRAPHQL_ENDPOINT"));

    Server::bind(&dotenv!("GRAPHQL_ENDPOINT").parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
