use crate::sl::fuzzy_match_vocab::{LearnVocab, VocabFuzzyMatch};
use async_graphql::*;

#[derive(Clone)]
struct Challenge {
    vocab_id: i32,
    vocab_study_id: i32,
    prompt: String,
}

#[Object]
impl Challenge {
    async fn vocab_id(&self) -> i32 {
        self.vocab_id
    }

    async fn vocab_study_id(&self) -> i32 {
        self.vocab_study_id
    }

    async fn prompt(&self) -> String {
        self.prompt.clone()
    }
}

#[derive(Clone)]
struct AwesomeProfile {
    id: i32,               // Primary key used to look up stats in the data layer.
    num_known: i32,        // Number of pairs moved to fully known state.
    num_correct: i32,      // Total number of correct guesses.
    num_incorrect: i32,    // Total number of incorrect guesses.
    total_percentage: f64, // Percentage guess correctly.
    name: String,          // User's name, optional
    smallest_vocab: i32,   // Size of the smallest vocab word to be tested.
}

#[Object]
impl AwesomeProfile {
    async fn id(&self) -> i32 {
        self.id
    }
    async fn num_known(&self) -> i32 {
        self.num_known
    }
    async fn num_correct(&self) -> i32 {
        self.num_correct
    }
    async fn num_incorrect(&self) -> i32 {
        self.num_incorrect
    }
    async fn total_percentage(&self) -> f64 {
        self.total_percentage
    }
    async fn name(&self) -> String {
        self.name.clone()
    }
    async fn smallest_vocab(&self) -> i32 {
        self.smallest_vocab
    }
}

#[derive(Clone)]
struct VocabStats {
    learning: String,        // The word being studied.
    attempts: i32,           // The number of times this vocab was attempted.
    correct_attempts: i32,   // The number of times this vocab was correctly attempted.
    percentage_correct: f64, // The percentage of correct guesses calculated using the distance from the correct match.
    last_change: f64,        // The most recent percentage correct change
    last_tested: String,     // The last time this pair was attempted.
}

#[Object]
impl VocabStats {

    async fn learning(&self) -> String {
        self.learning.clone()
    }

    async fn attempts(&self) -> i32 {
        self.attempts
    }

    async fn correct_attempts(&self) -> i32 {
        self.correct_attempts
    }

    async fn percentage_correct(&self) -> f64 {
        self.percentage_correct
    }

    async fn last_change(&self) -> f64 {
        self.last_change
    }

    async fn last_tested(&self) -> String {
        self.last_tested.clone()
    }

}

/// GraphQL Queries
pub struct QueryRoot;

#[Object]
impl QueryRoot {

    /// Fetches a list of vocab study challenges for a specified awesome person.
    ///
    /// This async function retrieves a set of vocab words for the awesome person to study,
    /// limited by the specified `limit`. Each challenge includes a prompt generated based
    /// on the vocab word and any user notes associated with the vocab study.
    ///
    /// # Arguments
    ///
    /// * `awesome_id` - The ID of the awesome person for whom to fetch the study challenges.
    /// * `limit` - The maximum number of challenges to return.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `Challenge` structs on success, or an error message string on failure.
    /// Each `Challenge` struct includes the vocab ID, vocab study ID, and the generated prompt.
    async fn study(&self, awesome_id: i32, limit: i64) -> Result<Vec<Challenge>> {
        let match_service = VocabFuzzyMatch::instance();

        let mut study_list: Vec<Challenge> = Vec::new();

        let vocab = match_service.get_vocab_to_learn(awesome_id, limit)?;
        for (vs, v) in vocab {
            let prompt = match_service.determine_prompt(&v, &vs.user_notes.unwrap_or_default());
            study_list.push(Challenge {
                vocab_id: v.id,
                vocab_study_id: vs.id,
                prompt,
            });
        }

        Ok(study_list)
    }

    /// Retrieves detailed profile information for an awesome person by their ID.
    ///
    /// This async function queries the database for the specified awesome person's data,
    /// including their learning statistics and basic profile details. If the awesome person
    /// cannot be found, it returns default values for each field.
    ///
    /// # Arguments
    ///
    /// * `awesome_id` - The unique identifier of the awesome person whose profile is being requested.
    ///
    /// # Returns
    ///
    /// A `Result` wrapping an `AwesomeProfile` struct containing the awesome person's data on success,
    /// or an error message string on failure.
    async fn get_awesome_person(&self, awesome_id: i32) -> Result<AwesomeProfile> {
        let match_service = VocabFuzzyMatch::instance();
        let pub_awesome_person = match_service.get_awesome_person(awesome_id)?;
        let pub_awesome_person = pub_awesome_person.unwrap_or_default();

        Ok(AwesomeProfile {
            id: pub_awesome_person.id,
            num_known: pub_awesome_person.num_known.unwrap_or_default(),
            num_correct: pub_awesome_person.num_correct.unwrap_or_default(),
            num_incorrect: pub_awesome_person.num_incorrect.unwrap_or_default(),
            total_percentage: pub_awesome_person.total_percentage.unwrap_or_default(),
            name: pub_awesome_person.name.unwrap_or_default(),
            smallest_vocab: pub_awesome_person.smallest_vocab,
        })
    }

    /// Retrieves statistical information for a specific vocabulary study session by its ID.
    ///
    /// This async function looks up the study session for a particular vocabulary word and compiles
    /// key statistics about the user's attempts, successes, and overall performance with that word.
    /// It also includes the last tested time for the vocabulary, formatted as a readable string.
    ///
    /// # Arguments
    ///
    /// * `vocab_study_id` - The unique identifier of the vocabulary study session.
    ///
    /// # Returns
    ///
    /// A `Result` wrapping a `VocabStats` struct containing detailed statistics about the study session on success,
    /// or an error string on failure.
    async fn get_vocab_stats(&self, vocab_study_id: i32) -> Result<VocabStats> {

        let match_service = VocabFuzzyMatch::instance();

        let (vocab_study, vocab) = match_service.get_vocab_stats(vocab_study_id)?;

        let last_tested = if vocab_study.last_tested.is_some() {
            vocab_study.last_tested.unwrap().format("%Y-%m-%d %H:%M:%S %Z").to_string()
        } else {
            "".to_string()
        };

        Ok(VocabStats {
            learning: vocab.learning_lang.clone(),
            attempts: vocab_study.attempts.unwrap_or_default(),
            correct_attempts: vocab_study.correct_attempts.unwrap_or_default(),
            percentage_correct: vocab_study.percentage_correct.unwrap_or_default(),
            last_change: vocab_study.last_change.unwrap_or_default(),
            last_tested
        })
    }
}

/// GraphQL Mutations
pub struct MutationRoot;

#[Object]
impl MutationRoot {

    /// Checks the user's response for a given vocabulary study session.
    ///
    /// This function compares the user's entered response against the correct answer for the specified vocabulary.
    /// It leverages the `VocabFuzzyMatch` service to assess the accuracy of the response and provides feedback.
    ///
    /// # Arguments
    ///
    /// * `vocab_id` - The identifier of the vocabulary item being studied.
    /// * `vocab_study_id` - The identifier of the vocab study session, linking the user and the vocab item.
    /// * `entered` - The response entered by the user for the vocabulary item.
    ///
    /// # Returns
    ///
    /// Returns a `Result<String>` where:
    /// - `Ok(String)` contains the feedback or prompt based on the comparison of the entered response and the correct answer.
    /// - `Err` contains an error message if the operation fails.
    async fn check_response(
        &self,
        vocab_id: i32,
        vocab_study_id: i32,
        entered: String,
    ) -> Result<String> {
        let match_service = VocabFuzzyMatch::instance();

        let prompt = match_service.check_response(vocab_id, vocab_study_id, entered)?;

        Ok(prompt)
    }
}
