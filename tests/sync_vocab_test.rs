use palabras::config::{VocabConfig, TranslationsConfig};
use palabras::dal::db_connection::{verify_connection_migrate_db};
use palabras::dal::translation_pair::{DbTranslationPairRepository, TranslationPairRepository};
use palabras::models::TranslationPair;
use palabras::sl::sync_vocab::{import_duo_vocab, load_vocab_from_json};

#[test]
fn test_load_from_json() {
    let vocab_json = "tests/data/testing_small_vocab.json";

    let vocab = load_vocab_from_json(vocab_json).expect("Import should work");
    assert!(
        !vocab.vocab_overview.is_empty(),
        "Should have found a vocab list."
    )
}

/// Tests the duo lingo import by loading it into the test database.
#[test]
fn test_import_vocab_use_xml_no_combining() {
    use dotenv;
    dotenv::from_filename("test.env").ok();

    verify_connection_migrate_db();

    let vocab_config = VocabConfig {
             vocab_json_file_name: "tests/data/testing_small_vocab.json".to_string(),
             plural_suffix: None,
             non_verb_matching_suffixes: None,
    };

    let translation_configs = vec![
        TranslationsConfig {
            file_name: "data/mananoreboton/short-es-en.xml".to_string(),
            header_lines: 4,
            learning_regex: Some(r#"<c>([^"]+)</c>"#.to_string()),
            first_regex: Some(r#"<d>([^"]+)</d>"#.to_string()),
            ..Default::default()
        },
        TranslationsConfig {
            file_name: "tests/data/es_en_mapping/llm_import.csv".to_string(),
            header_lines: 1,
            learning_index: 0,
            first_index: 4,
            delimiter: ",".to_string(),
            ..Default::default()
        },
    ];

    import_duo_vocab(&vocab_config, Some(translation_configs)).unwrap_or_else(|err| {
        eprintln!("Problem processing word pairs: {}", err);
        panic!("Import failed");
    });

    let repo = DbTranslationPairRepository;

    if let Ok(list) = repo.get_empty_first_lang_pairs(10) {
        assert!(list.len() > 0, "Expected records");
    } else {
        panic!("Should have returned result.")
    }
}

#[test]
fn test_import_small_vocab_with_llm_translations() {
    use dotenv;
    dotenv::from_filename("test.env").ok();

    verify_connection_migrate_db();

    let vocab_config = VocabConfig {
        vocab_json_file_name: "tests/data/testing_small_vocab.json".to_string(),
        plural_suffix: Some("s".to_string()),
        non_verb_matching_suffixes: None,
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

    import_duo_vocab(&vocab_config, Some(translation_configs)).unwrap_or_else(|err| {
        eprintln!("Problem processing word pairs: {}", err);
        panic!("Import failed");
    });

    let repo = DbTranslationPairRepository;

    if let Ok(list) = repo.get_empty_first_lang_pairs(10) {
        assert!(list.len() > 0, "Expected records");
    } else {
        panic!("Should have returned result.")
    }
}

#[test]
fn test_import_duo_vocab_no_xml() {
    use dotenv;
    dotenv::from_filename("test.env").ok();

    verify_connection_migrate_db();

    let vocab_config = VocabConfig {
        vocab_json_file_name: "tests/data/testing_playa.json".to_string(),
        plural_suffix: Some("s".to_string()),
        non_verb_matching_suffixes: Some("o,a,os,as".to_string()),
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

    import_duo_vocab(&vocab_config, Some(translation_configs)).unwrap_or_else(|err| {
        eprintln!("Problem processing word pairs: {}", err);
        panic!("Import failed");
    });

    let repo = DbTranslationPairRepository;

    if let Ok(list) = repo.get_empty_first_lang_pairs(10) {
        assert!(list.len() > 0, "Expected records");
    } else {
        panic!("Should have returned result.")
    }
}

#[test]
fn test_import_vocab_combine_similar_playa() {
    use dotenv;
    dotenv::from_filename("test.env").ok();

    verify_connection_migrate_db();

    let vocab_config = VocabConfig {
        vocab_json_file_name: "tests/data/testing_playa.json".to_string(),
        plural_suffix: Some("s".to_string()),
        non_verb_matching_suffixes: Some("o,a,os,as".to_string()),
    };

    import_duo_vocab(&vocab_config, None).unwrap_or_else(|err| {
        eprintln!("Problem processing word pairs: {}", err);
        panic!("Import failed");
    });

    let repo = DbTranslationPairRepository;

    // Get them all to make sure our records get included
    if let Ok(list) = repo.get_empty_first_lang_pairs(i64::MAX) {

        let filtered: Vec<TranslationPair> = list.into_iter().filter(|tp| tp.learning_lang.starts_with("testingplaya")).collect();
        assert_eq!(filtered.len(), 1, "Expected a single record");
        assert_eq!(filtered[0].learning_lang, "testingplaya", "Expected singular");
        assert_eq!(filtered[0].alternatives.as_deref().unwrap_or_default(), "testingplayas", "Expected plural");
    } else {
        panic!("Should have returned result.")
    }
}

#[test]
fn test_import_vocab_combine_similar_amarilla() {
    use dotenv;
    dotenv::from_filename("test.env").expect("Should have loaded test.env");

    verify_connection_migrate_db();

    let vocab_config = VocabConfig {
        vocab_json_file_name: "tests/data/testing_amarilla.json".to_string(),
        plural_suffix: None, // No swap of plural to singular for learning_lang (main word/phrase)
        non_verb_matching_suffixes: Some("o,a,os,as".to_string()),
    };

    import_duo_vocab(&vocab_config, None).unwrap_or_else(|err| {
        eprintln!("Problem processing word pairs: {}", err);
        panic!("Import failed");
    });

    let repo = DbTranslationPairRepository;

    // Get them all to make sure our records get included
    if let Ok(list) = repo.get_empty_first_lang_pairs(i64::MAX) {

        let filtered: Vec<TranslationPair> = list.into_iter().filter(|tp| tp.learning_lang.starts_with("testingamarill")).collect();
        assert_eq!(filtered.len(), 1, "Expected a single record");
        assert_eq!(filtered[0].learning_lang, "testingamarillas", "Expected plural feminine since it was found first in json");
        assert!(filtered[0].alternatives.as_deref().unwrap_or_default().starts_with("testingamarilla"), "Expected starts with singular");
    } else {
        panic!("Should have returned result.")
    }
}
