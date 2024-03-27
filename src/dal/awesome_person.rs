use crate::dal::db_connection::{error_to_string, get_connection};
use crate::models::{AwesomePerson, NewAwesomePerson};
use crate::schema::palabras::awesome_person::dsl::awesome_person;
use crate::schema::palabras::awesome_person::dsl::*;
use diesel::ExpressionMethods;
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};

/// Trait for interacting with awesome person records in a database.
///
/// This trait abstracts the operations related to fetching and updating records, allowing for
/// different implementations including ones suitable for testing with mock data.
pub trait AwesomePersonRepository: Send + Sync {
    /// Retrieves a single awesome person record by its primary key.
    ///
    /// # Parameters
    ///
    /// * `awesome_id` - The primary key (`id`) of the awesome person record to retrieve.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(AwesomePerson))` if an awesome person record with the specified `id` exists,
    /// Ok(None) if not found or an error if the query fails.
    fn get_awesome_person_by_id(&self, awesome_id: i32) -> Result<Option<AwesomePerson>, String>;

    /// Retrieves a single awesome person record by their lookup code.
    ///
    /// # Parameters
    ///
    /// * `look_up_code` - The (`look_up_code`) used find an awesome person record.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(AwesomePerson))` if an awesome person record with the specified `code` exists,
    /// Ok(None) if not found or an error if the query fails.
    fn get_awesome_person_by_code(
        &self,
        look_up_code: String,
    ) -> Result<Option<AwesomePerson>, String>;

    /// Updates an existing `AwesomePerson` record in the database based on the provided `AwesomePerson` instance.
    ///
    /// # Parameters
    ///
    /// * `stats` - A `AwesomePerson` struct representing the updated state of a record.
    ///
    /// # Returns
    ///
    /// Returns the number of records updated in the database, or an error if the update operation fails.
    fn update_awesome_person(&self, stats: AwesomePerson) -> Result<usize, String>;

    /// Creates a new `AwesomePerson` record in the database based on the provided `NewAwesomePerson` instance.
    ///
    /// # Parameters
    ///
    /// * `stats` - A `NewAwesomePerson` struct representing the record to create
    ///
    /// # Returns
    ///
    /// Returns `Ok(AwesomePerson)` if the awesome person record was created with a newly assigned `id`,
    /// or an error if create fails.
    fn create_awesome_person(
        &self,
        new_awesome_person: &NewAwesomePerson,
    ) -> Result<AwesomePerson, String>;
}

pub struct DbAwesomePersonRepository;

impl AwesomePersonRepository for DbAwesomePersonRepository {
    /// Implementation, see trait for details [`AwesomePersonRepository::get_awesome_person_by_id`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn get_awesome_person_by_id(&self, awesome_id: i32) -> Result<Option<AwesomePerson>, String> {
        let mut conn = get_connection()?;
        awesome_person
            .find(awesome_id)
            .first(&mut conn)
            .optional()
            .map_err(|err| error_to_string(err))
    }

    /// Implementation, see trait for details [`AwesomePersonRepository::get_awesome_person_by_code`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn get_awesome_person_by_code(
        &self,
        sec_code_search: String,
    ) -> Result<Option<AwesomePerson>, String> {
        let mut conn = get_connection()?;
        awesome_person
            .filter(sec_code.eq(sec_code_search))
            .first(&mut conn)
            .optional()
            .map_err(|err| error_to_string(err))
    }

    /// Implementation, see trait for details [`AwesomePersonRepository::update_awesome_person`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn update_awesome_person(&self, updating: AwesomePerson) -> Result<usize, String> {
        let mut conn = get_connection()?;

        let num_updated = diesel::update(awesome_person.find(updating.id))
            .set(&updating)
            .execute(&mut conn)
            .map_err(|err| err.to_string())?;

        Ok(num_updated)
    }

    /// Implementation, see trait for details [`AwesomePersonRepository::create_awesome_person`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn create_awesome_person(
        &self,
        new_awesome_person: &NewAwesomePerson,
    ) -> Result<AwesomePerson, String> {
        let mut conn = get_connection()?;
        let inserted = diesel::insert_into(awesome_person)
            .values(new_awesome_person)
            .get_result(&mut conn)
            .map_err(|err| err.to_string())?;

        Ok(inserted)
    }
}
