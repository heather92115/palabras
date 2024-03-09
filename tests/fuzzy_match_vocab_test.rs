use palabras::config::{TranslationsConfig, VocabConfig};
use palabras::dal::db_connection::verify_connection_migrate_db;
use palabras::dal::vocab::{DbVocabRepository, VocabRepository};
use palabras::sl::fuzzy_match_vocab::{LearnVocab, VocabFuzzyMatch};
use palabras::sl::sync_vocab::import_duo_vocab;

/// Tests the duo lingo import by loading it into the test database.
#[test]
fn test_study_vocab_with_import() {
    use dotenv;
    dotenv::from_filename("test.env").ok();

    verify_connection_migrate_db();

    // There are 4 vocab words that will be translated back to the first language because they
    // are in the following llm_import.csv file used to find the first language translations missing
    // from duolingo.
    // viajas, miraste, quedan, visito
    let vocab_config = VocabConfig {
        vocab_json_file_name: "tests/data/testing_study_vocab.json".to_string(),
        plural_suffix: None,
        non_verb_matching_suffixes: None,
        pronouns: None
    };

    let translation_configs = vec![
        TranslationsConfig {
            file_name: "tests/data/es_en_mapping/llm_import.csv".to_string(),
            header_lines: 1,
            learning_index: 0,
            first_index: 4,
            delimiter: ",".to_string(),
            ..Default::default()
        },
    ];

    let awesome_person_id = 1;

    // Runs the import and translates any vocab found in the llm import.
    import_duo_vocab(&vocab_config, Some(translation_configs), awesome_person_id).unwrap_or_else(|err| {
        eprintln!("Problem processing word pairs: {}", err);
        panic!("Import failed");
    });

    // Verifying words were imported and translated at expected.
    check_vocab_expectations("viajas".to_string(), "you travel".to_string());
    check_vocab_expectations("miraste".to_string(), "you looked".to_string());
    check_vocab_expectations("quedan".to_string(), "they remain".to_string());
    check_vocab_expectations("visito".to_string(), "I visit".to_string());

    // Now the real test starts
    let match_service = VocabFuzzyMatch::instance();
    let study_set
        = match_service
        .get_vocab_to_learn(awesome_person_id, i64::MAX)
        .expect("Expect vocab request to work");

    assert!(study_set.len() >= 4, "Expected at least 4, there may be others");

    let (_, quedan_v)
        = study_set
            .into_iter()
            .find(|(_, v)| v.learning_lang.eq("quedan"))
            .expect("Should have found 'quedan'");

    // Check a perfect match
    let distance = match_service
        .check_vocab_match(
            &quedan_v.learning_lang,
            &quedan_v.alternatives.unwrap_or_default(),
            &quedan_v.learning_lang);
    assert_eq!(distance, 0, "Should have match and therefore been 0")
}

/// Checks that vocab loaded
fn check_vocab_expectations(learning: String, first: String) {
    // Verifying our words were imported and translated at expected.
    let vocab_repo = DbVocabRepository;
    if let Ok(Some(vocab)) = vocab_repo.find_vocab_by_learning_language(learning.clone()) {
        assert_eq!(vocab.learning_lang, learning, "Expected {}", learning);
        assert_eq!(vocab.first_lang, first, "Expected {}", first);
    } else {
        panic!("Should have returned result.")
    }
}
