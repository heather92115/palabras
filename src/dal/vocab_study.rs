use crate::dal::db_connection::get_connection;
use crate::models::{NewVocabStudy, Vocab, VocabStudy};
use crate::schema::palabras::vocab_study::dsl::vocab_study;
use crate::schema::palabras::vocab_study::dsl::*;
use crate::schema::palabras::vocab::dsl::vocab;

use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::{RunQueryDsl};

/// The data mapping layer. Diesel is used to query and update vocab study.
/// Connections are pulled from a static singleton pool for each operation.

/// Trait for accessing vocab study records in a database.
///
/// This trait abstracts the operations related to fetching and updating vocab study records, allowing for
/// different implementations including ones suitable for testing with mock data.
pub trait VocabStudyRepository: Send + Sync {
    ///
    /// Gets a single vocab study using its primary key.
    ///
    /// # Parameters
    ///
    /// * `vocab_study_id` - Primary key used to look up a vocab study.
    ///
    /// # Returns
    ///
    /// Returns `Ok(VocabStudy)` if a vocab study with the specified `vocab_study_id` exists,
    /// or a `DieselError` if the query fails (e.g., due to connection issues or if no
    /// record matches the given `vocab_study_id`).
    fn get_vocab_study_by_id(&self, vocab_study_id: i32) -> Result<VocabStudy, DieselError>;

    ///
    /// Gets a single vocab study using its two foreign references
    ///
    /// # Parameters
    ///
    /// * `v_id` - Primary key for vocab.
    /// * `ap_id` - Primary key for awesome person.
    ///
    /// # Returns
    ///
    /// Returns `Ok(VocabStudy)` if a vocab study with the specified ids exists,
    /// or a `DieselError` if the query fails (e.g., due to connection issues or if no
    /// record matches the given `vocab_study_id`).
    fn get_vocab_study_by_foreign_refs(&self, v_id: i32, ap_id:  i32) -> Result<Option<VocabStudy>, DieselError>;


    /// Retrieves a study set of vocabulary pairs for a specified awesome person.
    ///
    /// This function queries the database to find all vocabulary pairs associated with
    /// the given `awesome_person_id`. It performs an inner join between the `vocab_study`
    /// and `vocab` tables to gather detailed information about each vocabulary item in the
    /// study set.
    ///
    /// # Parameters
    ///
    /// - `ap_id`: The identifier of the awesome person for whom the study set is being retrieved.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(Vec<(VocabStudy, Vocab)>)`: A vector of tuples, each containing a `VocabStudy`
    ///   record and its corresponding `Vocab` record, representing the study set for the
    ///   specified awesome person.
    /// - `Err(String)`: An error message string if the database query fails. This could be
    ///   due to connection issues, or if the query itself encounters an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - There is a problem connecting to the database.
    /// - The SQL query fails to execute properly.
    fn get_study_set(&self, ap_id: i32) -> Result<Vec<(VocabStudy, Vocab)>, String>;

    /// Inserts a new `VocabStudy` record into the database.
    ///
    /// This function adds a new vocab study based on the provided `NewVocabStudy` data,
    /// which does not include an `id` field. After insertion, the database automatically assigns an `id`,
    /// and the function returns the newly created `VocabStudy` including its `id`.
    ///
    /// # Parameters
    ///
    /// * `new_vocab_study` - A reference to a `NewVocabStudy` struct containing the data
    ///   for the new vocab study to be created.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `Ok(VocabStudy)`: The newly created `VocabStudy`, including its database-assigned `id`.
    /// - `Err(String)`: An error message string if the insert operation fails.
    ///
    /// # Errors
    ///
    /// Returns an error if there's an issue performing the insert operation, including connection problems
    /// or violations of database constraints (e.g., unique constraints, foreign key constraints).
    /// The error is returned as a `String`
    /// describing the failure.
    fn create_vocab_study(
        &self,
        new_vocab_study: &NewVocabStudy,
    ) -> Result<VocabStudy, String>;

    /// Updates an existing `VocabStudy` record in the database.
    ///
    /// This function takes a fully specified `VocabStudy` instance, including its `id`, and updates
    /// the corresponding database record to match the provided values. The function returns the number
    /// of records updated, which should be 1 in successful cases.
    ///
    /// # Parameters
    ///
    /// * `updating` - A `VocabStudy` struct representing the updated state of a vocab study,
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
    fn update_vocab_study(&self, updating: VocabStudy) -> Result<usize, String>;
}

pub struct DbVocabStudyRepository;

/// Implementation of VocabStudyRepository
///
/// For behavior, see the documentation of [`VocabStudyRepository`].
impl VocabStudyRepository for DbVocabStudyRepository {
    /// Implementation, see trait for details [`VocabStudyRepository::get_vocab_study_by_id`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn get_vocab_study_by_id(&self, vocab_study_id: i32) -> Result<VocabStudy, DieselError> {
        let mut conn = get_connection();
        vocab_study.find(vocab_study_id).first(&mut conn)
    }

    /// Implementation, see trait for details [`VocabStudyRepository::get_vocab_study_by_foreign_refs`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn get_vocab_study_by_foreign_refs(&self, v_id: i32, ap_id:  i32) -> Result<Option<VocabStudy>, DieselError> {
        let mut conn = get_connection();

        vocab_study
            .filter(vocab_id.eq(v_id).and(awesome_person_id.eq(ap_id)))
            .first(&mut conn)
            .optional()
    }

    /// Implementation, see trait for details [`VocabStudyRepository::get_study_set`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn get_study_set(&self, ap_id: i32) -> Result<Vec<(VocabStudy, Vocab)>, String> {
        let mut conn = get_connection();

        let results = vocab_study
            .inner_join(vocab)
            .filter(awesome_person_id.eq(ap_id))
            .load::<(VocabStudy, Vocab)>(&mut conn)
            .map_err(|err| err.to_string())?;

        Ok(results)
    }

    /// Implementation, see trait for details [`VocabStudyRepository::create_vocab_study`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn create_vocab_study(
        &self,
        new_vocab_study: &NewVocabStudy,
    ) -> Result<VocabStudy, String> {
        let mut conn = get_connection();
        let inserted = diesel::insert_into(vocab_study)
            .values(new_vocab_study)
            .get_result(&mut conn)
            .map_err(|err| err.to_string())?;

        Ok(inserted)
    }

    /// Implementation, see trait for details [`VocabStudyRepository::update_vocab_study`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn update_vocab_study(&self, updating: VocabStudy) -> Result<usize, String> {
        let mut conn = get_connection();

        let updated = diesel::update(vocab_study.find(updating.id))
            .set(&updating).execute(&mut conn).map_err(|e| e.to_string())?;

        Ok(updated)
    }
}
