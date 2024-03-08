use crate::dal::awesome_person::{DbAwesomePersonRepository, AwesomePersonRepository};
use crate::models::{AwesomePerson, Vocab, VocabStudy};
use chrono::Utc;
use core::option::Option;
use strsim::levenshtein;
use crate::dal::vocab_study::{DbVocabStudyRepository, VocabStudyRepository};

/// #[derive(Clone)]
/// Represents the worst possible answer possible, and thus, it caps the distance.
/// It is used in calculations as well.
pub static MAX_DISTANCE: usize = 10;

/// Once percentage correct get higher, the pair is to be marked known or even too easy.
pub static WELL_KNOWN_THRESHOLD: f64 = 0.98;

pub trait LearnVocab {

    /// Retrieves a prioritized list of vocabulary sets for learning or review for a specified awesome person.
    ///
    /// This function queries the database to get a study set of vocabulary pairs for the given `awesome_id`.
    /// It prioritizes vocabulary based on whether it has been tested before and if it is not marked as well known.
    /// The result is a list of vocabulary pairs sorted to prioritize learning, with a limit on the number of pairs returned.
    ///
    /// # Parameters
    ///
    /// - `awesome_id`: The identifier of the awesome person for whom the vocabulary set is  being retrieved.
    /// - `limit`: The maximum size of the vocabulary set to return.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(Vec<(VocabStudy, Vocab)>)`: A vector of tuples, each containing a `VocabStudy` record
    ///   and its corresponding `Vocab` record, limited by the specified `limit`.
    /// - `Err(String)`: An error message string if the retrieval process fails.
    ///
    /// # Details
    ///
    /// The function first filters the vocabulary pairs to separate them into two groups based on their
    /// learning priority. Then, it sorts the high-priority group by the `last_tested` date to prioritize
    /// the most recently tested items. If the high-priority group contains fewer items than the specified limit,
    /// additional pairs from the secondary group are added to the result set. The final list is then truncated
    /// to meet the specified `limit` and reversed to ensure variety in presentation.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The retrieval of the study set from the database fails.
    fn get_vocab_to_learn(&self, awesome_id: i32, limit: i64) -> Result<Vec<(VocabStudy, Vocab)>, String>;

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
    fn check_vocab_match(
        &self,
        learning_lang: &String,
        alternatives: &String,
        guess: &String,
    ) -> usize;

    /// Updates the statistics for a specific vocab based on the latest guess's distance from the correct answer.
    ///
    /// This function retrieves the current statistics for a vocab, calculates the new percentage of correctness
    /// based on the distance provided, updates the pair's stats including whether it's now considered fully known, and then
    /// saves these changes. It also updates global stats accordingly.
    ///
    /// # Parameters
    ///
    /// * `pair_id` - The primary key (`id`) of the vocab to update.
    /// * `distance` - The distance from the correct answer for the latest guess, where 0 indicates a perfect match.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(Vocab)`: The updated `Vocab` record.
    /// - `Err(String)`: An error message string if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue fetching the current pair stats, performing the calculation, updating the record in the database,
    /// or updating global progress stats. The error is returned as a `String` describing the failure.
    fn update_vocab_study_stats(&self, vocab_study_id: i32,  awesome_person_id: i32, distance: usize) -> Result<VocabStudy, String>;

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
        awesome_person_id: i32,
        correct: bool,
        last_fully_known: bool,
    ) -> Result<AwesomePerson, String>;

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

    /// Constructs a translation prompt string for a given vocab.
    ///
    /// This function generates a prompt string to display to the user, based on the provided
    /// `Vocab`. The basic prompt format includes the phrase "Translate: 'first_lang'",
    /// where `first_lang` is replaced with the `first_lang` field of the `Vocab`.
    ///
    /// If the `Vocab` has a non-empty `hint` field, the hint is appended to the prompt
    /// with the format "hint: hint_value". Similarly, if the `Vocab` has a non-empty
    /// `pos` (part of speech) field, it is appended with the format "pos: pos_value".
    ///
    /// # Arguments
    ///
    /// * `vocab` - A `Vocab` instance containing the data to construct the prompt.
    /// * `user_notes` - Any user entered notes to help them with this vocab.
    ///
    /// # Returns
    ///
    /// Returns a `String` representing the constructed prompt for translation.
    fn determine_prompt(&self, vocab: &Vocab, user_notes: &str) -> String;
}

pub struct VocabFuzzyMatch {
    // Directly store the Box<dyn ProgressStatsRepository>
    awesome_person_repo: Box<dyn AwesomePersonRepository>,
    vocab_study_repo: Box<dyn VocabStudyRepository>,
}

pub fn create_fuzzy_match_service() -> Box<dyn LearnVocab + 'static> {
    let awesome_person_repo = Box::new(DbAwesomePersonRepository);
    let vocab_study_repo = Box::new(DbVocabStudyRepository);
    Box::new(VocabFuzzyMatch::new(
        awesome_person_repo,
        vocab_study_repo,
    ))
}

impl VocabFuzzyMatch {
    // The constructor takes Box<dyn Repos>
    pub fn new(
        awesome_person_repo: Box<dyn AwesomePersonRepository>,
        vocab_study_repo: Box<dyn VocabStudyRepository>,
    ) -> Self {
        VocabFuzzyMatch {
            awesome_person_repo,
            vocab_study_repo,
        }
    }
}

/// An implementation of the LearnVocabs service. Using fuzzy logic to check word
/// matching calculate the distance, The distance from a perfect match to decides how to change the vocab
/// correctness.
impl LearnVocab for VocabFuzzyMatch {

    /// Implementation, see trait for details [`LearnVocab::get_vocab_to_learn`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests in this module.
    fn get_vocab_to_learn(&self, awesome_id: i32, limit: i64) -> Result<Vec<(VocabStudy, Vocab)>, String> {
        let study_set = self.vocab_study_repo.get_study_set(awesome_id)?;

        // Separate tuples into two groups for prioritization.
        let (mut target_group, secondary_group): (Vec<_>, Vec<_>)
            = study_set.into_iter()
                .filter(|(_, v)| !v.first_lang.is_empty())
                .partition(|(vs, _)| vs.last_tested.is_some() && !vs.well_known);

        // Sorts the list by last_tested to find the most recently studied in the target group.
        target_group.sort_by(|(a_study, _), (b_study, _)| {
            b_study.last_tested.clone().unwrap_or_default().cmp(&a_study.last_tested.clone().unwrap_or_default())
        });

        // Grab more pairs from the secondary group as needed.
        if target_group.len() < limit as usize {
            target_group.extend(secondary_group.into_iter().take(limit as usize - target_group.len()));
        } else {
            target_group.truncate(limit as usize);
        }

        // Reverse the order to keep from presenting last word testing in the last set first in this set.
        target_group.reverse();

        Ok(target_group)
    }

    /// Implementation, see trait for details [`LearnVocab::check_vocab_match`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit tests in this module.
    fn check_vocab_match(
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

    /// Implementation, see trait for details [`LearnVocab::update_vocab_study_stats`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit tests in this module.
    fn update_vocab_study_stats(&self, vocab_study_id: i32,
                                awesome_person_id: i32,
                                distance: usize) -> Result<VocabStudy, String> {

        let current = self
            .vocab_study_repo
            .get_vocab_study_by_id(vocab_study_id)
            .map_err(|err| err.to_string())?;

        let updated_percentage_correct =
            self.calc_correctness(current.percentage_correct.unwrap_or_default(), distance);

        let last_change = updated_percentage_correct - current.percentage_correct.unwrap_or_default();

        let updating = VocabStudy {
            percentage_correct: Option::from(updated_percentage_correct),
            last_change: Option::from(last_change),
            last_tested: Option::from(Utc::now()),
            well_known: updated_percentage_correct > WELL_KNOWN_THRESHOLD,
            guesses: Option::from(current.guesses.unwrap_or_default() + 1),
            ..current
        };

        // Save changes to dal.
        self.vocab_study_repo
            .update_vocab_study(updating)
            .map_err(|err| err.to_string())?;
        let updated = self
            .vocab_study_repo
            .get_vocab_study_by_id(vocab_study_id)
            .map_err(|err| err.to_string())?;

        // Update the global stats too.
        self.update_overall_progress(awesome_person_id, distance == 0, updated.well_known.clone())?;
        Ok(updated)
    }

    /// Implementation, see trait for details [`LearnVocab::update_overall_progress`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit tests in this module.
    fn update_overall_progress(
        &self,
        awesome_person_id: i32,
        correct: bool,
        last_fully_known: bool,
    ) -> Result<AwesomePerson, String> {
        let current = self
            .awesome_person_repo
            .get_awesome_person_by_id(awesome_person_id)
            .map_err(|err| err.to_string())?;

        // Increment counters based on whether the answer was correct
        let (num_correct, num_incorrect) = (
            current.num_correct.unwrap_or(0) + correct as i32,
            current.num_incorrect.unwrap_or(0) + (!correct) as i32,
        );

        // Calculate the total percentage
        let total_percentage = num_correct as f64 / (num_correct + num_incorrect) as f64;

        // Prepare the updated stats
        let updating = AwesomePerson {
            num_known: if last_fully_known { Some(current.num_known.unwrap_or(0) + 1) } else { current.num_known },
            num_correct: Some(num_correct),
            num_incorrect: Some(num_incorrect),
            total_percentage: Some(total_percentage),
            updated: Utc::now(),
            ..current
        };

        // Update the stats and return the updated record
        self.awesome_person_repo.update_awesome_person(updating)
            .map_err(|err| err.to_string())?;
        
        self.awesome_person_repo.get_awesome_person_by_id(awesome_person_id)
            .map_err(|err| err.to_string())
    }

    /// Implementation, see trait for details [`LearnVocab::calc_correctness`]
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

    /// Implementation, see trait for details [`LearnVocab::determine_prompt`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit tests in this module.
    fn determine_prompt(&self, vocab: &Vocab, user_notes: &str) -> String {
        let mut prompt = format!("Translate: '{}'",  &vocab.first_lang);
        if !vocab.hint.clone().unwrap_or_default().is_empty() {
            prompt =  format!("{}    hint: {}",  prompt, vocab.hint.clone().unwrap_or_default());
        }

        if !vocab.pos.clone().unwrap_or_default().is_empty() {
            prompt =  format!("{}    pos: {}",  prompt, vocab.pos.clone().unwrap_or_default());
        }

        if !user_notes.is_empty() {
            prompt =  format!("{}    your notes: {}",  prompt, user_notes);
        }

        prompt
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::fixture_setup;
    use super::*;

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

        let fuzzy_service = fixture_setup().fuzzy_service;

        for (learning_lang, alternatives, guess, expected) in test_cases {
            let result = fuzzy_service.check_vocab_match(
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

        let fuzzy_service = fixture_setup().fuzzy_service;

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

        let fuzzy_service = fixture_setup().fuzzy_service;

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
        let fuzzy_service = fixture_setup().fuzzy_service;

        let awesome_person_id = 1;
        let correct = true;
        let last_fully_known = false;
        match fuzzy_service.update_overall_progress(awesome_person_id, correct, last_fully_known) {
            Ok(updated_progress) => println!("Updated progress stats: {}", updated_progress.id),
            Err(e) => println!("Error updating progress stats: {}", e),
        }
    }

    #[test]
    fn unit_test_update_pair_stats() {
        let vocab_study_id = 1;
        let awesome_person_id = 1;
        let distance = 2; // The guess was close, but not perfect

        let fuzzy_service = fixture_setup().fuzzy_service;

        match fuzzy_service.update_vocab_study_stats(vocab_study_id, awesome_person_id, distance) {
            Ok(updated_pair) => println!(
                "Updated vocab stats: {:?}, well known: {}",
                updated_pair.percentage_correct, updated_pair.well_known
            ),
            Err(e) => println!("Error updating vocab stats: {}", e),
        }
    }

    #[test]
    fn unit_test_check_pair_match() {
        let fuzzy_service = fixture_setup().fuzzy_service;
        // Test a perfect guess
        let learning_lang = "La gata es muy inteligente".to_string(); // The word to learn
        let alternatives = "La felina es muy inteligente".to_string(); // Alternative correct answers
        let guess = learning_lang.clone(); // A perfect guess
        let distance = fuzzy_service.check_vocab_match(&learning_lang, &alternatives, &guess);
        assert_eq!(
            distance, 0,
            "A perfect guess should return a distance of 0."
        );

        // Demonstrating the effect of a close, but not perfect, guess
        let close_guess = "La gata es muy perezosa".to_string();
        let distance_for_close_guess =
            fuzzy_service.check_vocab_match(&learning_lang, &alternatives, &close_guess);
        println!("Distance for a close guess: {}", distance_for_close_guess);
        // Expecting a small distance greater than 0 but less than MAX_DISTANCE

        // Demonstrating the effect of a guess with no similarity
        let no_similarity_guess = "This isn't even spanish!".to_string();
        let distance_for_no_similarity =
            fuzzy_service.check_vocab_match(&learning_lang, &alternatives, &no_similarity_guess);
        assert_eq!(
            distance_for_no_similarity, MAX_DISTANCE,
            "A guess with no similarity should return the maximum distance."
        );
    }

    #[test]
    fn unit_test_determine_prompt() {
        let fuzzy_service = fixture_setup().fuzzy_service;

        // Define test cases
        let test_cases = vec![
            (
                Vocab {
                    first_lang: "amor".to_string(),
                    hint: Some("noun".to_string()),
                    pos: Some("love".to_string()),
                    ..Default::default()
                },
                "",
                "Translate: 'amor'    hint: noun    pos: love".to_string(),
            ),
            (
                Vocab {
                    first_lang: "correr".to_string(),
                    hint: None,
                    pos: Some("verb".to_string()),
                    ..Default::default()
                },
                "",
                "Translate: 'correr'    pos: verb".to_string(),
            ),
            (
                Vocab {
                    first_lang: "amarillo".to_string(),
                    hint: Some("color".to_string()),
                    pos: None,
                    ..Default::default()
                },
                "",
                "Translate: 'amarillo'    hint: color".to_string(),
            ),
            (
                Vocab {
                    first_lang: "libro".to_string(),
                    hint: None,
                    pos: None,
                    ..Default::default()
                },
                "",
                "Translate: 'libro'".to_string(),
            ),
            (
                Vocab {
                    first_lang: "libro".to_string(),
                    ..Default::default()
                },
                "something you read",
                "Translate: 'libro'    your notes: something you read".to_string(),
            ),
        ];

        // Run test cases
        for (pair, user_notes, expected_prompt) in test_cases {
            let prompt = fuzzy_service.determine_prompt(&pair, user_notes);
            assert_eq!(prompt, expected_prompt, "Prompt did not match expected value for Vocab");
        }

    }
}
