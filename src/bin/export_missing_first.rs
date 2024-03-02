use dotenv::dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::sl::sync_vocab::export_missing_first_lang_pairs;
use std::env;
use std::error::Error;

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

    // TODO Add modes to this for various types of exports
    export_missing_first_lang_pairs(&export_file)
}
