use dotenv::dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::sl::sync_vocab::export_missing_first_lang_pairs;
use std::env;
use std::error::Error;

/// Main entry point for the vocabulary export tool.
///
/// This function initializes the application by loading the environment variables,
/// verifies and migrates the database schema as necessary, and performs a vocabulary
/// export operation. The export file path can be specified as a command-line argument;
/// if not provided, a default file path is used.
///
/// # Environment
///
/// The function reads the `PALABRA_DATABASE_URL` environment variable to establish a
/// database connection. Environment variables should be defined in a `.env` file located
/// in the same directory as the binary.
///
/// # Arguments
///
/// - `argv[1]` (optional): The path to the export file. If not specified, defaults to
///   `"data/export.csv"`.
///
/// # Behavior
///
/// The function supports exporting missing first language pairs from the database into a CSV file.
/// Future versions may include additional modes for different types of exports, such as filtering
/// by specific languages or exporting vocabulary for specific users.
///
/// # Errors
///
/// Returns an error if it encounters issues loading environment variables, connecting to the
/// database, performing the migration, or exporting the data.
///
/// # Example Usage
///
/// Run without arguments to export to the default file:
/// ```sh
/// cargo run --bin export_missing_first
/// ```
/// Or specify a custom export file path:
/// ```sh
/// cargo run --bin export_missing_first "custom/path/export.csv"
/// ```
///
/// Note: This function is designed to be run as a standalone tool. It should be invoked from
/// the command line with the necessary environment configuration in place.
fn main() -> Result<(), Box<dyn Error>> {
    // Returning the PROD database URL defined in the env var: PALABRA_DATABASE_URL
    dotenv().ok(); // Load environment variables from .env file
    verify_connection_migrate_db();

    let args: Vec<String> = env::args().collect();

    let export_file = if args.len() < 2 {
        "data/export.csv".to_string()
    } else {
        args[1].clone()
    };

    // TODO Add modes to this for various types of exports,
    // TODO alternative export filters, specific languages or specific user vocabs
    export_missing_first_lang_pairs(&export_file)
}
