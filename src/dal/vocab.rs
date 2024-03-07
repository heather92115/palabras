use crate::dal::db_connection::get_connection;
use crate::models::{NewVocab, Vocab};
use crate::schema::palabras::vocab::dsl::vocab;
use crate::schema::palabras::vocab::dsl::*;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::{RunQueryDsl};

/// The data mapping layer. Diesel is used to query and update vocabs.
/// Connections are pulled from a static singleton pool for each operation.

/// Trait for accessing vocab records in a database.
///
/// This trait abstracts the operations related to fetching and updating vocab records, allowing for
/// different implementations including ones suitable for testing with mock data.
pub trait VocabRepository {
    ///
    /// Gets a single vocab using its primary key.
    ///
    /// # Parameters
    ///
    /// * `vocab_id` - Primary key used to look up a vocab.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vocab)` if a vocab with the specified `vocab_id` exists,
    /// or a `DieselError` if the query fails (e.g., due to connection issues or if no
    /// vocab matches the given `vocab_id`).
    fn get_vocab_by_id(&self, vocab_id: i32) -> Result<Vocab, DieselError>;

    /// Looks up a single vocab by the learning language.
    ///
    /// This function is designed to support scenarios where vocabs need to be retrieved
    /// based on the `learning_lang` field, which is expected to be unique within the dataset.
    /// It is particularly useful for processing Duolingo JSON exports in binary programs.
    ///
    /// # Parameters
    ///
    /// * `learning_lang_search` - The learning language string used to search for the corresponding vocab.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(Vocab))` if a vocab matching the `learning_lang_search` exists,
    /// `Ok(None)` if no matching vocab is found, or an `Err(diesel::result::Error)` if there's an issue with the database query.
    fn find_vocab_by_learning_language(
        &self,
        learning_lang_search: String,
    ) -> Result<Option<Vocab>, DieselError>;

    /// Looks up a single vocab by the searching alternatives.
    ///
    /// This function is designed to support scenarios where vocabs need to be retrieved
    /// based on the `alternatives` field, which is expected to be unique within the dataset.
    /// It is particularly useful for processing Duolingo JSON exports in binary programs.
    ///
    /// # Parameters
    ///
    /// * `alternative_search` - The string used to search for the corresponding vocab.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(Vocab))` if a vocab matching the `alternative_search` exists,
    /// `Ok(None)` if no matching vocab is found, or an `Err(diesel::result::Error)` if there's an issue with the database query.
    fn find_vocab_by_alternative(
        &self,
        alternative_search: String,
    ) -> Result<Option<Vocab>, DieselError>;

    /// Retrieves a list of `Vocab` records where the `first_lang` fields are empty.
    ///
    /// This function queries the database for vocabs that lack a primary language definition,
    /// indicating they may require further processing or completion. It is useful for identifying
    /// incomplete entries within the dataset.
    ///
    /// # Parameters
    ///
    /// * `limit` - Specifies the maximum number of vocabs to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(Vec<Vocab>)`: A vector of `Vocab` instances with empty `first_lang` fields,
    ///   which could be empty if no such records exist.
    /// - `Err(String)`: An error message string if the database query fails.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue executing the query, including connection problems
    /// or syntax errors in the query itself. The error is returned as a `String` describing the failure.
    fn get_empty_first_lang(&self, limit: i64) -> Result<Vec<Vocab>, String>;

    /// Inserts a new `Vocab` record into the database.
    ///
    /// This function adds a new vocab based on the provided `NewVocab` data,
    /// which does not include an `id` field. After insertion, the database automatically assigns an `id`,
    /// and the function returns the newly created `Vocab` including its `id`.
    ///
    /// # Parameters
    ///
    /// * `new_vocab` - A reference to a `NewVocab` struct containing the data
    ///   for the new vocab to be created.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(Vocab)`: The newly created `Vocab`, including its database-assigned `id`.
    /// - `Err(String)`: An error message string if the insert operation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue performing the insert operation, including connection problems
    /// or violations of database constraints (e.g., unique constraints). The error is returned as a `String`
    /// describing the failure.
    fn create_vocab(
        &self,
        new_vocab: &NewVocab,
    ) -> Result<Vocab, String>;

    /// Updates an existing `Vocab` record in the database.
    ///
    /// This function takes a fully specified `Vocab` instance, including its `id`, and updates
    /// the corresponding database record to match the provided values. The function returns the number
    /// of records updated, which should be 1 in successful cases.
    ///
    /// # Parameters
    ///
    /// * `updating` - A `Vocab` struct representing the updated state of a vocab,
    ///   including its `id` for lookup.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(usize)`: The number of records updated in the database, expected to be 1 when successful.
    /// - `Err(String)`: An error message string if the update operation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue performing the update operation, including connection problems,
    /// attempting to update a record that does not exist, or violations of database constraints. The error
    /// is returned as a `String` describing the failure.
    fn update_vocab(&self, updating: Vocab) -> Result<usize, String>;
}

pub struct DbVocabRepository;

/// Implementation of VocabRepository
///
/// For behavior, see the documentation of [`VocabRepository`].
impl VocabRepository for DbVocabRepository {
    /// Implementation, see trait for details [`VocabRepository::get_vocab_by_id`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn get_vocab_by_id(&self, vocab_id: i32) -> Result<Vocab, DieselError> {
        let mut conn = get_connection();
        vocab.find(vocab_id).first(&mut conn)
    }

    /// Implementation, see trait for details [`VocabRepository::find_vocab_by_learning_language`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn find_vocab_by_learning_language(
        &self,
        learning_lang_search: String,
    ) -> Result<Option<Vocab>, DieselError> {
        let mut conn = get_connection();
        vocab
            .filter(learning_lang.eq(learning_lang_search))
            .first(&mut conn)
            .optional()
    }

    /// Implementation, see trait for details [`VocabRepository::find_vocab_by_alternative`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn find_vocab_by_alternative(
        &self,
        alternative_search: String,
    ) -> Result<Option<Vocab>, DieselError> {
        let mut conn = get_connection();

        let like_pattern = format!("%{}%", alternative_search);
        vocab
            .filter(alternatives.ilike(like_pattern))
            .first(&mut conn)
            .optional()
    }

    /// Implementation, see trait for details [`VocabRepository::get_empty_first_lang`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn get_empty_first_lang(&self, limit: i64) -> Result<Vec<Vocab>, String> {
        let mut conn = get_connection();
        let vocabs = vocab
            .filter(first_lang.eq(""))
            .limit(limit)
            .get_results(&mut conn)
            .map_err(|err| err.to_string())?;

        Ok(vocabs)
    }

    /// Implementation, see trait for details [`VocabRepository::create_vocab`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn create_vocab(
        &self,
        new_vocab: &NewVocab,
    ) -> Result<Vocab, String> {
        let mut conn = get_connection();
        let inserted = diesel::insert_into(vocab)
            .values(new_vocab)
            .get_result(&mut conn)
            .map_err(|err| err.to_string())?;

        Ok(inserted)
    }

    /// Implementation, see trait for details [`VocabRepository::update_vocab`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn update_vocab(&self, updating: Vocab) -> Result<usize, String> {
        let mut conn = get_connection();

        let updated = diesel::update(vocab.find(updating.id))
            .set(&updating)
            .execute(&mut conn)
            .map_err(|err| err.to_string())?;

        Ok(updated)
    }
}
