use dotenv::dotenv;
use palabras::aws::glue::find_the_database;
use palabras::dal::db_connection::{establish_connection_pool, verify_connection_migrate_db};
use palabras::gql::router::start_axum;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Starts the web server using Axum framework to handle GraphQL queries.
///
/// This program initializes the database connection, sets up routing for GraphQL queries,
/// and starts an HTTP server to listen for requests. The server's listening address and port
/// are configured through the `PAL_SERVER_ADDR` environment variable. Otherwise, it defaults to `127.0.0.1:3000`.
/// The database connection is established based on the `PALABRA_DATABASE_URL`
/// environment variable.
///
/// # Environment Variables
///
/// - `PAL_DB_LINK`: The link used to get the database connection info.
/// - `PAL_REGION`: Region where the database connection info is expected to be located.
/// - `PAL_DATABASE_URL`: Specifies the fallback database URL for connecting to the PostgreSQL database.
///   If neither of the former env vars are set, this will attempt to be used.
///
///
/// - `PAL_SERVER_ADDR`: Defines the IP address and port where the server will listen for incoming HTTP requests.
///   The format should be `IP:PORT`, e.g., `127.0.0.1:3000`.
///
/// # Panics
///
/// The function will panic if:
///
/// - The there is no method available to connect to the database.
///
/// - The `PAL_SERVER_ADDR`  cannot be parsed as a `SocketAddr`.
///
/// - The TCP listener fails to bind to the specified address.
///
/// # Examples
///
/// Ensure you have set the required environment variables in your `.env` file:
///
/// ```plaintext
/// PAL_DATABASE_URL=postgres://rustacean:password@127.0.0.1/postgres
/// PAL_SERVER_ADDR=127.0.0.1:3000
/// ```
///
/// Run the server with `cargo run`, and it will start listening on the specified `PAL_SERVER_ADDR`.
///
/// # Errors
///
/// Returns an error if any operation within the function fails, encapsulated within a `Box<dyn Error>`.
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    println!("Num CPUs {}", num_cpus::get());

    // Returning the PROD database URL defined in the env var: PALABRA_DATABASE_URL
    dotenv().ok(); // Load environment variables from .env file

    let db_url = find_the_database().await;
    establish_connection_pool(db_url);
    verify_connection_migrate_db()?;

    // Get the server address from the `PAL_SERVER_ADDR` environment variable
    let env_server_addr = std::env::var("PAL_SERVER_ADDR").unwrap_or_default();
    let server_addr = if env_server_addr.is_empty() {
        "0.0.0.0:3000".to_string()
    } else {
        env_server_addr
    };

    // Parse the address as a `SocketAddr`
    let addr: SocketAddr = server_addr
        .parse()
        .expect("Failed to parse SERVER_ADDR as SocketAddr");

    println!("Started server running on {addr}");

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to listen on {addr}: {e}"));

    start_axum(listener).await;

    Ok(())
}
