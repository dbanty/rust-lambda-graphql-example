use std::str::FromStr;

use handler::handle_request;
use lambda_http::{run, Error};
use lambda_runtime::service_fn;
use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, Registry};

mod errors;
mod handler;
mod schema;

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_tracing();
    run(service_fn(handle_request)).await?;
    Ok(())
}

fn setup_tracing() {
    LogTracer::init().expect("Failed to set logger");

    let log_levels = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let env_filter =
        Targets::from_str(&log_levels).expect("Failed to parse log levels from RUST_LOG env var");
    let subscriber = Registry::default().with(env_filter);
    set_global_default(subscriber).expect("Failed to set subscriber");
}
