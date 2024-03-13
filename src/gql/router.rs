use tokio::signal;
use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use crate::gql::studies::{QueryRoot, MutationRoot};

/// Adds GraphiQL as a middleware for testing out queries and mutations.
async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/gql").finish())
}

/// Starts the Axum web server with the GraphQL schema.
///
/// This function initializes the Axum web server to listen on a given TCP listener
/// and serves the GraphQL API. It sets up routes for both the GraphiQL IDE and the GraphQL
/// endpoint itself. The server runs with graceful shutdown enabled, allowing it to
/// cleanly shut down when a shutdown signal is received.
///
/// # Arguments
///
/// * `listener` - A `TcpListener` that the server will accept connections on.
///
pub async fn start_axum(listener: TcpListener) {

    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .finish();

    let app = Router::new().route("/gql", get(graphiql).post_service(GraphQL::new(schema)));

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap_or_else(|e| panic!("failed to run `axum::serve`: {e}"));
}

/// Listens for shutdown signals to gracefully terminate the application.
///
/// This asynchronous function sets up signal handlers to listen for Ctrl+C (SIGINT) on all platforms,
/// and for SIGTERM on Unix-based systems. It then awaits any of these signals before proceeding,
/// effectively pausing execution until the application should shut down.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
        let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
