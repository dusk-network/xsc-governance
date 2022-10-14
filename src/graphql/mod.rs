pub mod schema;

use self::schema::QueryRoot;

use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{self, IntoResponse},
};

pub type TransactionSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub async fn graphql_handler(
    schema: Extension<TransactionSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphiql() -> impl IntoResponse {
    let endpoint = format!("http://{}", dotenv!("GRAPHQL_ENDPOINT"));
    response::Html(GraphiQLSource::build().endpoint(&endpoint).finish())
}
