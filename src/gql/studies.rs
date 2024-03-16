use crate::sl::fuzzy_match_vocab::{LearnVocab, VocabFuzzyMatch};
use async_graphql::*;

/// Represents a challenge presented to a user for vocabulary practice.
///
/// Each challenge is generated based on the user's learning history and targets specific vocabulary
/// that the user is currently studying. It includes a prompt that may consist of a word or phrase
/// in the target language, a sentence for translation, or any other form of query designed to
/// test the user's knowledge and recall of the vocabulary.
///
/// # Fields
///
/// - `vocab_id`: The unique identifier of the vocabulary item being challenged. This relates to a specific
/// word or phrase in the study material.
/// - `vocab_study_id`: The unique identifier for the user's study history with this vocabulary item,
/// allowing for tracking of progress and retrieval of user-specific study data.
/// - `first_lang`: The translation of the word or phrase into the user's first language, used as a prompt.
/// - `infinitive`: Optional. For verbs, the infinitive form of the word. Empty for non-verb vocabulary items.
/// - `pos`: Optional. The part of speech of the vocabulary item, aiding in the application of grammatical rules.
/// - `hint`: Optional. A hint provided to assist users in translating the word or phrase.
/// - `num_learning_words`: The number of words contained in the `learning_lang` field, calculated for analytical purposes.
/// - `user_notes`: Optional notes added by the user to aid in recall or provide additional context for the vocabulary word
/// - `correct_attempts`: The number of times the vocabulary word was guessed or recalled correctly by the user.
/// - `known_lang_code`: Language code for this known language.
/// - `learning_lang_code`: Language code for this learning language.
#[derive(Clone)]
pub struct Challenge {
    pub vocab_id: i32,
    pub vocab_study_id: i32,
    pub first_lang: String,
    pub infinitive: String,
    pub pos: String,
    pub hint: String,
    pub num_learning_words: i32,
    pub user_notes: String,
    pub correct_attempts: i32,
    pub known_lang_code: String,
    pub learning_lang_code: String,
}

#[Object]
impl Challenge {
    async fn vocab_id(&self) -> i32 {
        self.vocab_id
    }

    async fn vocab_study_id(&self) -> i32 {
        self.vocab_study_id
    }

    async fn first_lang(&self) -> String {
        self.first_lang.clone()
    }
    async fn infinitive(&self) -> String {
        self.infinitive.clone()
    }
    async fn pos(&self) -> String {
        self.pos.clone()
    }
    async fn hint(&self) -> String {
        self.hint.clone()
    }
    async fn num_learning_words(&self) -> i32 {
        self.num_learning_words.clone()
    }
    async fn user_notes(&self) -> String {
        self.user_notes.clone()
    }
    async fn correct_attempts(&self) -> i32 {
        self.correct_attempts.clone()
    }

    async fn known_lang_code(&self) -> String {
        self.known_lang_code.clone()
    }

    async fn learning_lang_code(&self) -> String {
        self.learning_lang_code.clone()
    }
}

/// Represents the profile of an awesome person with their vocabulary learning statistics.
///
/// This struct is used to encapsulate the learning progress of an individual, tracking both
/// their successes and areas for improvement in vocabulary study. It includes a variety of
/// metrics such as the total number of known words, correct and incorrect guesses, as well
/// as an overall success percentage. Additionally, it provides personal information such as the
/// user's name and a threshold for the smallest vocabulary word considered for testing.
///
/// # Fields
///
/// - `id`: The unique identifier of the awesome person. This serves as the primary key for lookup in the data layer.
/// - `num_known`: The number of vocabulary pairs that the user has fully mastered or known.
/// - `num_correct`: The total number of correct responses or guesses made by the user across all vocabulary tests.
/// - `num_incorrect`: The total number of incorrect responses or guesses made by the user across all vocabulary tests.
/// - `total_percentage`: The overall success rate calculated as the percentage of correct guesses out of the total number of guesses.
/// - `name`: The name of the user. This field is optional and can be anything the user wants.
/// - `smallest_vocab`: The minimum length of vocabulary words that are considered for testing. This helps tailor the difficulty of the tests to the user's level.
///
/// # Example
///
/// ```
/// use palabras::gql::studies::AwesomeProfile;
/// let awesome_profile = AwesomeProfile {
///     id: 1,
///     num_known: 150,
///     num_correct: 200,
///     num_incorrect: 50,
///     total_percentage: 80.0,
///     name: String::from("Michelle"),
///     smallest_vocab: 4,
/// };
/// ```
#[derive(Clone)]
pub struct AwesomeProfile {
    pub id: i32,
    pub num_known: i32,
    pub num_correct: i32,
    pub num_incorrect: i32,
    pub total_percentage: f64,
    pub name: String,
    pub smallest_vocab: i32,
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

/// Represents the statistical data related to the study of a specific vocabulary word.
///
/// This struct encapsulates the learning metrics for a particular vocabulary word, providing insight
/// into the user's success rates. It includes the total number of attempts,
/// the success rate, and the most recent changes in performance, giving a comprehensive view of
/// the learning progress for that word.
///
/// # Fields
///
/// - `learning`: The vocabulary word being studied. This field represents the target word or phrase that the user is attempting to learn.
/// - `attempts`: The total number of attempts made by the user to study or guess the `learning` word.
/// This metric helps track the amount of effort put into learning the word.
/// - `correct_attempts`: The number of successful attempts where the user correctly guessed or recalled
/// the `learning` word. This measures the effectiveness of the learning process.
/// - `percentage_correct`: The success rate for the `learning` word, calculated as the percentage of
/// correct attempts out of the total attempts. This provides a quantitative measure of the user's mastery over the word.
/// - `last_change`: Indicates the most recent change in the success rate (`percentage_correct`).
/// This metric can help identify recent trends in the user's learning curve, such as improvements or setbacks.
/// - `last_tested`: The timestamp of the last attempt to study the `learning` word. This field helps
/// track the recency of the user's study efforts and can be used to prompt further review if too much time has elapsed.
/// # Example
///
/// ```use palabras::gql::studies::VocabStats;
/// let vocab_stats = VocabStats {
///     learning: String::from("palabra"),
///     attempts: 10,
///     correct_attempts: 8,
///     percentage_correct: 80.0,
///     last_change: 5.0,
///     last_tested: String::from("2022-03-21 15:00:00"),
/// };
/// ```
#[derive(Clone)]
pub struct VocabStats {
    pub learning: String,
    pub attempts: i32,
    pub correct_attempts: i32,
    pub percentage_correct: f64,
    pub last_change: f64,
    pub last_tested: String,
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
    async fn get_study_list(&self, awesome_id: i32, limit: i64) -> Result<Vec<Challenge>> {
        let match_service = VocabFuzzyMatch::instance();

        let mut study_list: Vec<Challenge> = Vec::new();

        let vocab = match_service.get_vocab_to_learn(awesome_id, limit)?;
        for (vs, v) in vocab {
            study_list.push(Challenge {
                vocab_id: v.id,
                vocab_study_id: vs.id,
                first_lang: v.first_lang,
                infinitive: v.infinitive.unwrap_or_default(),
                pos: v.pos.unwrap_or_default(),
                hint: v.hint.unwrap_or_default(),
                num_learning_words: v.num_learning_words,
                user_notes: vs.user_notes.unwrap_or_default(),
                correct_attempts: vs.correct_attempts.unwrap_or_default(),
                known_lang_code: v.known_lang_code,
                learning_lang_code: v.learning_lang_code,
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
            vocab_study
                .last_tested
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S %Z")
                .to_string()
        } else {
            "".to_string()
        };

        Ok(VocabStats {
            learning: vocab.learning_lang.clone(),
            attempts: vocab_study.attempts.unwrap_or_default(),
            correct_attempts: vocab_study.correct_attempts.unwrap_or_default(),
            percentage_correct: vocab_study.percentage_correct.unwrap_or_default(),
            last_change: vocab_study.last_change.unwrap_or_default(),
            last_tested,
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
