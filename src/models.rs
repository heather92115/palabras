use crate::schema::palabras::progress_stats;
use crate::schema::palabras::translation_pair;
use chrono::prelude::*;
use diesel::prelude::*;

#[derive(Queryable, QueryableByName, Selectable, Identifiable, AsChangeset, Clone)]
#[diesel(table_name = translation_pair)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TranslationPair {
    pub id: i32,               // Primary key used to look up pair in the data layer.
    pub learning_lang: String, // Word in new language to be learned.
    pub first_lang: String,    // Word in fist language used to prompt for learning_lang.
    pub percentage_correct: Option<f64>, // The percentage of correct guesses calculated using the distance from the correct match.
    pub created: DateTime<Utc>, // Assuming `created` is set automatically to NOW(), you might not need this field during insertion.
    pub last_tested: Option<DateTime<Utc>>, // The last time this pair was guessed or tested.
    pub fully_known: bool,      // If the pair is known, it is skipped to focus on new pairs
    pub guesses: Option<i32>,   // The number of times this pair was attempted.
    pub alternatives: Option<String>, // Alternate answers in the learning language, also use to match answers.
    pub skill: Option<String>, // Alternate answers in the learning language, also use to match answers.
    pub too_easy: bool,        // Pair is too easy and never presented to the user.
    pub infinitive: Option<String>, // If the word is a verb, the infinitive form, otherwise none.
    pub pos: Option<String>, // Part of Speech, used to decide how to handle imports of first language.
    pub hint: Option<String>, // Hint to be displayed to help user translate words.
    pub user_notes: Option<String>, // Hint supplied by user to be displayed to help user translate words.
}

impl Default for TranslationPair {
    fn default() -> Self {
        Self {
            id: Default::default(),
            learning_lang: Default::default(), // Defaults to an empty String
            first_lang: Default::default(),    // Defaults to an empty String
            percentage_correct: None,          // Option types default to None
            created: Utc::now(),               // Set a default creation time
            last_tested: None,                 // Option types default to None
            fully_known: Default::default(),   // Defaults to false for bool
            guesses: Default::default(),       // Defaults to 0
            alternatives: Default::default(),  // Defaults to ""
            skill: Default::default(),         // Defaults to ""
            too_easy: Default::default(),      // Defaults to false for bool
            infinitive: Default::default(),    // Defaults to ""
            pos: Default::default(),           // Defaults to ""
            hint: Default::default(),          // Defaults to ""
            user_notes: Default::default(),   // Defaults to ""
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = translation_pair)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTranslationPair {
    pub learning_lang: String,           // Word in new language to be learned.
    pub first_lang: String,              // Word in fist language used to prompt for learning_lang.
    pub percentage_correct: Option<f64>, // The percentage of correct guesses calculated using the distance from the correct match.
    pub created: DateTime<Utc>, // Assuming `created` is set automatically to NOW(), you might not need this field during insertion.
    pub last_tested: Option<DateTime<Utc>>, // The last time this pair was guessed or tested.
    pub fully_known: bool,      // If the pair is known, it is skipped to focus on new pairs
    pub guesses: Option<i32>,   // The number of times this pair was attempted.
    pub alternatives: Option<String>, // Alternate answers in the learning language, also use to match answers.
    pub skill: Option<String>, // Alternate answers in the learning language, also use to match answers.
    pub too_easy: bool,        // Pair is too easy and never presented to the user.
    pub infinitive: Option<String>, // If the word is a verb, the infinitive form, otherwise none.
    pub pos: Option<String>, // Part of Speech, used to decide how to handle imports of first language.
    pub hint: Option<String>, // Hint to be displayed to help user translate words.
    pub user_notes: Option<String>, // Hint supplied by user to be displayed to help user translate words.
}

impl Default for NewTranslationPair {
    fn default() -> Self {
        Self {
            learning_lang: Default::default(), // Defaults to an empty String
            first_lang: Default::default(),    // Defaults to an empty String
            percentage_correct: None,          // Option types default to None
            created: Utc::now(),               // Set a default creation time
            last_tested: None,                 // Option types default to None
            fully_known: Default::default(),   // Defaults to false for bool
            guesses: Default::default(),       // Defaults to 0
            alternatives: Default::default(),  // Defaults to ""
            skill: Default::default(),         // Defaults to ""
            too_easy: Default::default(),      // Defaults to false for bool
            infinitive: Default::default(),    // Defaults to ""
            pos: Default::default(),           // Defaults to ""
            hint: Default::default(),          // Defaults to ""
            user_notes: Default::default(),   // Defaults to ""
        }
    }
}

#[derive(Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = progress_stats)]
pub struct ProgressStats {
    pub id: i32,                       // Primary key used to look up stats in the data layer.
    pub num_known: Option<i32>,        // Number of pairs moved to fully known state.
    pub num_correct: Option<i32>,      // Total number of correct guesses.
    pub num_incorrect: Option<i32>,    // Total number of incorrect guesses.
    pub total_percentage: Option<f64>, // Percentage guess correctly.
    pub updated: DateTime<Utc>,        // Last time stats where updated, (after each guess).
}
