use dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::dal::progress_stats::{DbProgressStatsRepository, ProgressStatsRepository};
use palabras::models::ProgressStats;
use palabras::sl::learn_pairs::PROGRESS_STATS_ID;

#[test]
fn test_progress_stats() {
    dotenv::from_filename("test.env").ok();

    verify_connection_migrate_db();
    let repo = DbProgressStatsRepository;
    let current = repo
        .get_progress_stats_by_id(PROGRESS_STATS_ID)
        .expect("Should find progress stats");
    assert_eq!(
        current.id, PROGRESS_STATS_ID,
        "Progress Status ID should be 1"
    );

    let num_correct = current.num_correct.unwrap() + 1;
    let updating = ProgressStats {
        num_correct: Some(num_correct),
        ..current
    };

    let num_updated = repo
        .update_progress_stats(updating)
        .expect("Should update progress stats");
    assert_eq!(num_updated, 1, "Should update 1 progress stats record");

    let updated = repo
        .get_progress_stats_by_id(PROGRESS_STATS_ID)
        .expect("Should find updated progress stats");

    assert_eq!(
        updated.id, PROGRESS_STATS_ID,
        "Progress Status ID should be 1"
    );
    assert_eq!(
        updated.num_correct.unwrap(),
        num_correct,
        "Num correct should match, result {}, expected {}",
        updated.num_correct.unwrap(),
        num_correct
    );
}
