use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager, PooledConnection};
use diesel::sql_query;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use lazy_static::lazy_static;
use std::env;

/// Creates a database pool of Postgres connections. The pool is lazy loaded and available globally.
/// Environment variable DATABASE_URL is required for PROD and TEST_DATABASE_URL is required for Tests.
/// If the connection fails, the program panics.

/// Embedded migrations for the application's database.
///
/// This constant represents the migrations embedded into the binary from the `migrations`
/// directory. These migrations are applied to the database to ensure the schema is up-to-date.
///
/// The `embed_migrations!` macro from `diesel_migrations` compiles SQL migration scripts
/// located in the specified directory into the Rust binary, allowing for the application
/// to apply migrations at runtime without needing to locate or load the SQL files from the filesystem.
///
/// # Usage
///
/// Migrations embedded in this way are typically applied using `run_pending_migrations`
/// at application startup to ensure the database schema is current.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Type alias for a connection pool managed by `r2d2` using Diesel's `PgConnection`.
///
/// `DbPool` simplifies references to the specific type of pool used throughout the application,
/// which manages PostgreSQL connections. It encapsulates the complexity of connection management,
/// including creating new connections when needed, handling connection pooling, and recycling connections.
///
/// The pool configuration and instantiation are managed by the `establish_connection_pool` function,
/// which reads database configuration from environment variables and sets up the pool accordingly.
type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn get_database_url() -> String {
    env::var("PALABRA_DATABASE_URL").expect("PALABRA_DATABASE_URL must be set")
}

lazy_static! {
    /// Global instance of the database connection pool.
    ///
    /// This static reference to a `DbPool` is initialized using `lazy_static!` to ensure thread-safe,
    /// one-time initialization. It provides a convenient way to access the database connection pool
    /// from anywhere in the application without needing to pass the pool through function arguments.
    ///
    /// The pool is configured and instantiated at application startup, using the `establish_connection_pool`
    /// function. It reads the database configuration from environment variables and sets up the pool,
    /// which can then be used to get database connections.
    pub static ref POOL: DbPool = establish_connection_pool(get_database_url());
}

/// Establishes and returns a database connection pool using the `DATABASE_URL` environment variable.
///
/// The function reads the database URL directly from the `DATABASE_URL` environment variable,
/// initializes a connection manager with it, and then sets up a connection pool for use throughout
/// the application. The connection pool is configured with default settings.
///
/// # Panics
///
/// Panics if:
/// - The `DATABASE_URL` environment variable is not set.
/// - The connection pool cannot be created due to configuration errors or connection issues.
///
/// # Returns
///
/// A `DbPool` (an `r2d2::Pool` of `ConnectionManager<PgConnection>`) that manages database connections,
/// ready to be used for executing database operations.
///
/// # Example Usage
///
/// use std::env;
/// dotenv::from_filename("test.env").ok();
/// use palabras::dal::db_connection::{establish_connection_pool, get_database_url};
/// let pool = establish_connection_pool(get_database_url());
/// // Now you can use `pool` to get database connections.
///
/// Ensure that the `DATABASE_URL` environment variable is correctly set in your environment before
/// calling this function, for example:
///
/// ```sh
/// export DATABASE_URL=postgres://username:password@localhost/mydatabase
/// ```
///
/// Note: Adjust the DATABASE_URL to match your database credentials and server details.
pub fn establish_connection_pool(db_url: String) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

/// Verifies database connectivity and runs pending Diesel migrations.
///
/// This function attempts to acquire a database connection from the global pool,
/// performs a simple query to ensure the connection is valid, and then runs any pending
/// migrations located in the `migrations` directory.
///
/// # Panics
///
/// Panics if:
/// - A database connection cannot be established.
/// - The simple query check fails.
/// - Running migrations fails due to errors in the migration files or database issues.
pub fn verify_connection_migrate_db() {
    let mut conn = get_connection();
    query_check(&mut conn).expect("DB connections should have worked.");
    run_pending_migrations(&mut conn).expect("Any required migrations should have completed.");
}

/// Fetches a database connection from the global connection pool.
///
/// This function attempts to acquire a database connection from the pool established
/// by `establish_connection_pool`. It is intended for use whenever a new database operation
/// is about to be performed.
///
/// # Panics
///
/// Panics if a database connection cannot be retrieved from the pool, indicating
/// potential issues with the database connectivity or pool configuration.
///
/// # Returns
///
/// A `PooledConnection<ConnectionManager<PgConnection>>`, which is a managed connection
/// that will be returned to the pool once it goes out of scope.
pub fn get_connection() -> PooledConnection<ConnectionManager<PgConnection>> {
    POOL.get()
        .expect("Failed to get a connection from the pool.")
}

/// Executes pending Diesel migrations against the database.
///
/// This function applies any migrations that have not yet been applied to the database,
/// ensuring the schema is up-to-date. Migrations are defined in the `migrations` directory
/// and managed by Diesel's migration harness.
///
/// # Parameters
///
/// * `conn`: A mutable reference to a `PgConnection` to execute migrations on.
///
/// # Returns
///
/// A `Result<(), String>` indicating success or returning an error message if migrations
/// fail to run.
///
/// # Errors
///
/// Returns an error if applying migrations fails, encapsulating the error message as a `String`.
pub fn run_pending_migrations(conn: &mut PgConnection) -> Result<(), String> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|err| err.to_string())?;

    Ok(())
}

/// Performs a simple connectivity check against the database using a provided connection.
///
/// This function executes a trivial SQL query ("SELECT 1") to verify that the database connection
/// is active and working correctly. It is used primarily as a health check before performing
/// more complex operations or running migrations.
///
/// # Parameters
///
/// * `conn`: A mutable reference to a `PgConnection` to perform the check on.
///
/// # Returns
///
/// A `QueryResult<()>` indicating success if the query executes successfully, or containing
/// an error if the query fails.
pub fn query_check(conn: &mut PgConnection) -> QueryResult<()> {
    // This is a simple query that should always work if the connection is set up correctly
    sql_query("SELECT 1").execute(conn)?;

    // If we reach this point, the query executed successfully, and the connection works
    println!("Database connection successful.");
    Ok(())
}
