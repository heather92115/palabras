use crate::dal::awesome_person::{AwesomePersonRepository, DbAwesomePersonRepository};
use crate::dal::vocab::{DbVocabRepository, VocabRepository};
use crate::dal::vocab_study::{DbVocabStudyRepository, VocabStudyRepository};
use crate::models::{AwesomePerson, Vocab, VocabStudy};
use chrono::Utc;
use core::option::Option;
use lazy_static::lazy_static;
use std::sync::{Mutex, MutexGuard};
use strsim::levenshtein;

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
    fn get_vocab_to_learn(
        &self,
        awesome_id: i32,
        limit: i64,
    ) -> Result<Vec<(VocabStudy, Vocab)>, String>;

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

    /// Checks the provided response against the correct answer for a given vocabulary item and updates statistics accordingly.
    ///
    /// This function takes the identifiers for a vocabulary item and its study record, along with the user's response,
    /// to perform a fuzzy match checking how close the response is to the correct answer. It updates both the specific vocabulary
    /// study statistics and the overall progress statistics for the awesome person associated with the vocab study.
    ///
    /// # Parameters
    /// - `vocab_id`: The identifier for the vocabulary item being studied.
    /// - `vocab_study_id`: The identifier for the vocabulary study record.
    /// - `response`: The user's response as a `String`.
    ///
    /// # Returns
    /// - `Ok(String)`: A string indicating the result of the match. Can provide feedback such as a perfect match, close match, or incorrect match.
    /// - `Err(String)`: An error message if any step in the process fails.
    ///
    /// # Errors
    /// This function returns an error if:
    /// - It fails to retrieve the vocabulary item based on the provided `vocab_id`.
    /// - There are issues updating the vocabulary study statistics or the overall progress.
    ///
    /// This function is intended to be used as part of a vocabulary learning application where users are presented
    /// with vocabulary words to translate or identify. The function assesses the accuracy of their responses and
    /// updates their learning progress accordingly.
    fn check_response(
        &self,
        vocab_id: i32,
        vocab_study_id: i32,
        response: String,
    ) -> Result<String, String>;

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
    /// * `vocab_study_id` - The primary key (`id`) of the vocab to update.
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
    fn update_vocab_study_stats(
        &self,
        vocab_study_id: i32,
        distance: usize,
    ) -> Result<VocabStudy, String>;

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
    ) -> Result<Option<AwesomePerson>, String>;

    /// Determines the match prompt based on the distance between the correct answer and the user's response.
    ///
    /// This function takes the correct answer, the user's response, and the Levenshtein distance between them.
    /// It returns a string indicating the quality of the match.
    ///
    /// # Parameters
    /// - `correct`: The correct answer as a string slice.
    /// - `user_response`: The user's response as a string slice.
    /// - `distance`: The Levenshtein distance between the correct answer and the user's response, as an usize.
    ///
    /// # Returns
    /// A `String` that provides feedback on how close the user's response was to the correct answer.
    /// - Returns "Perfect Match!" if the distance is 0.
    /// - Returns "Close, it was '[correct]', you entered '[user_response]'" if the distance is 3 or less.
    /// - Otherwise, returns "It was '[correct]', you entered '[user_response]'".
    fn determine_match_prompt(&self, correct: &str, user_response: &str, distance: usize)
        -> String;

    /// Retrieves a single awesome person record by its primary key.
    ///
    /// # Parameters
    ///
    /// * `id` - The primary key (`id`) of the awesome person record to retrieve.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(AwesomePerson))` if an awesome person record with the specified `id` exists,
    /// Ok(None) if not found or an error if the query fails.
    fn get_awesome_person(&self, awesome_person_id: i32) -> Result<Option<AwesomePerson>, String>;

    /// Retrieves a single tuple of vocab study and vocab by the vocab study id.
    ///
    /// # Parameters
    ///
    /// * `vocab_study_id` - The primary key (`vocab_study_id`) of the vocab study record to retrieve.
    ///
    /// # Returns
    ///
    /// Returns `Ok((VocabStudy, Vocab))` if the both records were found.
    /// Err if either are not found or if the query fails.
    fn get_vocab_stats(&self, vocab_study_id: i32) -> Result<(VocabStudy, Vocab), String>;
}

pub struct VocabFuzzyMatch {
    awesome_person_repo: Box<dyn AwesomePersonRepository>,
    vocab_study_repo: Box<dyn VocabStudyRepository>,
    vocab_repo: Box<dyn VocabRepository>,
}

lazy_static! {
    static ref FUZZY_MATCH_SERVICE: Mutex<VocabFuzzyMatch> = Mutex::new(VocabFuzzyMatch::new(
        Box::new(DbAwesomePersonRepository),
        Box::new(DbVocabStudyRepository),
        Box::new(DbVocabRepository),
    ));
}

impl VocabFuzzyMatch {
    // The constructor takes Box<dyn Repos>
    pub fn new(
        awesome_person_repo: Box<dyn AwesomePersonRepository>,
        vocab_study_repo: Box<dyn VocabStudyRepository>,
        vocab_repo: Box<dyn VocabRepository>,
    ) -> Self {
        VocabFuzzyMatch {
            awesome_person_repo,
            vocab_study_repo,
            vocab_repo,
        }
    }

    // Method to access the singleton instance, but one thread at a time
    pub fn instance() -> MutexGuard<'static, VocabFuzzyMatch> {
        FUZZY_MATCH_SERVICE.lock().unwrap()
    }
}

/// An implementation of the LearnVocabs service. Using fuzzy logic to check word
/// matching calculate the distance, The distance from a perfect match to decides how to change the vocab
/// correctness.
impl LearnVocab for VocabFuzzyMatch {
    /// Implementation, see trait for details [`LearnVocab::get_vocab_to_learn`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit integration tests in this module.
    fn get_vocab_to_learn(
        &self,
        awesome_id: i32,
        limit: i64,
    ) -> Result<Vec<(VocabStudy, Vocab)>, String> {
        // TODO limit the number of results returned by the db, perhaps with a MV.
        let study_set = self.vocab_study_repo.get_study_set(awesome_id)?;

        // Separate tuples into two groups for prioritization.
        let (mut target_group, secondary_group): (Vec<_>, Vec<_>) = study_set
            .into_iter()
            .filter(|(_, v)| !v.first_lang.is_empty())
            .partition(|(vs, _)| vs.last_tested.is_some() && !vs.well_known);

        // Sorts the list by last_tested to find the most recently studied in the target group.
        target_group.sort_by(|(a_study, _), (b_study, _)| {
            b_study
                .last_tested
                .clone()
                .unwrap_or_default()
                .cmp(&a_study.last_tested.clone().unwrap_or_default())
        });

        // Grab more pairs from the secondary group as needed.
        if target_group.len() < limit as usize {
            target_group.extend(
                secondary_group
                    .into_iter()
                    .take(limit as usize - target_group.len()),
            );
        } else {
            target_group.truncate(limit as usize);
        }

        // Reverse the order to keep from presenting last word testing in the last set first in this set.
        target_group.reverse();

        // Returning a curated vocab lesson
        Ok(target_group)
    }

    /// Implementation, see trait for details [`LearnVocab::determine_prompt`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit tests in this module.
    fn determine_prompt(&self, vocab: &Vocab, user_notes: &str) -> String {
        let mut prompt = format!("Translate: '{}'", &vocab.first_lang);
        if !vocab.hint.clone().unwrap_or_default().is_empty() {
            prompt = format!(
                "{}    hint: {}",
                prompt,
                vocab.hint.clone().unwrap_or_default()
            );
        }

        if !vocab.pos.clone().unwrap_or_default().is_empty() {
            prompt = format!(
                "{}    pos: {}",
                prompt,
                vocab.pos.clone().unwrap_or_default()
            );
        }

        if !user_notes.is_empty() {
            prompt = format!("{}    your notes: {}", prompt, user_notes);
        }

        prompt
    }

    /// Implementation, see trait for details [`LearnVocab::check_response`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit and integration tests for this module.
    fn check_response(
        &self,
        vocab_id: i32,
        vocab_study_id: i32,
        response: String,
    ) -> Result<String, String> {
        // Get the vocab containing the possible correct responses.
        let vocab = self
            .vocab_repo
            .get_vocab_by_id(vocab_id)
            .map_err(|e| e.to_string())?;

        // Use the fuzzy matching logic to see how much "distance" the response, 0 is correct.
        let distance = self.check_vocab_match(
            &vocab.learning_lang,
            &vocab.alternatives.unwrap_or_default(),
            &response,
        );

        // Update the awesome person's stats for this vocab word.
        let vocab_study = self.update_vocab_study_stats(vocab_study_id, distance)?;

        // Update the awesome person's overall status.
        self.update_overall_progress(
            vocab_study.awesome_person_id,
            distance == 0,
            vocab_study.well_known.clone(),
        )?;

        // For the response text to be displayed to the awesome person
        Ok(self.determine_match_prompt(&vocab.learning_lang, &response, distance))
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
    fn update_vocab_study_stats(
        &self,
        vocab_study_id: i32,
        distance: usize,
    ) -> Result<VocabStudy, String> {
        let current = self
            .vocab_study_repo
            .get_vocab_study_by_id(vocab_study_id)
            .map_err(|err| err.to_string())?;

        let updated_percentage_correct =
            self.calc_correctness(current.percentage_correct.unwrap_or_default(), distance);

        let last_change =
            updated_percentage_correct - current.percentage_correct.unwrap_or_default();

        let updating = VocabStudy {
            percentage_correct: Option::from(updated_percentage_correct),
            last_change: Option::from(last_change),
            last_tested: Option::from(Utc::now()),
            well_known: updated_percentage_correct > WELL_KNOWN_THRESHOLD,
            attempts: Option::from(current.attempts.unwrap_or_default() + 1),
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

        Ok(updated)
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

    /// Implementation, see trait for details [`LearnVocab::update_overall_progress`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit tests in this module.
    fn update_overall_progress(
        &self,
        awesome_person_id: i32,
        correct: bool,
        last_fully_known: bool,
    ) -> Result<Option<AwesomePerson>, String> {
        let awesome_person = self
            .awesome_person_repo
            .get_awesome_person_by_id(awesome_person_id)
            .map_err(|err| err.to_string())?;

        let awesome_person = if awesome_person.is_some() {
            awesome_person.unwrap_or_default()
        } else {
            return Err(format!(
                "Failed to find awesome person with id {}",
                awesome_person_id
            ));
        };

        // Increment counters based on whether the answer was correct
        let (num_correct, num_incorrect) = (
            awesome_person.num_correct.unwrap_or(0) + correct as i32,
            awesome_person.num_incorrect.unwrap_or(0) + (!correct) as i32,
        );

        // Calculate the total percentage
        let total_percentage = num_correct as f64 / (num_correct + num_incorrect) as f64;

        // Prepare the updated stats
        let updating = AwesomePerson {
            num_known: if last_fully_known {
                Some(awesome_person.num_known.unwrap_or(0) + 1)
            } else {
                awesome_person.num_known
            },
            num_correct: Some(num_correct),
            num_incorrect: Some(num_incorrect),
            total_percentage: Some(total_percentage),
            updated: Utc::now(),
            ..awesome_person
        };

        // Update the stats and return the updated record
        self.awesome_person_repo
            .update_awesome_person(updating)
            .map_err(|err| err.to_string())?;

        self.awesome_person_repo
            .get_awesome_person_by_id(awesome_person_id)
            .map_err(|err| err.to_string())
    }

    /// Implementation, see trait for details [`LearnVocab::determine_match_prompt`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the unit tests in this module.
    fn determine_match_prompt(
        &self,
        correct: &str,
        user_response: &str,
        distance: usize,
    ) -> String {
        return if distance == 0 {
            "Perfect Match!".to_string()
        } else if distance <= 3 {
            format!(
                "Close, it was '{}', you entered '{}'",
                correct, user_response
            )
        } else {
            format!("It was '{}', you entered '{}'", correct, user_response)
        };
    }

    /// Implementation, see trait for details [`LearnVocab::get_awesome_person`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests in this module.
    fn get_awesome_person(&self, awesome_person_id: i32) -> Result<Option<AwesomePerson>, String> {
        let awesome_person = self
            .awesome_person_repo
            .get_awesome_person_by_id(awesome_person_id)
            .map_err(|e| e.to_string())?;

        // Get sec matters private
        let pub_awesome_person = AwesomePerson {
            code: Some("".to_string()),
            ..awesome_person.unwrap_or_default()
        };

        Ok(Some(pub_awesome_person))
    }

    /// Implementation, see trait for details [`LearnVocab::get_vocab_stats`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests in this module.
    fn get_vocab_stats(&self, vocab_study_id: i32) -> Result<(VocabStudy, Vocab), String> {
        let vocab_study = self
            .vocab_study_repo
            .get_vocab_study_by_id(vocab_study_id)
            .map_err(|e| e.to_string())?;

        let vocab = self
            .vocab_repo
            .get_vocab_by_id(vocab_study.vocab_id)
            .map_err(|e| e.to_string())?;

        Ok((vocab_study, vocab))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::fixture_setup;

    #[test]
    fn unit_test_get_vocab_to_learn() {
        // get the mocked service complete with mocked repos data test data
        let fuzzy_service = fixture_setup().fuzzy_service;
        let result = fuzzy_service
            .get_vocab_to_learn(1, 1)
            .expect("No issues expected with mocked data");
        assert!(result.len() >= 1, "Mocked data expected");
    }

    #[test]
    fn unit_test_determine_prompt() {
        // Note: the mocked repos aren't used in this test
        let fuzzy_service = fixture_setup().fuzzy_service;

        // Define test cases to check the prompt is as expected
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

        // Run test cases. Note: the mocked db connections aren't used with this method.
        for (pair, user_notes, expected_prompt) in test_cases {
            let prompt = fuzzy_service.determine_prompt(&pair, user_notes);
            assert_eq!(
                prompt, expected_prompt,
                "Prompt did not match expected value for Vocab"
            );
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

        // Note: the mocked repos aren't used in this test
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
        // Testing a miss, but the match was close
        let previous_correctness = 0.99; // 99% correctness prior to the latest guess
        let distance_for_latest_guess = 2; // The guess was fairly close, but not perfect

        let fuzzy_service = fixture_setup().fuzzy_service;

        let new_correctness =
            fuzzy_service.calc_correctness(previous_correctness, distance_for_latest_guess);
        assert!(
            new_correctness < previous_correctness,
            "Expected correctness to go down"
        );

        // Demonstrating the effect of a perfect guess
        let previous_correctness = 0.5; // 50% correctness prior to the latest guess
        let perfect_distance = 0; // A perfect guess
        let new_correctness =
            fuzzy_service.calc_correctness(previous_correctness, perfect_distance);
        assert!(
            new_correctness > previous_correctness,
            "Expected correctness to go up"
        );

        // Demonstrating a miss, but the guess was better than before
        let previous_correctness = 0.3; // 50% correctness prior to the latest guess
        let perfect_distance = 2; // A close guess
        let new_correctness =
            fuzzy_service.calc_correctness(previous_correctness, perfect_distance);
        assert!(
            new_correctness > previous_correctness,
            "Expected correctness to go up even on miss"
        );

        // Demonstrating a perfect guess with a previous low correctness percentage
        let previous_correctness = 0.1; // 50% correctness prior to the latest guess
        let perfect_distance = 0; // A perfect guess
        let new_correctness =
            fuzzy_service.calc_correctness(previous_correctness, perfect_distance);
        assert!(
            new_correctness > 0.5,
            "Expected correctness to to be above 0.5"
        );
    }

    #[test]
    fn unit_test_update_overall_progress() {
        let fuzzy_service = fixture_setup().fuzzy_service;

        let awesome_person_id = 1;
        let correct = true;
        let last_fully_known = false;
        let awesome_person = fuzzy_service
            .update_overall_progress(awesome_person_id, correct, last_fully_known)
            .expect("Expected default user");
        let _ = awesome_person.expect("Expected some value for default user");
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
    fn unit_test_match_prompt() {
        let fuzzy_service = fixture_setup().fuzzy_service;

        // (correct word, guessed, calculated distance, prompt)
        let test_cases = vec![
            ("palabra", "palabra", 0, "Perfect Match!"),
            (
                "palabra",
                "palabre",
                1,
                "Close, it was 'palabra', you entered 'palabre'",
            ),
            (
                "palabra",
                "idioma",
                6,
                "It was 'palabra', you entered 'idioma'",
            ),
        ];

        for (correct, guessed, distance, prompt) in test_cases {
            let actual = fuzzy_service.determine_match_prompt(correct, guessed, distance);
            assert!(
                actual.eq(prompt),
                "Expected {}, but got {} for parameters {}, {}, {}",
                correct,
                actual,
                guessed,
                distance,
                prompt
            );
        }
    }

    #[test]
    fn unit_test_check_response() {
        let fuzzy_service = fixture_setup().fuzzy_service;

        let vocab_test_data = fuzzy_service
            .vocab_repo
            .get_vocab_by_id(1)
            .expect("Mocked repo should have returned an instance of vocab");

        let vocab_study_test_data = fuzzy_service
            .vocab_study_repo
            .get_vocab_study_by_id(1)
            .expect("Mocked repo should have returned an instance of vocab study");

        // Test a perfect match
        let match_prompt = fuzzy_service
            .check_response(
                vocab_test_data.id,
                vocab_study_test_data.id,
                vocab_test_data.learning_lang.clone(),
            )
            .expect("No error results expected fn check_response with mocked repos");
        assert_eq!(
            match_prompt, "Perfect Match!",
            "Expected perfect match from mocked data, but actual prompt was {}",
            match_prompt
        );

        // Test an inaccurate answer, '123'
        let match_prompt = fuzzy_service
            .check_response(
                vocab_test_data.id,
                vocab_study_test_data.id,
                "123".to_string(),
            )
            .expect("No error results expected fn check_response with mocked repos");
        assert_ne!(
            match_prompt, "Perfect Match!",
            "Expected a miss from mocked data, but actual prompt was {}",
            match_prompt
        );

        // Test a close but incorrect answer
        let test_response = format!("{}a", vocab_test_data.learning_lang.clone());
        let match_prompt = fuzzy_service
            .check_response(vocab_test_data.id, vocab_study_test_data.id, test_response)
            .expect("No error results expected fn check_response with mocked repos");
        assert_ne!(
            match_prompt, "Perfect Match!",
            "Expected a miss from mocked data, but actual prompt was {}",
            match_prompt
        );
    }
}
