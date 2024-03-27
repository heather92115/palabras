use dotenv::dotenv;
use palabras::dal::db_connection::{establish_connection_pool, verify_connection_migrate_db};
use palabras::sl::sync_vocab::export_missing_first_lang_pairs;
use std::path::Path;
use std::{env, fs};

fn get_test_db_url() -> String {
    env::var("TEST_DATABASE_URL").expect("env var TEST_DATABASE_URL was not found")
}

#[test]
fn test_export_missing_first_lang_pairs() {
    dotenv().ok(); // Load environment variables from .env file

    establish_connection_pool(get_test_db_url());
    verify_connection_migrate_db().expect("connection and migration should have worked");

    let export_file = "tests/data/es_en_mapping/test_export.csv";

    if let Err(e) = delete_file_if_exists(export_file) {
        eprintln!("Error deleting file: {}", e);
    }

    export_missing_first_lang_pairs(export_file).unwrap_or_else(|err| {
        eprintln!("Problem processing word pairs: {}", err);
        panic!("Export failed");
    });
}

fn delete_file_if_exists(file_path: &str) -> std::io::Result<()> {
    let path = Path::new(file_path);
    if path.exists() {
        fs::remove_file(path)?;
        println!("File {} has been deleted.", file_path);
    } else {
        println!("File {} does not exist, no need to delete.", file_path);
    }
    Ok(())
}
