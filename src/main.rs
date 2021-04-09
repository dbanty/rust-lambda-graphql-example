mod errors;
mod handler;

use async_graphql::{
    Context, EmptySubscription, Object, Result as GraphQlResult, Schema, SimpleObject,
};
use lamedh_http::{handler, Error};
use sqlx::postgres::PgPoolOptions;

use crate::errors::ServerError;
use handler::GraphQlHandler;
use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_futures::Instrument;
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

#[derive(SimpleObject)]
struct User {
    id: i32,
    name: String,
}

struct Query;

#[Object]
impl Query {
    async fn users(&self, ctx: &Context<'_>) -> GraphQlResult<Vec<User>> {
        let span = tracing::info_span!("Querying users");

        let pool = ctx.data_unchecked::<PgPool>();
        let users: Vec<User> = sqlx::query_as!(User, "SELECT id, name FROM users")
            .fetch_all(pool)
            .instrument(span)
            .await
            .map_err(ServerError::from)?;
        tracing::info!("Finished query");
        Ok(users)
    }
}

struct Mutation;

#[Object]
impl Mutation {
    async fn create_user(&self, ctx: &Context<'_>, name: String) -> GraphQlResult<i32> {
        let span = tracing::info_span!("Querying users");

        let pool = ctx.data_unchecked::<PgPool>();
        let result: i32 =
            sqlx::query_scalar!("INSERT INTO users (name) VALUES ($1) RETURNING id", name)
                .fetch_one(pool)
                .instrument(span)
                .await
                .map_err(ServerError::from)?;
        tracing::info!("Finished query");
        Ok(result)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    setup_tracing();
    let gql_handler = GraphQlHandler::new(create_schema().await?);

    lamedh_runtime::run(handler(gql_handler)).await?;
    Ok(())
}

fn setup_tracing() {
    LogTracer::init().expect("Failed to set logger");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(
        "GraphQL".into(),
        // Output the formatted spans to stdout.
        std::io::stdout,
    );
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
}

async fn setup_database() -> Result<PgPool, sqlx::Error> {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    tracing::debug!("Got {} as DB URL", &db_url);
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
}

async fn create_schema() -> Result<Schema<Query, Mutation, EmptySubscription>, ServerError> {
    let pool = setup_database().await.map_err(ServerError::from)?;
    Ok(Schema::build(Query, Mutation, EmptySubscription)
        .data(pool)
        .finish())
}

#[cfg(test)]
mod tests {
    use crate::setup_database;

    #[tokio::test]
    async fn test_setup_database() {
        setup_database()
            .await
            .expect("Database could not be set up");
    }
}

#[cfg(test)]
mod test_users {
    use serde_json::json;

    use crate::create_schema;

    #[tokio::test]
    async fn test_create_and_get_user() {
        let schema = create_schema().await.unwrap();
        let res = schema.execute("mutation {createUser(name: \"Bob\")}").await;
        let id = res.data.into_json().expect("Result was not JSON")["createUser"]
            .as_u64()
            .expect("Result was not u64");
        let res = schema
            .execute("{users{id, name}}")
            .await
            .data
            .into_json()
            .unwrap();
        assert_eq!(res, json!({"users": [{"name": "Bob", "id": id}]}))
    }
}
