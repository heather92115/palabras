use dotenv::dotenv;
use palabras::dal::awesome_person::{AwesomePersonRepository, DbAwesomePersonRepository};
use palabras::dal::db_connection::{establish_connection_pool, verify_connection_migrate_db};
use palabras::models::{AwesomePerson, NewAwesomePerson};
use rand::Rng;
use std::env;

fn get_test_db_url() -> String {
    env::var("PAL_TEST_DATABASE_URL").expect("env var TEST_DATABASE_URL was not found")
}

#[test]
fn test_awesome_person_stats() {
    dotenv().ok(); // Load environment variables from .env file

    let awesome_person_id = 1;
    establish_connection_pool(get_test_db_url());
    verify_connection_migrate_db().expect("connection and migration should have worked");
    let repo = DbAwesomePersonRepository;

    let current = repo
        .get_awesome_person_by_id(awesome_person_id)
        .expect("Should find progress stats")
        .unwrap_or_default();
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
        .expect("Should find updated progress stats")
        .unwrap_or_default();

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

#[test]
fn test_create_awesome_person() {
    dotenv().ok(); // Load environment variables from .env file
    establish_connection_pool(get_test_db_url());
    verify_connection_migrate_db().expect("connection and migration should have worked");
    let repo = DbAwesomePersonRepository;
    let test_name = "Alice".to_string();
    let unique_num = rand::thread_rng().gen_range(100000..=1000000000);
    let sec_code = format!("test-code{}", unique_num);

    let awesome_person = NewAwesomePerson {
        name: Some(test_name.clone()),
        sec_code,
        max_learning_words: 2,

        ..Default::default()
    };

    let created = repo
        .create_awesome_person(&awesome_person)
        .expect("New awesome person should have been created");
    assert!(!created.id.to_string().is_empty(), "Expected the ID");
    assert_eq!(
        created.name.clone().unwrap_or_default(),
        test_name,
        "Expected the name to be '{}', actual '{}'",
        test_name,
        created.name.unwrap_or_default()
    );

    let found = repo
        .get_awesome_person_by_id(created.id)
        .expect("Should find newly created awesome person")
        .unwrap_or_default();
    assert_eq!(
        found.id, created.id,
        "Awesome person ids mismatched, expected {}, actual {}",
        created.id, found.id
    );

    let found = repo
        .get_awesome_person_by_code(created.sec_code)
        .expect("Should find newly created awesome person")
        .unwrap_or_default();
    assert_eq!(
        found.id, created.id,
        "Awesome person ids mismatched, expected {}, actual {}",
        created.id, found.id
    );
}
