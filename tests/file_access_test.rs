use palabras::config::TranslationsConfig;
use palabras::dal::file_access::{
    find_first_lang_translations,
};

#[test]
fn test_csv_parsing_with_commas() {
    let config = TranslationsConfig {
        file_name: "tests/data/es_en_mapping/llm_import.csv".to_string(),
        header_lines: 1,
        learning_index: 0,
        first_index: 4,
        delimiter: ",".to_string(),
        ..Default::default()
    };

    let learning_to_first_lang_map =
        find_first_lang_translations(&config).expect("Expected a hash map of translations");

    assert!(
        learning_to_first_lang_map.len() > 0,
        "Expected map to have at least one translation."
    );

    // for (key, value) in learning_to_first_lang_map {
    //     println!("{} -> {}", key, value);
    // }
}
