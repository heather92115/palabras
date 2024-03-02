use dotenv;
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::dal::translation_pair::{DbTranslationPairRepository, TranslationPairRepository};
use palabras::models::{NewTranslationPair, TranslationPair};
use rand::Rng;
use std::string::ToString;

pub static INTEGRATION_TEST_SKILL: &str = "integration test";

#[test]
fn test_create_translation() {
    dotenv::from_filename("test.env").ok();

    verify_connection_migrate_db();
    let repo = DbTranslationPairRepository;

    let pair = test_new_translation_pair_instance();

    let current = repo
        .find_translation_pair_by_learning_language(pair.learning_lang.clone())
        .unwrap_or_else(|_| None);

    // This is extremely likely
    if current.is_none() {
        let created = repo.create_translation_pair(&pair).expect("Create failed");

        let updated_percent = rand::thread_rng().gen_range(1..=100) as f64 / 100f64;
        let alternatives = "comprobar, examinar, examinar".to_string();
        let updating = TranslationPair {
            percentage_correct: Some(updated_percent),
            last_tested: Some(chrono::Utc::now()),
            guesses: Some(1),
            alternatives: Some(alternatives.clone()),
            ..created
        };

        let key_id = updating.id.clone();
        let num_updated = repo
            .update_translation_pair(updating)
            .expect("Update to previous create failed");
        assert_eq!(num_updated, 1, "Expected only one record to be updated");

        let updated = repo
            .get_translation_pair_by_id(key_id.clone())
            .expect("Should have found updated record");
        let tolerance = 0.01; // Define a suitable tolerance for the comparison of floats
        assert!(
            (updated.percentage_correct.unwrap() - updated_percent).abs() < tolerance,
            "Correctness should have matched. Result: {}, Expected: {}",
            updated.percentage_correct.unwrap(),
            updated_percent
        );

        assert_eq!(
            updated.guesses,
            Some(1),
            "Expected guesses to be 1 got {}",
            updated.guesses.unwrap()
        );
        assert_eq!(
            updated.alternatives.clone().unwrap(),
            alternatives,
            "Expected alternatives to match, Result {} expected {}",
            updated.alternatives.unwrap(),
            alternatives
        );

        let by_learning_lang = repo
            .find_translation_pair_by_learning_language(updated.learning_lang.clone())
            .expect("Lookup by learning lang should have worked")
            .expect("Lookup by learning lang option should unwrap.");

        assert_eq!(
            by_learning_lang.guesses,
            Some(1),
            "Expected guesses to be 1 got {}",
            by_learning_lang.guesses.unwrap()
        );
        assert_eq!(
            by_learning_lang.alternatives.clone().unwrap(),
            alternatives,
            "Expected alternatives to match, Result {} expected {}",
            by_learning_lang.alternatives.unwrap(),
            alternatives
        );

        alternatives.clone().split(',').for_each(|alt| {
            let by_an_alternative = repo
                .find_translation_pair_by_alternative(alt.to_string())
                .expect("Lookup by learning lang should have worked")
                .expect("Lookup by learning lang option should unwrap.");

            assert_eq!(
                by_an_alternative.guesses,
                Some(1),
                "Expected guesses to be 1 got {}",
                by_learning_lang.guesses.unwrap()
            );
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
    let repo = DbTranslationPairRepository;
    let num_records = 3;

    for _ in 0..num_records {
        let pair = test_new_translation_pair_instance();
        let missing_first_lang = NewTranslationPair {
            first_lang: "".to_string(),
            ..pair
        };

        let created = repo
            .create_translation_pair(&missing_first_lang)
            .expect("New record should be created");
        assert_eq!(
            created.first_lang.clone(),
            "",
            "Expected empty first lang but got {}",
            created.first_lang
        );
    }

    let list = repo
        .get_empty_first_lang_pairs(num_records.clone() + 1)
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

#[test]
fn test_get_study_pairs() {
    dotenv::from_filename("test.env").ok();

    verify_connection_migrate_db();
    let repo = DbTranslationPairRepository;
    let num_records = 3;

    for _ in 0..num_records {
        let pair = test_new_translation_pair_instance();
        let missing_first_lang = NewTranslationPair { ..pair };

        let created = repo
            .create_translation_pair(&missing_first_lang)
            .expect("New record should be created");
        assert_eq!(
            created.fully_known.clone(),
            false,
            "Expected false fully known flag"
        );
        assert_eq!(
            created.too_easy.clone(),
            false,
            "Expected false to easy flag"
        );
        assert_eq!(
            created.skill.clone().unwrap(),
            INTEGRATION_TEST_SKILL.to_string(),
            "Expected false to easy flag"
        );
    }

    let study_list = repo
        .get_study_pairs(num_records)
        .expect("Should have gotten some study pairs");
    assert!(
        study_list.len() >= num_records as usize,
        "Should have gotten more study records"
    );
}

pub fn test_new_translation_pair_instance() -> NewTranslationPair {
    let unique_num = rand::thread_rng().gen_range(1..=1000000);
    let learning_lang = format!("probar {}", unique_num);
    let first_lang = format!("to test {}", unique_num);

    NewTranslationPair {
        learning_lang,
        first_lang,
        percentage_correct: Some(0.45),
        created: chrono::Utc::now(),
        last_tested: None,
        fully_known: false,
        guesses: None,
        alternatives: None,
        skill: Some(INTEGRATION_TEST_SKILL.to_string()),
        too_easy: false,
        infinitive: None,
        pos: None,
        direction: None,
    }
}
