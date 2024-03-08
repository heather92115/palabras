use dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::dal::vocab::{DbVocabRepository, VocabRepository};
use palabras::models::{NewVocab, Vocab};
use rand::Rng;
use std::string::ToString;

pub static INTEGRATION_TEST_SKILL: &str = "integration test";

#[test]
fn test_create_translation() {
    dotenv::from_filename("test.env").ok();

    verify_connection_migrate_db();
    let repo = DbVocabRepository;

    let new_vocab = test_new_vocab_instance();

    let current = repo
        .find_vocab_by_learning_language(new_vocab.learning_lang.clone())
        .unwrap_or_else(|_| None);

    // This is extremely likely
    if current.is_none() {
        let created = repo.create_vocab(&new_vocab).expect("Create failed");

        let alternatives = "comprobar, examinar, examinar".to_string();
        let updating = Vocab {
            alternatives: Some(alternatives.clone()),
            ..created
        };

        let num_updated = repo
            .update_vocab(updating)
            .expect("Update to previous create failed");
        assert_eq!(num_updated, 1, "Expected only one record to be updated");

        let by_learning_lang = repo
            .find_vocab_by_learning_language(new_vocab.learning_lang.clone())
            .expect("Lookup by learning lang should have worked")
            .expect("Lookup by learning lang option should unwrap.");

        assert_eq!(
            by_learning_lang.alternatives.clone().unwrap(),
            alternatives,
            "Expected alternatives to match, Result {} expected {}",
            by_learning_lang.alternatives.unwrap(),
            alternatives
        );

        alternatives.clone().split(',').for_each(|alt| {
            let by_an_alternative = repo
                .find_vocab_by_alternative(alt.to_string())
                .expect("Lookup by learning lang should have worked")
                .expect("Lookup by learning lang option should unwrap.");

            assert_eq!(
                by_an_alternative.alternatives.clone().unwrap(),
                alternatives,
                "Expected alternatives to match, Result {} expected {}",
                by_learning_lang.alternatives.clone().unwrap(),
                alternatives
            );
        });
    }
}

#[test]
fn test_fix_first_lang() {
    dotenv::from_filename("test.env").ok();

    verify_connection_migrate_db();
    let repo = DbVocabRepository;
    let num_records = 3;

    for _ in 0..num_records {
        let pair = test_new_vocab_instance();
        let missing_first_lang = NewVocab {
            first_lang: "".to_string(),
            ..pair
        };

        let created = repo
            .create_vocab(&missing_first_lang)
            .expect("New record should be created");
        assert_eq!(
            created.first_lang.clone(),
            "",
            "Expected empty first lang but got {}",
            created.first_lang
        );
    }

    let list = repo
        .get_empty_first_lang(num_records.clone() + 1)
        .expect("Should have gotten records with no first lang");
    assert!(
        list.len() >= num_records as usize,
        "Should have gotten more records"
    );
    for found in list {
        assert_eq!(
            found.first_lang.clone(),
            "",
            "Expected empty first lang but got {}",
            found.first_lang
        );
    }
}

pub fn test_new_vocab_instance() -> NewVocab {
    let unique_num = rand::thread_rng().gen_range(1..=1000000);
    let learning_lang = format!("probar {}", unique_num);
    let first_lang = format!("to test {}", unique_num);

    NewVocab {
        learning_lang,
        first_lang,
        skill: Some(INTEGRATION_TEST_SKILL.to_string()),
        ..Default::default()
    }
}
