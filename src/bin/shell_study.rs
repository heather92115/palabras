use dotenv::dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::sl::learn_pairs::create_fuzzy_match_service;
use std::error::Error;
use std::io;
use std::io::Write;


/// Entry point for the Vocab Learning CLI application.
///
/// This function initializes the application environment, verifies and migrates the database
/// schema as necessary, and starts the learning session for a specified `awesome_person_id`.
/// It loads a set of vocabulary to learn, prompts the user for translations, and updates the
/// study progress based on the user's guesses.
///
/// # Environment
///
/// Requires an environment variable `PALABRA_DATABASE_URL` to specify the database connection URL.
/// The `.env` file is used to load environment variables.
///
/// # Behavior
///
/// - Retrieves the study set for the `awesome_person_id`.
/// - For each vocab item in the study set, displays a prompt for the user to enter a translation.
/// - Reads the user's input and calculates the similarity distance between the guessed word and the correct translation.
/// - Updates the vocabulary study stats based on the user's guess.
/// - Displays feedback about the correctness of the guess and the updated correctness percentage.
///
/// # Errors
///
/// This function returns an `Err` if any step of the process fails, including database connection
/// issues, reading from stdin, or any other internal error.
///
///
/// To run the application, ensure that you have a `.env` file with the `PALABRA_DATABASE_URL`
/// defined, and then execute the binary. The application will guide you through learning sessions
/// based on your progress.
///
/// ```sh
/// cargo run --bin shell_study
/// }
/// ```
pub fn main() -> Result<(), Box<dyn Error>> {

    // Returning the PROD database URL defined in the env var: PALABRA_DATABASE_URL
    dotenv().ok(); // Load environment variables from .env file

    verify_connection_migrate_db();

    let awesome_person_id = 1; // todo make this not for just me

    let match_service = create_fuzzy_match_service();
    let study_set = match_service.get_vocab_to_learn(awesome_person_id, 10)?;
    for (vocab_study, vocab) in study_set {
        println!();
        println!("{}", match_service.determine_prompt(&vocab, &vocab_study.user_notes.unwrap_or_default()));

        io::stdout().flush().unwrap(); // Ensure the prompt is displayed before reading input
        let mut guess = String::new(); // Create a mutable variable to store the input

        io::stdin().read_line(&mut guess)?;

        let distance = match_service.check_vocab_match(
            &vocab.learning_lang,
            &vocab.alternatives.clone().unwrap(),
            &guess,
        );

        let updated = match_service.update_vocab_study_stats(vocab_study.id, awesome_person_id, distance)?;

        if distance > 0 {
            println!("'{}' != '{}'", &vocab.learning_lang, &guess.trim());
        }

        println!(
            "Correctness {} -> {}",
            &vocab_study.percentage_correct.unwrap(),
            &updated.percentage_correct.unwrap()
        );
    }

    Ok(())
}
