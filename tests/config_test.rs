use palabras::config::{load_translations_config, load_vocab_config};

#[test]
fn test_duo_vocab_config() {
    let config = load_vocab_config()
        .expect("JSON file should have deserialized to config");
    assert!(config.vocab_json_file_name.len() > 0, "Expected a filename")
}

#[test]
fn test_load_translation_config() {

    if let Some(configs) = load_translations_config()
        .expect("JSON file should have deserialized to a Vec of configs") {

        assert!(
            configs.len() > 0,
            "Expected at least one config object to load"
        );
    }
}
