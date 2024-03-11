use dotenv::dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::gql::router::start_axum;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Starts the web server using Axum framework to handle GraphQL queries.
///
/// This program initializes the database connection, sets up routing for GraphQL queries,
/// and starts an HTTP server to listen for requests. The server's listening address and port
/// are configured through the `SERVER_ADDR` environment variable. Otherwise, it defaults to `127.0.0.1:3000`.
/// The database connection is established based on the `PALABRA_DATABASE_URL`
/// environment variable.
///
/// # Environment Variables
///
/// - `PALABRA_DATABASE_URL`: Specifies the database URL for connecting to the PostgreSQL database.
///   It must be set in the `.env` file or the environment.
///
/// - `SERVER_ADDR`: Defines the IP address and port where the server will listen for incoming HTTP requests.
///   The format should be `IP:PORT`, e.g., `127.0.0.1:3000`.
///
/// # Panics
///
/// The function will panic if:
///
/// - The `PALABRA_DATABASE_URL` environment variable is not set or fails to
///   connect to a database with the schema `palabras` Read the docs for detailed instructions.
///
/// - The `SERVER_ADDR`  cannot be parsed as a `SocketAddr`.
///
/// - The TCP listener fails to bind to the specified address.
///
/// # Examples
///
/// Ensure you have set the required environment variables in your `.env` file:
///
/// ```plaintext
/// PALABRA_DATABASE_URL=postgres://rustacean:password@0.0.0.0/postgres
/// SERVER_ADDR=0.0.0.0:3000
/// ```
///
/// Run the server with `cargo run`, and it will start listening on the specified `SERVER_ADDR`.
///
/// # Errors
///
/// Returns an error if any operation within the function fails, encapsulated within a `Box<dyn Error>`.
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    println!("Num CPUs {}", num_cpus::get());

    // Returning the PROD database URL defined in the env var: PALABRA_DATABASE_URL
    dotenv().ok(); // Load environment variables from .env file

    verify_connection_migrate_db();

    // Get the server address from the `SERVER_ADDR` environment variable
    let env_server_addr = std::env::var("SERVER_ADDR").unwrap_or_default();
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
