use handler::handle_request;
use lambda_http::{run, Error};
use lambda_runtime::service_fn;
use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

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

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = Registry::default().with(env_filter);
    set_global_default(subscriber).expect("Failed to set subscriber");
}
