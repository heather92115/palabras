use crate::schema::palabras::awesome_person;
use crate::schema::palabras::vocab;
use crate::schema::palabras::vocab_study;
use chrono::prelude::*;
use diesel::prelude::*;

/// A struct representing a vocabulary item in a language learning application.
///
/// This struct is used to store information about words or phrases that users are learning,
/// including translations, alternatives, and metadata to assist in the learning process.
///
/// # Fields
/// - `id`: Primary key used to uniquely identify a vocabulary item in the data layer.
/// - `learning_lang`: The word or phrase in the language being learned.
/// - `first_lang`: The translation of the word or phrase into the user's first language, used as a prompt.
/// - `created`: Timestamp when the vocabulary item was created. It is typically set automatically to the current time.
/// - `alternatives`: Optional. Additional correct answers or variations in the learning language.
/// - `skill`: Optional. The skill or category associated with the vocabulary item, used for organizing content.
/// - `infinitive`: Optional. For verbs, the infinitive form of the word. `None` for non-verb vocabulary items.
/// - `pos`: Optional. The part of speech of the vocabulary item, aiding in the application of grammatical rules.
/// - `hint`: Optional. A hint provided to assist users in translating the word or phrase.
/// - `num_learning_words`: The number of words contained in the `learning_lang` field, calculated for analytical purposes.
/// - `known_lang_code`: Language code for this known language.
/// - `learning_lang_code`: Language code for this learning language.
///
/// # Usage
/// This struct is primarily used with Diesel ORM for querying and manipulating vocabulary data in a PostgreSQL database.
/// It is annotated with Diesel-specific attributes to map it to the `vocab` table and ensure compatibility with the PostgreSQL backend.
#[derive(Queryable, QueryableByName, Selectable, Identifiable, AsChangeset, Clone)]
#[diesel(table_name = vocab)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Vocab {
    pub id: i32,
    pub learning_lang: String,
    pub first_lang: String,
    pub created: DateTime<Utc>,
    pub alternatives: Option<String>,
    pub skill: Option<String>,
    pub infinitive: Option<String>,
    pub pos: Option<String>,
    pub hint: Option<String>,
    pub num_learning_words: i32,
    pub known_lang_code: String,
    pub learning_lang_code: String,
}

impl Default for Vocab {
    fn default() -> Self {
        Self {
            id: Default::default(),
            learning_lang: Default::default(),
            first_lang: Default::default(),
            created: Utc::now(),
            alternatives: Default::default(),
            skill: Default::default(),
            infinitive: Default::default(),
            pos: Default::default(),
            hint: Default::default(),
            num_learning_words: 1,
            known_lang_code: Default::default(),
            learning_lang_code: Default::default(),
        }
    }
}

/// A struct for inserting new vocabulary records into a language learning application's database.
///
/// This struct defines the data structure used when adding new words or phrases to be learned.
/// It encapsulates all necessary information about a vocabulary item, excluding the unique identifier,
/// which is generated by the database upon insertion.
///
/// # See [`Models::Vocab`] for details
///
/// # Usage
/// This struct is used with Diesel's ORM capabilities for inserting data into the `vocab` table in a PostgreSQL database.
/// It is annotated with Diesel-specific macros to ensure it correctly maps to the table schema and supports PostgreSQL operations.
///
/// This struct streamlines the process of adding new vocabulary items by organizing all relevant information into a single data structure,
/// making it easy to maintain and extend the vocabulary database.
#[derive(Insertable)]
#[diesel(table_name = vocab)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewVocab {
    pub learning_lang: String,
    pub first_lang: String,
    pub created: DateTime<Utc>,
    pub alternatives: Option<String>,
    pub skill: Option<String>,
    pub infinitive: Option<String>,
    pub pos: Option<String>,
    pub hint: Option<String>,
    pub num_learning_words: i32,
    pub known_lang_code: String,
    pub learning_lang_code: String,
}

impl Default for NewVocab {
    fn default() -> Self {
        Self {
            learning_lang: Default::default(),
            first_lang: Default::default(),
            created: Utc::now(),
            alternatives: Default::default(),
            skill: Default::default(),
            infinitive: Default::default(),
            pos: Default::default(),
            hint: Default::default(),
            num_learning_words: 1,
            known_lang_code: Default::default(),
            learning_lang_code: Default::default(),
        }
    }
}

/// Represents a record of study progress for a specific vocabulary item by an awesome person (user).
///
/// This struct is used to query and manipulate data in the `vocab_study` table and provides a comprehensive
/// overview of a user's interaction with a particular vocabulary word or phrase. It tracks both the effort
/// (through attempts and correct attempts) and outcomes (through percentage correct and well-known status)
/// of studying, allowing for detailed monitoring and adjustment of the learning process.
///
/// # Fields
/// - `id`: The primary key for the record, unique to each study instance.
/// - `vocab_id`: A foreign key linking to the `vocab` table, identifying the vocabulary word being studied.
/// - `awesome_person_id`: A foreign key linking to the `awesome_person` table, identifying the user studying the vocabulary.
/// - `attempts`: The total number of attempts made by the user to guess or recall the vocabulary word correctly.
/// - `correct_attempts`: The number of times the vocabulary word was guessed or recalled correctly by the user.
/// - `percentage_correct`: The percentage of attempts that were correct, providing a measure of the user's mastery over time.
/// - `last_change`: The change in percentage correct since the last recorded attempt, indicating progress or regression.
/// - `created`: The timestamp when the study record was created, generally set to the current time upon creation.
/// - `last_tested`: The timestamp of the last attempt to study this vocabulary word, used to schedule future reviews.
/// - `well_known`: A boolean flag indicating whether the user has mastered this vocabulary word to the extent that it can be considered "well known" and potentially deprioritized in future study sessions.
/// - `user_notes`: Optional notes added by the user to aid in recall or provide additional context for the vocabulary word.
///
/// # Usage
/// The `VocabStudy` struct is integral to the operation of a language learning application, as it captures and reflects
/// the user's progress and engagement with the study material. By tracking detailed metrics of study sessions, it supports
/// personalized learning experiences and helps identify areas needing additional focus or reinforcement.
#[derive(Queryable, QueryableByName, Selectable, Identifiable, AsChangeset, Clone)]
#[diesel(table_name = vocab_study)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VocabStudy {
    pub id: i32,
    pub vocab_id: i32,
    pub awesome_person_id: i32,
    pub attempts: Option<i32>,
    pub percentage_correct: Option<f64>,
    pub last_change: Option<f64>,
    pub created: DateTime<Utc>,
    pub last_tested: Option<DateTime<Utc>>,
    pub well_known: bool,
    pub user_notes: Option<String>,
    pub correct_attempts: Option<i32>,
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
            attempts: None,
            correct_attempts: None,
        }
    }
}

/// A struct for inserting new vocab study records into a language learning application's database.
///
/// # See [`Models::VocabStudy`] for details
#[derive(Insertable, Default)]
#[diesel(table_name = vocab_study)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewVocabStudy {
    pub vocab_id: i32,
    pub awesome_person_id: i32,
    pub attempts: Option<i32>,
    pub percentage_correct: Option<f64>,
    pub last_change: Option<f64>,
    pub created: DateTime<Utc>,
    pub last_tested: Option<DateTime<Utc>>,
    pub well_known: bool,
    pub user_notes: Option<String>,
    pub correct_attempts: Option<i32>,
}


/// Represents an awesome person (user) in the language learning application, tracking their progress and personal details.
///
/// This struct is designed to manage and query data from the `awesome_person` table, encapsulating both the learning
/// performance metrics and basic user information. It is a central piece in understanding how a user interacts with
/// and benefits from the language learning platform.
///
/// # Fields
/// - `id`: The unique identifier for the user in the database, serving as the primary key.
/// - `num_known`: The total number of vocabulary pairs that the user has learned to the point of being considered "fully known".
/// - `num_correct`: The cumulative number of correct answers provided by the user during vocabulary tests.
/// - `num_incorrect`: The cumulative number of incorrect answers provided by the user, offering insights into areas of difficulty.
/// - `total_percentage`: An overall success rate calculated as the percentage of correct answers out of all attempts, reflecting the user's proficiency.
/// - `updated`: The timestamp of the last update to the user's statistics, indicating the most recent interaction with the study material.
/// - `name`: The user's name, allowing for a personalized experience within the application.
/// - `sec_code`: A unique code assigned to the user, particularly useful during the alpha testing phase for easy identification without requiring authentication.
/// - `smallest_vocab`: Specifies the smallest size of vocabulary word that the user is comfortable with, assisting in customizing the difficulty level of the tests.
/// - `max_learning_words`: The maximum number of new words (learning words) the user is comfortable being tested on in a single session, helping tailor the learning experience to the user's capacity.
///
/// # Usage
/// The `AwesomePerson` struct plays a crucial role in the personalized adaptation of the language learning application to the user's
/// needs and preferences. By keeping a detailed record of learning metrics, it enables the system to adjust the complexity and focus of
/// study sessions, ensuring a more efficient and enjoyable learning journey.
#[derive(Queryable, QueryableByName, Selectable, Identifiable, AsChangeset, Clone)]
#[diesel(table_name = awesome_person)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AwesomePerson {
    pub id: i32,
    pub num_known: Option<i32>,
    pub num_correct: Option<i32>,
    pub num_incorrect: Option<i32>,
    pub total_percentage: Option<f64>,
    pub updated: DateTime<Utc>,
    pub name: Option<String>,
    pub sec_code: String,
    pub smallest_vocab: i32,
    pub max_learning_words: i32,
}

impl Default for AwesomePerson {
    fn default() -> Self {
        Self {
            id: Default::default(),
            num_known: Some(0),
            num_correct: Some(0),
            num_incorrect: Some(0),
            total_percentage: Some(0.0),
            updated: chrono::Utc::now(),
            name: Some("".to_string()),
            sec_code: "".to_string(),
            smallest_vocab: 1,
            max_learning_words: 5,
        }
    }
}

/// A struct for inserting new Awesome Person records into a language learning application's database.
///
/// # See [`Models::AwesomePerson`] for details
#[derive(Insertable)]
#[diesel(table_name = awesome_person)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewAwesomePerson {
    pub num_known: Option<i32>,
    pub num_correct: Option<i32>,
    pub num_incorrect: Option<i32>,
    pub total_percentage: Option<f64>,
    pub updated: DateTime<Utc>,
    pub name: Option<String>,
    pub sec_code: String,
    pub smallest_vocab: i32,
    pub max_learning_words: i32,
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
            sec_code: "".to_string(),
            smallest_vocab: 1,
            max_learning_words: 5,
        }
    }
}

/// Represents the relationship between a vocabulary item (`Vocab`) and its study metrics (`VocabStudy`) for an individual user
/// (`AwesomePerson`) in the language learning application.
///
/// This struct forms a bridge between a vocabulary item and its study progress for a specific user, ensuring that each user's
/// interaction with the vocabulary item is tracked uniquely. The one-to-one relationship between `Vocab` and `VocabStudy`
/// within the context of an individual user's study session signifies that each `StudySet` is personalized, containing study
/// metrics tailored to the user's performance and progress with that particular vocabulary item.
///
/// # Fields
/// - `vocab`: The `Vocab` struct, representing the target language word or phrase, including its translation and linguistic details like part of speech or conjugation.
/// - `vocab_study`: The `VocabStudy` struct, recording the individual user's study metrics for the `vocab` item, such as the number of attempts, correctness percentage, last studied time, and personalized notes or hints.
///
/// # Usage
/// The `StudySet` struct plays a crucial role in providing a personalized and adaptive learning experience in the language learning
/// application. By associating each vocabulary item with specific study metrics for each user, it allows for the tracking of progress
/// and proficiency improvements over time. This detailed association helps in tailoring the learning and review sessions to fit the
/// user's needs, enabling a more efficient and targeted approach to language learning.
///
/// Each `StudySet` is unique to an `AwesomePerson` (user), indicating that users will have their own distinct `VocabStudy` records
/// for the same `Vocab` item. This structure supports the creation of customized learning paths and facilitates the adaptation of
/// the learning experience to the user's performance and progress.
pub struct StudySet {
    pub vocab: Vocab,
    pub vocab_study: VocabStudy
}