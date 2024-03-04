use crate::dal::progress_stats::{DbProgressStatsRepository, ProgressStatsRepository};
use crate::dal::translation_pair::{DbTranslationPairRepository, TranslationPairRepository};
use crate::models::{ProgressStats, TranslationPair};
use chrono::Utc;
use core::option::Option;
use strsim::levenshtein;

/// #[derive(Clone)]
/// Represents the worst possible answer possible, and thus, it caps the distance.
/// It is used in calculations as well.
pub static MAX_DISTANCE: usize = 10;

/// For simplicity, there is a single user so a single record for overall stats. This record is
/// created as part of the database migration ran by Diesel Migration.
/// TODO Add the ability to track more than one user.
pub static PROGRESS_STATS_ID: i32 = 1;

/// Once percentage correct get higher, the pair is to be marked known or even too easy.
pub static FULLY_KNOWN_THRESHOLD: f64 = 0.98;
pub static TOO_EASY_THRESHOLD: f64 = 0.998;

pub trait LearnTranslationPairs {
    fn get_study_pairs(&self, limit: i64) -> Result<Vec<TranslationPair>, String>;

    /// Evaluates the guessed word against potential correct answers, returning the "distance" from an exact match.
    ///
    /// This function considers both the primary `learning_lang` string and any additional `alternatives` as possible correct answers.
    /// It calculates the Levenshtein distance between the guess and each possible match to find the closest one.
    /// A distance of 0 indicates a perfect match, whereas a distance of 10 represents the worst-case scenario,
    /// meaning no similarity between the guess and possible answers.
    ///
    /// # Parameters
    ///
    /// * `learning_lang` - The primary correct answer string.
    /// * `alternatives` - A comma-separated string of alternative correct answers.
    /// * `guess` - The user's guessed word.
    ///
    /// # Returns
    ///
    /// The smallest Levenshtein distance between the guess and the set of possible correct answers, capped at a maximum of 10.
    fn check_pair_match(
        &self,
        learning_lang: &String,
        alternatives: &String,
        guess: &String,
    ) -> usize;

    /// Updates the statistics for a specific translation pair based on the latest guess's distance from the correct answer.
    ///
    /// This function retrieves the current statistics for a translation pair, calculates the new percentage of correctness
    /// based on the distance provided, updates the pair's stats including whether it's now considered fully known, and then
    /// saves these changes. It also updates global stats accordingly.
    ///
    /// # Parameters
    ///
    /// * `pair_id` - The primary key (`id`) of the translation pair to update.
    /// * `distance` - The distance from the correct answer for the latest guess, where 0 indicates a perfect match.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(TranslationPair)`: The updated `TranslationPair` record.
    /// - `Err(String)`: An error message string if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue fetching the current pair stats, performing the calculation, updating the record in the database,
    /// or updating global progress stats. The error is returned as a `String` describing the failure.
    fn update_pair_stats(&self, pair_id: i32, distance: usize) -> Result<TranslationPair, String>;

    /// Updates the overall progress stats based on the latest quiz result.
    ///
    /// This function calculates the new values for the number of correct and incorrect answers,
    /// the total percentage of correct answers, and updates the progress stats record accordingly.
    /// If the `last_fully_known` flag is true, it also increments the count of known items.
    ///
    /// # Parameters
    ///
    /// * `correct` - A boolean indicating whether the latest answer was correct.
    /// * `last_fully_known` - A boolean indicating whether the last item is now fully known.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(ProgressStats)`: The updated `ProgressStats` record.
    /// - `Err(String)`: An error message string if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue fetching the current progress stats,
    /// performing the calculation, or updating the record in the database.
    /// The error is returned as a `String` describing the failure.
    fn update_overall_progress(
        &self,
        correct: bool,
        last_fully_known: bool,
    ) -> Result<ProgressStats, String>;

    /// Calculates the new average correctness based on the previous correctness value and the distance
    /// of the latest guess. A distance of 0 indicates a perfect match and is given a heavier weighting
    /// in the calculation to favor accuracy.
    ///
    /// # Parameters
    ///
    /// * `previous` - The previous correctness percentage as a floating point number where 1.0
    ///   represents 100% correctness.
    /// * `distance` - The distance from the correct answer for the latest guess, where 0 indicates
    ///   a perfect match.
    ///
    /// # Returns
    ///
    /// The new correctness percentage as a floating point number. This represents the averaged
    /// correctness taking into account the latest guess and applying a heavier weight to perfect
    /// matches.
    fn calc_correctness(&self, previous: f64, distance: usize) -> f64;

    /// Constructs a translation prompt string for a given translation pair.
    ///
    /// This function generates a prompt string to display to the user, based on the provided
    /// `TranslationPair`. The basic prompt format includes the phrase "Translate: 'first_lang'",
    /// where `first_lang` is replaced with the `first_lang` field of the `TranslationPair`.
    ///
    /// If the `TranslationPair` has a non-empty `hint` field, the hint is appended to the prompt
    /// with the format "hint: hint_value". Similarly, if the `TranslationPair` has a non-empty
    /// `pos` (part of speech) field, it is appended with the format "pos: pos_value".
    ///
    /// # Arguments
    ///
    /// * `pair` - A `TranslationPair` instance containing the data to construct the prompt.
    ///
    /// # Returns
    ///
    /// Returns a `String` representing the constructed prompt for translation.
    fn determine_prompt(&self, pair: TranslationPair) -> String;
}

pub struct LearnTranslationPairsFuzzyMatch {
    // Directly store the Box<dyn ProgressStatsRepository>
    process_repo: Box<dyn ProgressStatsRepository>,
    pair_repo: Box<dyn TranslationPairRepository>,
}

pub fn create_fuzzy_match_service() -> Box<dyn LearnTranslationPairs + 'static> {
    let progress_repo = Box::new(DbProgressStatsRepository);
    let pair_repo = Box::new(DbTranslationPairRepository);
    Box::new(LearnTranslationPairsFuzzyMatch::new(
        progress_repo,
        pair_repo,
    ))
}

impl LearnTranslationPairsFuzzyMatch {
    // The constructor now directly takes Box<dyn ProgressStatsRepository>
    // No need for a lifetime parameter
    pub fn new(
        process_repo: Box<dyn ProgressStatsRepository>,
        pair_repo: Box<dyn TranslationPairRepository>,
    ) -> Self {
        LearnTranslationPairsFuzzyMatch {
            process_repo,
            pair_repo,
        }
    }
}

/// An implementation of the LearnTranslationPairs service. It uses fuzzy logic to check word
/// matching then uses the distance from a perfect match to decide how to change the pair's
/// correctness.
impl LearnTranslationPairs for LearnTranslationPairsFuzzyMatch {
    fn get_study_pairs(&self, limit: i64) -> Result<Vec<TranslationPair>, String> {
        let study_pairs = self.pair_repo.get_study_pairs()?;

        // Separate pairs into those that have been tested and those that have not.
        let (mut study_list, not_tested): (Vec<_>, Vec<_>) = study_pairs.into_iter()
            .partition(|l| l.last_tested.is_some());

        // Now sort the list by last_tested in ascending order giving a little time before presenting
        // the last studied pair.
        study_list.sort_by(|a, b| {
            a.last_tested.clone().unwrap_or_default().cmp(&b.last_tested.clone().unwrap_or_default())
        });

        // Grab more pairs if the user has finished learning the already tested pairs.
        if study_list.len() < limit as usize {
            study_list.extend(not_tested.into_iter().take(limit as usize - study_list.len()));
        }

        Ok(study_list)
    }

    /// Implementation, see trait for details [`LearnTranslationPairs::check_pair_match`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit tests in this module.
    fn check_pair_match(
        &self,
        learning_lang: &String,
        alternatives: &String,
        guess: &String,
    ) -> usize {
        if guess.trim().is_empty() {
            return MAX_DISTANCE;
        }

        let mut possible_matches: Vec<String> = alternatives
            .to_lowercase()
            .split(",")
            .map(|s| s.trim().to_string())
            .collect();
        possible_matches.push(learning_lang.clone().to_lowercase().trim().to_string());

        let mut distance = MAX_DISTANCE;
        for possible_match in possible_matches {
            let score = levenshtein(&possible_match, guess.to_lowercase().trim());

            // Find the best match
            if score < distance {
                distance = score;
            }
        }

        if distance > MAX_DISTANCE {
            MAX_DISTANCE
        } else {
            distance
        }
    }

    /// Implementation, see trait for details [`LearnTranslationPairs::update_pair_stats`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit tests in this module.
    fn update_pair_stats(&self, pair_id: i32, distance: usize) -> Result<TranslationPair, String> {
        let current = self
            .pair_repo
            .get_translation_pair_by_id(pair_id)
            .map_err(|err| err.to_string())?;
        let updated_percentage_correct =
            self.calc_correctness(current.percentage_correct.unwrap_or_default(), distance);

        let updating = TranslationPair {
            percentage_correct: Option::from(updated_percentage_correct),
            last_tested: Option::from(Utc::now()),
            fully_known: updated_percentage_correct > FULLY_KNOWN_THRESHOLD,
            guesses: Option::from(current.guesses.unwrap_or_default() + 1),
            ..current
        };

        // Save changes to dal.
        self.pair_repo
            .update_translation_pair(updating)
            .map_err(|err| err.to_string())?;
        let updated = self
            .pair_repo
            .get_translation_pair_by_id(pair_id)
            .map_err(|err| err.to_string())?;

        // Update the global stats too.
        self.update_overall_progress(distance == 0, updated.fully_known.clone())?;
        Ok(updated)
    }

    /// Implementation, see trait for details [`LearnTranslationPairs::update_overall_progress`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit tests in this module.
    fn update_overall_progress(
        &self,
        correct: bool,
        last_fully_known: bool,
    ) -> Result<ProgressStats, String> {
        let current = self
            .process_repo
            .get_progress_stats_by_id(PROGRESS_STATS_ID)
            .map_err(|err| err.to_string())?;

        // Increment counters based on whether the answer was correct
        let (num_correct, num_incorrect) = (
            current.num_correct.unwrap_or(0) + correct as i32,
            current.num_incorrect.unwrap_or(0) + (!correct) as i32,
        );

        // Calculate the total percentage
        let total_percentage = num_correct as f64 / (num_correct + num_incorrect) as f64;

        // Prepare the updated stats
        let updating = ProgressStats {
            num_known: if last_fully_known { Some(current.num_known.unwrap_or(0) + 1) } else { current.num_known },
            num_correct: Some(num_correct),
            num_incorrect: Some(num_incorrect),
            total_percentage: Some(total_percentage),
            updated: Utc::now(),
            ..current
        };

        // Update the stats and return the updated record
        self.process_repo.update_progress_stats(updating).map_err(|err| err.to_string())?;
        self.process_repo.get_progress_stats_by_id(PROGRESS_STATS_ID).map_err(|err| err.to_string())
    }

    /// Implementation, see trait for details [`LearnTranslationPairs::calc_correctness`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit tests in this module.
    fn calc_correctness(&self, previous: f64, distance: usize) -> f64 {
        let mut score =
            ((MAX_DISTANCE as f64 - distance as f64) / MAX_DISTANCE as f64 + previous) / 2.0;

        if distance == 0 {
            // weights a perfect match as two perfect answers instead of one.
            score = (2.0 + previous) / 3.0;
        }

        score
    }

    /// Implementation, see trait for details [`LearnTranslationPairs::determine_prompt`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit tests in this module.
    fn determine_prompt(&self, pair: TranslationPair) -> String {
        let mut prompt = format!("Translate: '{}'",  &pair.first_lang);
        if !pair.hint.clone().unwrap_or_default().is_empty() {
            prompt =  format!("{}    hint: {}",  prompt, &pair.hint.unwrap_or_default());
        }

        if !pair.pos.clone().unwrap_or_default().is_empty() {
            prompt =  format!("{}    pos: {}",  prompt, &pair.pos.unwrap_or_default());
        }

        if !pair.user_notes.clone().unwrap_or_default().is_empty() {
            prompt =  format!("{}    your notes: {}",  prompt, &pair.user_notes.unwrap_or_default());
        }

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::NewTranslationPair;
    use diesel::result::Error as DieselError;

    // Mock-up functions to simulate actual function behaviors
    pub struct MockProgressStatsRepository;
    impl ProgressStatsRepository for MockProgressStatsRepository {
        fn get_progress_stats_by_id(&self, stats_id: i32) -> Result<ProgressStats, DieselError> {
            Ok(ProgressStats {
                id: stats_id,
                num_known: Some(100),
                num_correct: Some(80),
                num_incorrect: Some(20),
                total_percentage: Some(0.8),
                updated: chrono::Utc::now(),
            })
        }
        fn update_progress_stats(&self, _: ProgressStats) -> Result<usize, String> {
            Ok(1) // Simulate successful update of one record
        }
    }

    // Mock struct for TranslationPairRepository
    pub struct MockTranslationPairRepository;

    // Mock implementation of TranslationPairRepository
    impl TranslationPairRepository for MockTranslationPairRepository {
        fn get_translation_pair_by_id(&self, pair_id: i32) -> Result<TranslationPair, DieselError> {
            // Mock behavior: Return an Ok result with a dummy TranslationPair
            Ok(TranslationPair {
                id: pair_id,
                learning_lang: "Example language".to_string(),
                first_lang: "Example first language".to_string(),
                percentage_correct: Some(1.0),
                ..Default::default()
            })
        }

        fn find_translation_pair_by_learning_language(
            &self,
            learning_lang_search: String,
        ) -> Result<Option<TranslationPair>, DieselError> {
            // Mock behavior: Return Some(TranslationPair) or None based on a condition
            Ok(Some(TranslationPair {
                id: 1,
                learning_lang: learning_lang_search,
                first_lang: "Example first language".to_string(),
                percentage_correct: Some(1.0),
                ..Default::default()
            }))
        }

        fn find_translation_pair_by_alternative(
            &self,
            _alternative_search: String,
        ) -> Result<Option<TranslationPair>, DieselError> {
            todo!()
        }

        fn get_empty_first_lang_pairs(&self, _limit: i64) -> Result<Vec<TranslationPair>, String> {
            // Mock behavior: Return an empty Vec or a Vec with dummy TranslationPairs
            Ok(vec![])
        }

        fn get_study_pairs(&self) -> Result<Vec<TranslationPair>, String> {
            // Mock behavior: Return a Vec with a limited number of dummy TranslationPairs
            Ok(vec![
                TranslationPair {
                    id: 1,
                    learning_lang: "Study language".to_string(),
                    first_lang: "Study first language".to_string(),
                    percentage_correct: Some(0.5),
                    ..Default::default()
                }
            ])
        }

        fn create_translation_pair(
            &self,
            new_translation_pair: &NewTranslationPair,
        ) -> Result<TranslationPair, String> {
            // Mock behavior: Return an Ok result with a newly "created" TranslationPair
            Ok(TranslationPair {
                id: 2, // Simulate that a new ID was assigned
                learning_lang: new_translation_pair.learning_lang.clone(),
                first_lang: new_translation_pair.first_lang.clone(),
                percentage_correct: new_translation_pair.percentage_correct,
                ..Default::default()
            })
        }

        fn update_translation_pair(&self, _updating: TranslationPair) -> Result<usize, String> {
            // Mock behavior: Return Ok(1) to simulate a successful update
            Ok(1)
        }
    }

    #[test]
    fn unit_test_fuzzy_pair_match() {
        let test_cases = vec![
            ("comprendimos", "", "comprendimos", 0),
            ("comprendimos", "", "", 10),
            ("comprendimos", "entendemos, intiendemos", "comprendimos", 0),
            ("comprendimos", "entendemos, intiendemos", "entendemos", 0),
            ("comprendimos", "entendemos, intiendemos", "intiendemos", 0),
            ("comprendimos", "entendemos, intiendemos", "intiendemo", 1),
            // (learning_lang, alternatives, guess, expected)
        ];

        let progress_repo = Box::new(MockProgressStatsRepository);
        let pair_repo = Box::new(MockTranslationPairRepository);
        let fuzzy_service = Box::new(LearnTranslationPairsFuzzyMatch::new(
            progress_repo,
            pair_repo,
        ));

        for (learning_lang, alternatives, guess, expected) in test_cases {
            let result = fuzzy_service.check_pair_match(
                &learning_lang.to_string(),
                &alternatives.to_string(),
                &guess.to_string(),
            );
            assert!(
                result.le(&expected),
                "Calculated distance was not as expected. Result: {}, Expected: {} for learning_lang: {}, alternatives: {}, guess: {}",
                result, expected, &learning_lang, &alternatives, &guess
            )
        }
    }

    #[test]
    fn unit_test_calc_correctness() {
        let test_cases = vec![
            (0.5, 0, 0.83),
            (0.5, 2, 0.65),
            (0.4, 6, 0.4),
            (1.0, 10, 0.5),
            // (previous, distance, expected)
        ];

        let tolerance = 0.01; // Define a suitable tolerance for the comparison of floats

        let progress_repo = Box::new(MockProgressStatsRepository);
        let pair_repo = Box::new(MockTranslationPairRepository);
        let fuzzy_service = Box::new(LearnTranslationPairsFuzzyMatch::new(
            progress_repo,
            pair_repo,
        ));

        for (previous, distance, expected) in test_cases {
            let result = fuzzy_service.calc_correctness(previous, distance);
            assert!(
                (result - expected).abs() < tolerance,
                "Calculated correctness was not as expected. Result: {}, Expected: {} for previous: {}, distance: {}",
                result, expected, previous, distance
            );
        }
    }

    #[test]
    fn unit_test_update_correctness() {
        let previous_correctness = 0.75; // 75% correctness prior to the latest guess
        let distance_for_latest_guess = 2; // The guess was fairly close, but not perfect

        let progress_repo = Box::new(MockProgressStatsRepository);
        let pair_repo = Box::new(MockTranslationPairRepository);
        let fuzzy_service = Box::new(LearnTranslationPairsFuzzyMatch::new(
            progress_repo,
            pair_repo,
        ));

        let new_correctness =
            fuzzy_service.calc_correctness(previous_correctness, distance_for_latest_guess);
        println!("New correctness: {:.2}", new_correctness);

        // Demonstrating the effect of a perfect guess
        let perfect_distance = 0; // A perfect guess
        let new_correctness_with_perfect =
            fuzzy_service.calc_correctness(previous_correctness, perfect_distance);
        println!(
            "New correctness with a perfect guess: {:.2}",
            new_correctness_with_perfect
        );
    }

    #[test]
    fn unit_test_update_overall_progress() {
        let progress_repo = Box::new(MockProgressStatsRepository);
        let pair_repo = Box::new(MockTranslationPairRepository);
        let fuzzy_service = Box::new(LearnTranslationPairsFuzzyMatch::new(
            progress_repo,
            pair_repo,
        ));
        let correct = true;
        let last_fully_known = false;
        match fuzzy_service.update_overall_progress(correct, last_fully_known) {
            Ok(updated_progress) => println!("Updated progress stats: {}", updated_progress.id),
            Err(e) => println!("Error updating progress stats: {}", e),
        }
    }

    #[test]
    fn unit_test_update_pair_stats() {
        let pair_id = 1;
        let distance = 2; // The guess was close, but not perfect

        let progress_repo = Box::new(MockProgressStatsRepository);
        let pair_repo = Box::new(MockTranslationPairRepository);
        let fuzzy_service = Box::new(LearnTranslationPairsFuzzyMatch::new(
            progress_repo,
            pair_repo,
        ));
        match fuzzy_service.update_pair_stats(pair_id, distance) {
            Ok(updated_pair) => println!(
                "Updated translation pair stats: {:?}, fully known: {}",
                updated_pair.percentage_correct, updated_pair.fully_known
            ),
            Err(e) => println!("Error updating translation pair stats: {}", e),
        }
    }

    #[test]
    fn unit_test_check_pair_match() {
        let progress_repo = Box::new(MockProgressStatsRepository);
        let pair_repo = Box::new(MockTranslationPairRepository);
        let fuzzy_service = Box::new(LearnTranslationPairsFuzzyMatch::new(
            progress_repo,
            pair_repo,
        ));

        // Test a perfect guess
        let learning_lang = "La gata es muy inteligente".to_string(); // The word to learn
        let alternatives = "La felina es muy inteligente".to_string(); // Alternative correct answers
        let guess = learning_lang.clone(); // A perfect guess
        let distance = fuzzy_service.check_pair_match(&learning_lang, &alternatives, &guess);
        assert_eq!(
            distance, 0,
            "A perfect guess should return a distance of 0."
        );

        // Demonstrating the effect of a close, but not perfect, guess
        let close_guess = "La gata es muy perezosa".to_string();
        let distance_for_close_guess =
            fuzzy_service.check_pair_match(&learning_lang, &alternatives, &close_guess);
        println!("Distance for a close guess: {}", distance_for_close_guess);
        // Expecting a small distance greater than 0 but less than MAX_DISTANCE

        // Demonstrating the effect of a guess with no similarity
        let no_similarity_guess = "This isn't even spanish!".to_string();
        let distance_for_no_similarity =
            fuzzy_service.check_pair_match(&learning_lang, &alternatives, &no_similarity_guess);
        assert_eq!(
            distance_for_no_similarity, MAX_DISTANCE,
            "A guess with no similarity should return the maximum distance."
        );
    }

    #[test]
    fn unit_test_determine_prompt() {
        let progress_repo = Box::new(MockProgressStatsRepository);
        let pair_repo = Box::new(MockTranslationPairRepository);
        let fuzzy_service = Box::new(LearnTranslationPairsFuzzyMatch::new(
            progress_repo,
            pair_repo,
        ));

        // Define test cases
        let test_cases = vec![
            (
                TranslationPair {
                    first_lang: "amor".to_string(),
                    hint: Some("noun".to_string()),
                    pos: Some("love".to_string()),
                    ..Default::default()
                },
                "Translate: 'amor'    hint: noun    pos: love".to_string(),
            ),
            (
                TranslationPair {
                    first_lang: "correr".to_string(),
                    hint: None,
                    pos: Some("verb".to_string()),
                    ..Default::default()
                },
                "Translate: 'correr'    pos: verb".to_string(),
            ),
            (
                TranslationPair {
                    first_lang: "amarillo".to_string(),
                    hint: Some("color".to_string()),
                    pos: None,
                    ..Default::default()
                },
                "Translate: 'amarillo'    hint: color".to_string(),
            ),
            (
                TranslationPair {
                    first_lang: "libro".to_string(),
                    hint: None,
                    pos: None,
                    ..Default::default()
                },
                "Translate: 'libro'".to_string(),
            ),
            (
                TranslationPair {
                    first_lang: "libro".to_string(),
                    user_notes: Some("something you read".to_string()),
                    ..Default::default()
                },
                "Translate: 'libro'    your notes: something you read".to_string(),
            ),
        ];

        // Run test cases
        for (pair, expected_prompt) in test_cases {
            let prompt = fuzzy_service.determine_prompt(pair);
            assert_eq!(prompt, expected_prompt, "Prompt did not match expected value for TranslationPair");
        }

    }
}
