use crate::dal::db_connection::get_connection;
use crate::models::AwesomePerson;
use crate::schema::palabras::awesome_person::dsl::awesome_person;
use diesel::result::Error as DieselError;
use diesel::{QueryDsl, RunQueryDsl};

/// Trait for interacting with awesome person records in a database.
///
/// This trait abstracts the operations related to fetching and updating records, allowing for
/// different implementations including ones suitable for testing with mock data.
pub trait AwesomePersonRepository: Send + Sync {
    /// Retrieves a single awesome person record by its primary key.
    ///
    /// # Parameters
    ///
    /// * `id` - The primary key (`id`) of the progress stats record to retrieve.
    ///
    /// # Returns
    ///
    /// Returns `Ok(AwesomePerson)` if a progress stats record with the specified `id` exists,
    /// or an error if the query fails or if no record matches the given `id`.
    fn get_awesome_person_by_id(&self, id: i32) -> Result<AwesomePerson, DieselError>;

    /// Updates an existing `AwesomePerson` record in the database based on the provided `AwesomePerson` instance.
    ///
    /// # Parameters
    ///
    /// * `stats` - A `AwesomePerson` struct representing the updated state of a progress stats record.
    ///
    /// # Returns
    ///
    /// Returns the number of records updated in the database, or an error if the update operation fails.
    fn update_awesome_person(&self, stats: AwesomePerson) -> Result<usize, String>;
}

pub struct DbAwesomePersonRepository;

impl AwesomePersonRepository for DbAwesomePersonRepository {
    /// Implementation, see trait for details [`AwesomePersonRepository::get_awesome_person_by_id`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn get_awesome_person_by_id(&self, stats_id: i32) -> Result<AwesomePerson, DieselError> {
        let mut conn = get_connection();
        awesome_person.find(stats_id).first(&mut conn)
    }

    /// Implementation, see trait for details [`AwesomePersonRepository::update_awesome_person`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn update_awesome_person(&self, updating: AwesomePerson) -> Result<usize, String> {
        let mut conn = get_connection();

        let updated = diesel::update(awesome_person.find(updating.id))
            .set(&updating)
            .execute(&mut conn)
            .map_err(|err| err.to_string())?;

        Ok(updated)
    }
}
