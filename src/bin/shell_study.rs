use dotenv::dotenv;
use palabras::aws::glue::find_the_database;
use palabras::dal::db_connection::{establish_connection_pool, verify_connection_migrate_db};
use palabras::sl::fuzzy_match_vocab::{LearnVocab, VocabFuzzyMatch};
use std::error::Error;
use std::io::Write;
use std::{env, io};

/// Entry point for the Vocab Learning CLI application.
///
/// This function initializes the application environment, verifies and migrates the database
/// schema as necessary, and starts the learning session for a specified `awesome_person_id`.
/// It loads a set of vocabulary to learn, prompts the user for translations, and updates the
/// study progress based on the user's guesses.
///
/// # Environment
/// See the documentation of [`main`].
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
/// Change the awesome_person_id from it default of 1 with the only argument.
///
/// ```sh
/// cargo run --bin shell_study 1
/// }
/// ```
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok(); // Load environment variables from .env file
    let db_url = find_the_database().await;
    establish_connection_pool(db_url);
    verify_connection_migrate_db()?;

    let args: Vec<String> = env::args().collect();
    let awesome_person_id = if args.len() < 2 {
        1
    } else {
        args[1].clone().parse::<i32>().unwrap()
    };

    let match_service = VocabFuzzyMatch::instance();
    let study_set = match_service.get_vocab_to_learn(awesome_person_id, 10)?;
    for (vocab_study, vocab) in study_set {
        println!();
        println!(
            "{}",
            match_service.determine_prompt(&vocab, &vocab_study.user_notes.unwrap_or_default())
        );

        io::stdout().flush().unwrap(); // Ensure the prompt is displayed before reading input
        let mut guess = String::new(); // Create a mutable variable to store the input

        io::stdin().read_line(&mut guess)?;

        let prompt = match_service.check_response(vocab.id, vocab_study.id, guess)?;

        println!("{}", &prompt);
    }

    Ok(())
}
