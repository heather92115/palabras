use dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::sl::sync_vocab::export_missing_first_lang_pairs;
use std::fs;
use std::path::Path;

#[test]
fn test_export_missing_first_lang_pairs() {
    dotenv::from_filename("test.env").ok();
    verify_connection_migrate_db();

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
