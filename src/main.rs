mod errors;
mod handler;

use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Object, Result as GraphQlResult, Schema,
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

struct Query;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn value(&self, ctx: &Context<'_>) -> GraphQlResult<i32> {
        tracing::info!("Starting value");
        let span = tracing::info_span!("Querying Postgres");
        let pool = dbg!(ctx.data_unchecked::<PgPool>());
        tracing::info!("Got pool");
        let row: (i32,) = sqlx::query_as("SELECT $1")
            .bind(150_i32)
            .fetch_one(pool)
            .instrument(span)
            .await
            .map_err(ServerError::from)?;
        tracing::info!("Finished query");
        Ok(row.0)
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
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    tracing::debug!("Attempting to query database");
    let row: (i32,) = sqlx::query_as("SELECT $1")
        .bind(150_i32)
        .fetch_one(&pool)
        .await?;
    tracing::info!("{}", row.0);
    Ok(pool)
}

async fn create_schema() -> Result<Schema<Query, EmptyMutation, EmptySubscription>, ServerError> {
    let pool = setup_database().await.map_err(ServerError::from)?;
    Ok(Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(pool)
        .finish())
}

#[cfg(test)]
mod tests {
    use crate::{setup_database, setup_tracing};

    #[tokio::test]
    async fn test_setup_database() {
        setup_database()
            .await
            .expect("Database could not be set up");
    }
}

#[cfg(test)]
mod test_query {
    use crate::create_schema;

    #[tokio::test]
    async fn test_value() {
        let schema = create_schema().await.unwrap();
        let res = schema.execute("{value}").await;
        assert_eq!(
            serde_json::to_string(&res).unwrap(),
            "{\"data\":{\"value\":150}}"
        );
    }
}
