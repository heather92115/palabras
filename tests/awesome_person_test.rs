use dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::dal::awesome_person::{DbAwesomePersonRepository, AwesomePersonRepository};
use palabras::models::AwesomePerson;

#[test]
fn test_awesome_person_stats() {
    dotenv::from_filename("test.env").ok();

    let awesome_person_id = 1;
    verify_connection_migrate_db();
    let repo = DbAwesomePersonRepository;
    let current = repo
        .get_awesome_person_by_id(awesome_person_id)
        .expect("Should find progress stats");
    assert_eq!(
        current.id, awesome_person_id,
        "Progress Status ID should be 1"
    );

    let num_correct = current.num_correct.unwrap() + 1;
    let updating = AwesomePerson {
        num_correct: Some(num_correct),
        ..current
    };

    let num_updated = repo
        .update_awesome_person(updating)
        .expect("Should update progress stats");
    assert_eq!(num_updated, 1, "Should update 1 progress stats record");

    let updated = repo
        .get_awesome_person_by_id(awesome_person_id)
        .expect("Should find updated progress stats");

    assert_eq!(
        updated.id, awesome_person_id,
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
