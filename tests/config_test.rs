use palabras::config::load_translations_config;

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
