use dotenv::dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::sl::learn_pairs::create_fuzzy_match_service;
use std::error::Error;
use std::io;
use std::io::Write;


/// Ensure that the `PALABRA_DATABASE_URL` environment variable is correctly set in the file .env
pub fn main()  -> Result<(), Box<dyn Error>> {

    // Returning the PROD database URL defined in the env var: PALABRA_DATABASE_URL
    dotenv().ok(); // Load environment variables from .env file

    verify_connection_migrate_db();

    let match_service = create_fuzzy_match_service();
    let pairs = match_service.get_study_pairs(10)?;
    for pair in pairs {
        println!();
        println!("{}", match_service.determine_prompt(pair.clone()));

        io::stdout().flush().unwrap(); // Ensure the prompt is displayed before reading input
        let mut guess = String::new(); // Create a mutable variable to store the input

        io::stdin().read_line(&mut guess)?;

        let distance = match_service.check_pair_match(
            &pair.learning_lang,
            &pair.alternatives.unwrap(),
            &guess,
        );

        let updated = match_service.update_pair_stats(pair.id, distance)?;

        if distance > 0 {
            println!("'{}' != '{}'", &pair.learning_lang, &guess.trim());
        }

        println!(
            "Correctness {} -> {}",
            &pair.percentage_correct.unwrap(),
            &updated.percentage_correct.unwrap()
        );
    }

    Ok(())
}
