mod errors;
mod handler;

use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use lamedh_http::{handler, Error};

use handler::GraphQlHandler;

struct Query;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    lamedh_runtime::run(handler(GraphQlHandler::new(Schema::new(
        Query,
        EmptyMutation,
        EmptySubscription,
    ))))
    .await?;
    Ok(())
}
