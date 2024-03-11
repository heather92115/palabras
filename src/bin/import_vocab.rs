use dotenv::dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::sl::sync_vocab::import_duo_vocab;
use std::error::Error;
use std::{env, process};
use palabras::config::{load_translations_config, load_vocab_config};

/// A utility to import Duolingo's vocabulary JSON file and a common translations file into the database.
///
/// This program loads vocabulary data from a JSON file exported from Duolingo and common translation
/// mappings from a custom files.
///
/// It inserts this data into the project's database for further processing and use. This import can be
/// run multiple times to add new vocab words or new translations as needed.
///
/// ## Prerequisites
///
/// - Ensure the `DATABASE_URL` environment variable is correctly set to point to your database.
/// This variable should follow the format: `postgres://username:password@localhost/database_name`.
///


/// ## Configuration
///
/// `VocabConfig` struct contains the duolingo vocab file to be imported. It can be used to combine singular and plural words.
/// `TranslationsConfig` struct contains configs to import translations for the Duolingo vocab since they are not included.
///
/// ## Usage
///
/// Once the JSON files have been configured and your vocab from DuoLingo is in the referenced file, run the import command
/// with the awesome_person_id. It defaults to 1.
///
/// ```sh
/// cargo run --bin import_vocab 1
/// ```
///
fn main() -> Result<(), Box<dyn Error>> {
    // Returning the PROD database URL defined in the env var: PALABRA_DATABASE_URL
    dotenv().ok(); // Load environment variables from .env file
    verify_connection_migrate_db();

    let args: Vec<String> = env::args().collect();
    let awesome_person_id = if args.len() < 2 {
        None
    } else {
        Some(args[1].clone().parse::<i32>().unwrap())
    };

    if awesome_person_id.is_none() {
        eprintln!("Awesome person id is required");
        eprintln!("Try: ");
        eprintln!("cargo run --bin import_vocab 1");
        process::exit(2);
    }

    let awesome_person_id = awesome_person_id.unwrap_or_default();

    let vocab_config = load_vocab_config().unwrap_or_else(|err| {
        eprintln!("Failed to load import translation configs: {}", err);
        process::exit(3);
    });

    let translation_configs = load_translations_config().unwrap_or(None);

    import_duo_vocab(&vocab_config, translation_configs, awesome_person_id).unwrap_or_else(|err| {
        eprintln!("Problem processing word pairs: {}", err);
        process::exit(4);
    });

    println!("Done!");
    Ok(())
}
