use dotenv::dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use iced::{Application, Settings};
use palabras::gui::card::Card;

/// Ensure that the `PALABRA_DATABASE_URL` environment variable is correctly set in the file .env
///
pub fn main() -> iced::Result {
    // Returning the PROD database URL defined in the env var: PALABRA_DATABASE_URL
    dotenv().ok(); // Load environment variables from .env file

    verify_connection_migrate_db();

    Card::run(Settings::default())
}
