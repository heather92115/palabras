use dotenv::dotenv;
use palabras::aws::glue::find_the_database;
use palabras::dal::db_connection::{establish_connection_pool, verify_connection_migrate_db};
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
/// Vocab words with missing first language fields are exported, no matter what user uploaded them.
///
/// # Environment
/// See the documentation of [`main`].
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok(); // Load environment variables from .env file
    let db_url = find_the_database().await;
    establish_connection_pool(db_url);
    verify_connection_migrate_db()?;

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
