use crate::dal::db_connection::get_connection;
use crate::models::ProgressStats;
use crate::schema::palabras::progress_stats::dsl::progress_stats;
use diesel::result::Error as DieselError;
use diesel::{QueryDsl, RunQueryDsl};

/// Trait for interacting with progress stats records in a database.
///
/// TODO: Add the ability to track more than just one user.
///
/// This trait abstracts the operations related to fetching and updating progress stats records, allowing for
/// different implementations including ones suitable for testing with mock data.
pub trait ProgressStatsRepository {
    /// Retrieves a single progress stats record by its primary key.
    ///
    /// # Parameters
    ///
    /// * `id` - The primary key (`id`) of the progress stats record to retrieve.
    ///
    /// # Returns
    ///
    /// Returns `Ok(ProgressStats)` if a progress stats record with the specified `id` exists,
    /// or an error if the query fails or if no record matches the given `id`.
    fn get_progress_stats_by_id(&self, id: i32) -> Result<ProgressStats, DieselError>;

    /// Updates an existing `ProgressStats` record in the database based on the provided `ProgressStats` instance.
    ///
    /// # Parameters
    ///
    /// * `stats` - A `ProgressStats` struct representing the updated state of a progress stats record.
    ///
    /// # Returns
    ///
    /// Returns the number of records updated in the database, or an error if the update operation fails.
    fn update_progress_stats(&self, stats: ProgressStats) -> Result<usize, String>;
}

pub struct DbProgressStatsRepository;

impl ProgressStatsRepository for DbProgressStatsRepository {
    /// Implementation, see trait for details [`ProgressStatsRepository::get_progress_stats_by_id`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn get_progress_stats_by_id(&self, stats_id: i32) -> Result<ProgressStats, DieselError> {
        let mut conn = get_connection();
        progress_stats.find(stats_id).first(&mut conn)
    }

    /// Implementation, see trait for details [`ProgressStatsRepository::update_progress_stats`]
    ///
    /// For advanced usage and mock implementations, please refer to
    /// the integration tests for this module.
    fn update_progress_stats(&self, updating: ProgressStats) -> Result<usize, String> {
        let mut conn = get_connection();

        let updated = diesel::update(progress_stats.find(updating.id))
            .set(&updating)
            .execute(&mut conn)
            .map_err(|err| err.to_string())?;

        Ok(updated)
    }
}
