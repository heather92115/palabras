use crate::dal::db_connection::get_connection;
use crate::models::{NewTranslationPair, TranslationPair};
use crate::schema::palabras::translation_pair::dsl::translation_pair;
use crate::schema::palabras::translation_pair::dsl::*;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::{sql_query, RunQueryDsl};

/// The data mapping layer. Diesel is used to query and update translation pairs.
/// Connections are pulled from a static singleton pool for each operation.

/// Trait for accessing translation pair records in a database.
///
/// This trait abstracts the operations related to fetching and updating translation pair records, allowing for
/// different implementations including ones suitable for testing with mock data.
pub trait TranslationPairRepository {
    ///
    /// Gets a single translation pair using its primary key.
    ///
    /// # Parameters
    ///
    /// * `pair_id` - Primary key used to look up a translation pair.
    ///
    /// # Returns
    ///
    /// Returns `Ok(TranslationPair)` if a translation pair with the specified `pair_id` exists,
    /// or a `DieselError` if the query fails (e.g., due to connection issues or if no
    /// pair matches the given `pair_id`).
    fn get_translation_pair_by_id(&self, pair_id: i32) -> Result<TranslationPair, DieselError>;

    /// Looks up a single translation pair by the learning language.
    ///
    /// This function is designed to support scenarios where translation pairs need to be retrieved
    /// based on the `learning_lang` field, which is expected to be unique within the dataset.
    /// It is particularly useful for processing Duolingo JSON exports in binary programs.
    ///
    /// # Parameters
    ///
    /// * `learning_lang_search` - The learning language string used to search for the corresponding translation pair.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(TranslationPair))` if a translation pair matching the `learning_lang_search` exists,
    /// `Ok(None)` if no matching pair is found, or an `Err(diesel::result::Error)` if there's an issue with the database query.
    fn find_translation_pair_by_learning_language(
        &self,
        learning_lang_search: String,
    ) -> Result<Option<TranslationPair>, DieselError>;

    /// Looks up a single translation pair by the searching alternatives.
    ///
    /// This function is designed to support scenarios where translation pairs need to be retrieved
    /// based on the `alternatives` field, which is expected to be unique within the dataset.
    /// It is particularly useful for processing Duolingo JSON exports in binary programs.
    ///
    /// # Parameters
    ///
    /// * `alternative_search` - The string used to search for the corresponding translation pair.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(TranslationPair))` if a translation pair matching the `alternative_search` exists,
    /// `Ok(None)` if no matching pair is found, or an `Err(diesel::result::Error)` if there's an issue with the database query.
    fn find_translation_pair_by_alternative(
        &self,
        alternative_search: String,
    ) -> Result<Option<TranslationPair>, DieselError>;

    /// Retrieves a list of `TranslationPair` records where the `first_lang` fields are empty.
    ///
    /// This function queries the database for translation pairs that lack a primary language definition,
    /// indicating they may require further processing or completion. It is useful for identifying
    /// incomplete entries within the dataset.
    ///
    /// # Parameters
    ///
    /// * `limit` - Specifies the maximum number of translation pairs to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(Vec<TranslationPair>)`: A vector of `TranslationPair` instances with empty `first_lang` fields,
    ///   which could be empty if no such records exist.
    /// - `Err(String)`: An error message string if the database query fails.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue executing the query, including connection problems
    /// or syntax errors in the query itself. The error is returned as a `String` describing the failure.
    fn get_empty_first_lang_pairs(&self, limit: i64) -> Result<Vec<TranslationPair>, String>;

    /// Retrieves a list of `TranslationPair` records to be studied, excluding those marked as fully known.
    ///
    /// This function performs a database query to select translation pairs based on their `percentage_correct` field,
    /// prioritizing pairs that need further study. Pairs already marked as fully known are excluded from the results.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(Vec<TranslationPair>)`: A vector of `TranslationPair` instances up to the specified limit,
    ///   ordered by ascending `percentage_correct` value.
    /// - `Err(String)`: An error message string if the database query fails.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue executing the query, including connection problems
    /// or syntax errors in the query itself. The error is returned as a `String` describing the failure.
    fn get_study_pairs(&self) -> Result<Vec<TranslationPair>, String>;

    /// Inserts a new `TranslationPair` record into the database.
    ///
    /// This function adds a new translation pair based on the provided `NewTranslationPair` data,
    /// which does not include an `id` field. After insertion, the database automatically assigns an `id`,
    /// and the function returns the newly created `TranslationPair` including its `id`.
    ///
    /// # Parameters
    ///
    /// * `new_translation_pair` - A reference to a `NewTranslationPair` struct containing the data
    ///   for the new translation pair to be created.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(TranslationPair)`: The newly created `TranslationPair`, including its database-assigned `id`.
    /// - `Err(String)`: An error message string if the insert operation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue performing the insert operation, including connection problems
    /// or violations of database constraints (e.g., unique constraints). The error is returned as a `String`
    /// describing the failure.
    fn create_translation_pair(
        &self,
        new_translation_pair: &NewTranslationPair,
    ) -> Result<TranslationPair, String>;

    /// Updates an existing `TranslationPair` record in the database.
    ///
    /// This function takes a fully specified `TranslationPair` instance, including its `id`, and updates
    /// the corresponding database record to match the provided values. The function returns the number
    /// of records updated, which should be 1 in successful cases.
    ///
    /// # Parameters
    ///
    /// * `updating` - A `TranslationPair` struct representing the updated state of a translation pair,
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
    fn update_translation_pair(&self, updating: TranslationPair) -> Result<usize, String>;
}

pub struct DbTranslationPairRepository;

/// Implementation of TranslationPairRepository
///
/// For behavior, see the documentation of [`TranslationPairRepository`].
impl TranslationPairRepository for DbTranslationPairRepository {
    /// Implementation, see trait for details [`TranslationPairRepository::get_translation_pair_by_id`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn get_translation_pair_by_id(&self, pair_id: i32) -> Result<TranslationPair, DieselError> {
        let mut conn = get_connection();
        translation_pair.find(pair_id).first(&mut conn)
    }

    /// Implementation, see trait for details [`TranslationPairRepository::find_translation_pair_by_learning_language`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn find_translation_pair_by_learning_language(
        &self,
        learning_lang_search: String,
    ) -> Result<Option<TranslationPair>, DieselError> {
        let mut conn = get_connection();
        translation_pair
            .filter(learning_lang.eq(learning_lang_search))
            .first(&mut conn)
            .optional()
    }

    /// Implementation, see trait for details [`TranslationPairRepository::find_translation_pair_by_alternative`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn find_translation_pair_by_alternative(
        &self,
        alternative_search: String,
    ) -> Result<Option<TranslationPair>, DieselError> {
        let mut conn = get_connection();

        let like_pattern = format!("%{}%", alternative_search);
        translation_pair
            .filter(alternatives.ilike(like_pattern))
            .first(&mut conn)
            .optional()
    }

    /// Implementation, see trait for details [`TranslationPairRepository::get_empty_first_lang_pairs`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn get_empty_first_lang_pairs(&self, limit: i64) -> Result<Vec<TranslationPair>, String> {
        let mut conn = get_connection();
        let pairs = translation_pair
            .filter(first_lang.eq(""))
            .limit(limit)
            .order_by(percentage_correct)
            .get_results(&mut conn)
            .map_err(|err| err.to_string())?;

        Ok(pairs)
    }

    /// Implementation, see trait for details [`TranslationPairRepository::get_study_pairs`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn get_study_pairs(&self) -> Result<Vec<TranslationPair>, String> {
        let sql_text =
            "select * from translation_pair where not fully_known and not too_easy and length(first_lang) > 0 order by percentage_correct desc".to_string();

        let mut conn = get_connection();
        let pairs = sql_query(sql_text)
            .load::<TranslationPair>(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(pairs)
    }

    /// Implementation, see trait for details [`TranslationPairRepository::create_translation_pair`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn create_translation_pair(
        &self,
        new_translation_pair: &NewTranslationPair,
    ) -> Result<TranslationPair, String> {
        let mut conn = get_connection();
        let inserted = diesel::insert_into(translation_pair)
            .values(new_translation_pair)
            .get_result(&mut conn)
            .map_err(|err| err.to_string())?;

        Ok(inserted)
    }

    /// Implementation, see trait for details [`TranslationPairRepository::update_translation_pair`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn update_translation_pair(&self, updating: TranslationPair) -> Result<usize, String> {
        let mut conn = get_connection();

        let updated = diesel::update(translation_pair.find(updating.id))
            .set(&updating)
            .execute(&mut conn)
            .map_err(|err| err.to_string())?;

        Ok(updated)
    }
}
