use dotenv::dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::sl::learn_pairs::create_fuzzy_match_service;
use std::error::Error;
use std::io;
use std::io::Write;


/// Ensure that the `PALABRA_DATABASE_URL` environment variable is correctly set in the file .env
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
