use crate::schema::palabras::awesome_person;
use crate::schema::palabras::vocab;
use crate::schema::palabras::vocab_study;
use chrono::prelude::*;
use diesel::prelude::*;

#[derive(Queryable, QueryableByName, Selectable, Identifiable, AsChangeset, Clone)]
#[diesel(table_name = vocab)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Vocab {
    pub id: i32,               // Primary key used to look up pair in the data layer.
    pub learning_lang: String, // Word in new language to be learned.
    pub first_lang: String,    // Word in fist language used to prompt for learning_lang.
    pub created: DateTime<Utc>, // Assuming `created` is set automatically to NOW(), you might not need this field during insertion.
    pub alternatives: Option<String>, // Alternate answers in the learning language, also use to match answers.
    pub skill: Option<String>, // Alternate answers in the learning language, also use to match answers.
    pub infinitive: Option<String>, // If the word is a verb, the infinitive form, otherwise none.
    pub pos: Option<String>, // Part of Speech, used to decide how to handle imports of first language.
    pub hint: Option<String>, // Hint to be displayed to help user translate words.
}

impl Default for Vocab {
    fn default() -> Self {
        Self {
            id: Default::default(),
            learning_lang: Default::default(), // Defaults to an empty String
            first_lang: Default::default(),    // Defaults to an empty String
            created: Utc::now(),               // Set a default creation time
            alternatives: Default::default(),  // Defaults to ""
            skill: Default::default(),         // Defaults to ""
            infinitive: Default::default(),    // Defaults to ""
            pos: Default::default(),           // Defaults to ""
            hint: Default::default(),          // Defaults to ""
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = vocab)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewVocab {
    pub learning_lang: String,           // Word in new language to be learned.
    pub first_lang: String,              // Word in fist language used to prompt for learning_lang.
    pub created: DateTime<Utc>, // Assuming `created` is set automatically to NOW(), you might not need this field during insertion.
    pub alternatives: Option<String>, // Alternate answers in the learning language, also use to match answers.
    pub skill: Option<String>, // Alternate answers in the learning language, also use to match answers.
    pub infinitive: Option<String>, // If the word is a verb, the infinitive form, otherwise none.
    pub pos: Option<String>, // Part of Speech, used to decide how to handle imports of first language.
    pub hint: Option<String>, // Hint to be displayed to help user translate words.
}

impl Default for NewVocab {
    fn default() -> Self {
        Self {
            learning_lang: Default::default(), // Defaults to an empty String
            first_lang: Default::default(),    // Defaults to an empty String
            created: Utc::now(),               // Set a default creation time
            alternatives: Default::default(),  // Defaults to ""
            skill: Default::default(),         // Defaults to ""
            infinitive: Default::default(),    // Defaults to ""
            pos: Default::default(),           // Defaults to ""
            hint: Default::default(),          // Defaults to ""
        }
    }
}

#[derive(Queryable, QueryableByName, Selectable, Identifiable, AsChangeset, Clone)]
#[diesel(table_name = vocab_study)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VocabStudy {
    pub id: i32,               // Primary key used to look up pair in the data layer.
    pub vocab_id: i32,         // Foreign key relates to the vocab table in the data layer.
    pub awesome_person_id: i32, // Foreign key relates to the awesome_person table in the data layer.
    pub guesses: Option<i32>,   // The number of times this pair was attempted.
    pub percentage_correct: Option<f64>, // The percentage of correct guesses calculated using the distance from the correct match.
    pub last_change: Option<f64>, // The most recent percentage correct change
    pub created: DateTime<Utc>, // Assuming `created` is set automatically to NOW(), you might not need this field during insertion.
    pub last_tested: Option<DateTime<Utc>>, // The last time this pair was guessed or tested.
    pub well_known: bool,      // If the pair is known, it is skipped to focus on new pairs
    pub user_notes: Option<String>, // Hint supplied by user to be displayed to help user translate words.
}

impl Default for VocabStudy {
    fn default() -> Self {
        Self {
            id: Default::default(),
            vocab_id: Default::default(),
            awesome_person_id: Default::default(),
            created: Utc::now(),               // Set a default creation time
            percentage_correct: Default::default(),
            last_change: None,
            last_tested: Default::default(),
            well_known: Default::default(),
            user_notes: None,
            guesses: None,
        }
    }
}

#[derive(Insertable, Default)]
#[diesel(table_name = vocab_study)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewVocabStudy {
    pub vocab_id: i32,         // Foreign key relates to the vocab table in the data layer.
    pub awesome_person_id: i32, // Foreign key relates to the awesome_person table in the data layer.
    pub guesses: Option<i32>,   // The number of times this pair was attempted.
    pub percentage_correct: Option<f64>, // The percentage of correct guesses calculated using the distance from the correct match.
    pub last_change: Option<f64>, // The most recent percentage correct change
    pub created: DateTime<Utc>, // Assuming `created` is set automatically to NOW(), you might not need this field during insertion.
    pub last_tested: Option<DateTime<Utc>>, // The last time this pair was guessed or tested.
    pub well_known: bool,      // If the pair is known, it is skipped to focus on new pairs
    pub user_notes: Option<String>, // Hint supplied by user to be displayed to help user translate words.
}

#[derive(Queryable, Identifiable, AsChangeset, Default)]
#[diesel(table_name = awesome_person)]
pub struct AwesomePerson {
    pub id: i32,                       // Primary key used to look up stats in the data layer.
    pub num_known: Option<i32>,        // Number of pairs moved to fully known state.
    pub num_correct: Option<i32>,      // Total number of correct guesses.
    pub num_incorrect: Option<i32>,    // Total number of incorrect guesses.
    pub total_percentage: Option<f64>, // Percentage guess correctly.
    pub updated: DateTime<Utc>,        // Last time stats where updated, (after each guess).
    pub name: Option<String>,          // User's name.
    pub code: Option<String>,          // Code used to identify a user, while in alpha mode.
    pub smallest_vocab: i32,           // Size of the smallest vocab word to be tested.
}

#[derive(Insertable)]
#[diesel(table_name = awesome_person)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewAwesomePerson {
    pub num_known: Option<i32>,        // Number of pairs moved to fully known state.
    pub num_correct: Option<i32>,      // Total number of correct guesses.
    pub num_incorrect: Option<i32>,    // Total number of incorrect guesses.
    pub total_percentage: Option<f64>, // Percentage guess correctly.
    pub updated: DateTime<Utc>,        // Last time stats where updated, (after each guess).
    pub name: Option<String>,          // User's name.
    pub code: Option<String>,          // Code used to identify a user, while in alpha mode.
    pub smallest_vocab: i32,           // Size of the smallest vocab word to be tested.
}

impl Default for NewAwesomePerson {
    fn default() -> Self {
        Self {
            num_known: Some(0),
            num_correct: Some(0),
            num_incorrect: Some(0),
            total_percentage: Some(0.0),
            updated: chrono::Utc::now(),
            name: Some("".to_string()),
            code: None,
            smallest_vocab: 1,
        }
    }
}



pub struct StudySet {
    pub vocab: Vocab,
    pub vocab_study: VocabStudy
}