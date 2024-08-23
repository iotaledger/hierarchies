use axum::Router;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
  tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "example_templates=debug".into()))
    .with(tracing_subscriber::fmt::layer())
    .init();

  // build our application with some routes
  let app = Router::new();

  // run it
  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
  tracing::debug!("listening on {}", listener.local_addr().unwrap());
  axum::serve(listener, app).await.unwrap();
}
