use crate::config::{VocabConfig, TranslationsConfig};
use crate::dal::file_access::{
    find_first_lang_translations, load_buffer_from_file, write_missing_first_export,
};
use crate::dal::translation_pair::{DbTranslationPairRepository, TranslationPairRepository};
use crate::models::{NewTranslationPair, TranslationPair};
use crate::sl::learn_pairs::{FULLY_KNOWN_THRESHOLD, TOO_EASY_THRESHOLD};
use diesel::result::Error as DieselError;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;

/// This service handles importing Duolingo vocab exports. It looks for close matches in words and
/// combines then using the alternative field.

/// These structs are used to deserialize the Duolingo JSON file. It is possible their schema will change
/// which will require these structs to be updated to match.
#[derive(Serialize, Deserialize, Debug)]
pub struct LanguageData {
    language_string: String,
    learning_language: String,
    from_language: String,
    language_information: LanguageInformation,
    pub vocab_overview: Vec<VocabOverview>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LanguageInformation {
    pronoun_mapping: Vec<PronounMapping>,
    tenses: Tenses,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PronounMapping {
    tenses: HashMap<String, String>,
    pronoun: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tenses {
    indicative: Vec<TenseDetail>,
    subjunctive: Vec<TenseDetail>,
    others: Vec<TenseDetail>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TenseDetail {
    tense_string: String,
    tense: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VocabOverview {
    #[serde(default)]
    strength_bars: u8,
    infinitive: Option<String>,
    #[serde(default)]
    normalized_string: String,
    pos: Option<String>,
    #[serde(default)]
    last_practiced_ms: u64,
    #[serde(default)]
    skill: String,
    #[serde(default = "default_related_lexemes")]
    related_lexemes: Vec<String>,
    #[serde(default)]
    last_practiced: Option<String>,
    strength: f64,
    skill_url_title: Option<String>,
    gender: Option<String>,
    #[serde(default)]
    id: String,
    #[serde(default)]
    lexeme_id: String,
    word_string: String,
}

fn default_related_lexemes() -> Vec<String> {
    vec![]
}

/// Loads vocabulary data from a JSON file into a `LanguageData` struct.
///
/// This function is designed to parse vocabulary data stored in a JSON format file, converting it
/// into a `LanguageData` struct for further processing or analysis. It handles opening the file,
/// reading its contents, and deserializing the JSON into the specified Rust data structure.
///
/// # Arguments
///
/// * `json_file_name` - A string slice that holds the name (and path, if necessary) of the JSON
/// file containing the vocabulary data.
///
/// # Returns
///
/// Returns `Ok(LanguageData)` containing the deserialized vocabulary data if successful, or an
/// `Err(String)` containing an error message if the operation fails.
///
/// # Errors
///
/// This function can return an error in several cases, including:
/// - The JSON file cannot be opened (e.g., due to incorrect path or permissions issues).
/// - The file's contents cannot be properly deserialized into the `LanguageData` struct (e.g., due
/// to mismatched data formats or corrupted data).
///
/// # Example
///
/// Assuming a JSON file named "vocab_data.json" exists in the current directory and contains
/// valid vocabulary data formatted for the `LanguageData` struct:
///
/// ```
/// use std::error::Error;
/// use palabras::sl::sync_vocab::load_vocab_from_json;
/// let vocab_data = load_vocab_from_json("tests/data/duo_vocab.json");
/// println!("Loaded vocabulary data successfully.");
///
/// ```
pub fn load_vocab_from_json(json_file_name: &str) -> Result<LanguageData, String> {
    // Load the JSON file into a buffer.
    let reader = load_buffer_from_file(json_file_name).map_err(|err| err.to_string())?;
    let vocab_overview = serde_json::from_reader(reader).map_err(|err| err.to_string())?;

    Ok(vocab_overview)
}

/// Merges additional learning material into the current translation pair.
///
/// This function updates the `current` translation pair by potentially swapping its
/// `learning_lang` field with the `additional_learning` string, if the latter represents
/// a singular form matching the plural form in `current.learning_lang`. If the `additional_learning`
/// is not a match or a singular form of the current learning language, it's added to the list of
/// alternatives, avoiding duplicates.
///
/// # Arguments
///
/// * `current` - A mutable reference to the current translation pair being updated.
/// * `additional_learning` - The new word or phrase to be integrated into the translation pair.
/// * `plural_suffix` - The suffix indicating a plural form in the learning language.
///
/// # Examples
///
/// ```
/// use palabras::models::TranslationPair;
/// use palabras::sl::sync_vocab::merge_learning;
/// let mut pair = TranslationPair {
///     learning_lang: "cats".to_string(),
///     alternatives: None,
///     ..Default::default()
/// };
/// merge_learning(&mut pair, "cat".to_string(), "s");
/// assert_eq!(pair.learning_lang, "cat");
/// assert_eq!(pair.alternatives, Some("cats".to_string()));
///
/// // Adding a new alternative that is not a singular form or already listed
/// merge_learning(&mut pair, "kitty".to_string(), "s");
/// assert_eq!(pair.learning_lang, "cat");
/// assert_eq!(pair.alternatives, Some("cats, kitty".to_string()));
/// ```
pub fn merge_learning(
    current: &mut TranslationPair,
    additional_learning: String,
    plural_suffix: &str,
) {

    if current.learning_lang.ne(&additional_learning) {
        // See if the learning lang is in plural form and should be swapped with the new word.
        let (learning, additional) = if current.learning_lang
            .strip_suffix(plural_suffix)
            .unwrap_or_default()
            .eq(&additional_learning)
        {
            (additional_learning, current.learning_lang.clone())
        } else {
            (current.learning_lang.clone(), additional_learning)
        };
        current.learning_lang = learning;

        let current_alts = current.alternatives.clone().unwrap_or_default();
        if !current_alts.contains(&additional) {
            if current_alts.is_empty() {
                current.alternatives = Some(additional);
            } else {
                current.alternatives = Some(format!("{}, {}", current_alts, additional));
            }
        }
    }
}

/// Merges vocabulary overview information into a translation pair, updating various attributes.
///
/// This function consolidates the incoming information from a `VocabOverview` item into the given `TranslationPair`,
/// updating the learning and native language representations, strength indicators, and grammatical metadata
/// based on the configuration provided through `VocabConfig`. It ensures that the most relevant and updated
/// information is reflected in the `TranslationPair`.
///
/// # Arguments
///
/// * `vocab_config` - A reference to the configuration settings for vocabulary handling, including pluralization rules. Settings are found at `vocab_config.json`
/// * `current` - A mutable reference to the translation pair being updated with new information.
/// * `item` - A reference to the vocabulary overview containing the new information to be merged into `current`.
/// * `first` - The translation of the item back to the first language, used if `current` does not already have a translation.
///
/// # Examples
///
/// ```
/// // Assume the existence of appropriate structure and enum definitions for this example.
/// use palabras::config::VocabConfig;
/// use palabras::models::TranslationPair;
/// use palabras::sl::sync_vocab::{merge, VocabOverview};
/// let vocab_config = VocabConfig {
///     vocab_json_file_name: "".to_string(),
///     plural_suffix: Some("s".to_string()),
///     non_verb_matching_suffixes: None,};
///
/// let mut translation_pair = TranslationPair {
///     learning_lang: "gato".to_string(),
///     first_lang: "".to_string(),
///     percentage_correct: None,
///     ..Default::default()
/// };
///
/// let json_str = r#"{"infinitive": null,"pos": "Noun","strength": 0.9999,"gender": "Feminine","word_string": "gatos"}"#;
/// let vocab_overview: VocabOverview =
/// serde_json::from_str(json_str).expect("JSON should deserialize");
///
/// merge(&vocab_config, &mut translation_pair, &vocab_overview, "cat".to_string());
///
/// assert_eq!(translation_pair.learning_lang, "gato");
/// assert_eq!(translation_pair.first_lang, "cat");
/// assert_eq!(translation_pair.alternatives, Some("gatos".to_string()));
/// assert!(translation_pair.fully_known);
/// ```
pub fn merge(
    vocab_config: &VocabConfig,
    current: &mut TranslationPair,
    item: &VocabOverview,
    first: String,
) {

    // Determine placement of new learning word or phrase
    merge_learning(current,
                   item.word_string.clone(),
                   &vocab_config.plural_suffix.clone().unwrap_or_default(),
    );

    // Update the translation back to the first language if it wasn't already translated
    if current.first_lang.clone().is_empty() {
        current.first_lang = first;
    }

    // Update the strength if it is higher
    if item.strength > current.percentage_correct.clone().unwrap_or_default() {
        current.percentage_correct = Some(item.strength);
        current.fully_known = item.strength > FULLY_KNOWN_THRESHOLD;
        current.too_easy = item.strength > TOO_EASY_THRESHOLD;
    }

    current.infinitive = Some(item.infinitive.clone().unwrap_or_default());
    current.pos = Some(item.pos.clone().unwrap_or_default());
}

/// Creates a new translation pair and stores it in the database.
///
/// This function constructs a `NewTranslationPair` from given `VocabOverview` and `first` language
/// translation string, and then attempts to store this new translation pair in the database
/// using a database repository (`DbTranslationPairRepository`). If the operation is successful, it
/// returns `Ok(())`. In case of any database operation failure, it returns an `Err` with an error message.
///
/// # Arguments
///
/// * `item` - A reference to a `VocabOverview` instance containing data for the learning language side
/// of the translation pair.
/// * `first` - A `String` representing the translation of the item in the first language.
///
/// # Returns
///
/// A result indicating the success (`Ok`) or failure (`Err`) of the creation operation.
///
/// # Errors
///
/// This function returns an error if any issue is encountered during the database operation, encapsulated
/// as a `String` in the `Err` variant of the result.
///
/// # Examples
///
/// Due to the dependency on a database connection, detailed examples of using this function are
/// located in the integration tests for this function.
///
/// # See Also
///
/// It's recommended to review integration tests for examples of how to use this function, as they
/// demonstrate its use within a larger context, including setup and teardown of relevant
/// database state.
///
/// ```ignore
/// // Example usage (simplified and hypothetical, see integration tests for actual examples)
///
/// // Assume `item` is a `VocabOverview` populated with relevant data for a new vocabulary item
/// // and `first` is the translation of `item` into the user's primary language.
/// use palabras::sl::sync_vocab::create;
/// let result = create(&item, "First Language Translation".to_string());
///
/// match result {
///     Ok(_) => println!("Translation pair was successfully created."),
///     Err(e) => println!("Failed to create translation pair: {}", e),
/// }
/// ```
pub fn create(item: &VocabOverview, first: String) -> Result<(), String> {
    // Get the dal repo for translation pairs. It requires a database connection.
    let pair_repo = DbTranslationPairRepository;

    let new_translation_pair = NewTranslationPair {
        first_lang: first,
        learning_lang: item.word_string.clone(),
        percentage_correct: Some(item.strength.clone()),
        fully_known: item.strength > FULLY_KNOWN_THRESHOLD,
        too_easy: item.strength > TOO_EASY_THRESHOLD,
        skill: Some(item.skill.clone()),
        infinitive: Some(item.infinitive.clone().unwrap_or_default()),
        pos: Some(item.pos.clone().unwrap_or_default()),
        // Other fields use their default values
        ..Default::default()
    };

    pair_repo
        .create_translation_pair(&new_translation_pair)
        .map_err(|err| err.to_string())?;

    Ok(())
}

/// Searches for a translation pair with a word similar to `learning_lang`, differing only by specified suffixes.
///
/// This function is intended to reduce redundancy in vocabulary by identifying and reusing existing translation
/// pairs that represent the same word in different forms (e.g., singular/plural, masculine/feminine). It does so by
/// iterating through a list of allowed suffix changes, attempting to find a match in the database.
///
/// # Arguments
///
/// * `non_verb_matching_suffixes` - A `&str` containing a comma-separated list of suffixes to be considered
/// for matching similar words. Used to construct alternative word forms by replacing these suffixes in `learning_lang`.
/// * `learning_lang` - A `&str` representing the word in the learning language for which a similar existing translation
/// pair is being sought.
///
/// # Returns
///
/// This function returns a `Result` object which, on success, contains an `Option<TranslationPair>`. The contained
/// `Option` is `Some(TranslationPair)` if a translation pair with a similar word is found, or `None` if no similar
/// word could be found. An error of type `DieselError` is returned in case of database access issues.
///
/// # Errors
///
/// This function may return a `DieselError` if there is an issue during the database query operation, such as a
/// connection problem or a syntax error in the query.
///
/// # Examples
///
/// Due to the dependency on database interaction, specific examples of using this function cannot
/// easily be provided outside the context of an existing database session. Below is a hypothetical usage:
///
/// ```ignore
/// // Assume an existing `DbTranslationPairRepository` and a connection to a database
///
/// // If "gato, gata or gatas" is in the database, its translation_pair will be found and returned.
/// let non_verb_suffixes = "o,a,os,as,e,es";
/// let learning_lang_word = "gatos";
///
/// match find_similar(non_verb_suffixes, learning_lang_word) {
///     Ok(Some(translation_pair)) => {
///         println!("Found a similar word: {}", translation_pair.learning_lang);
///     },
///     Ok(None) => {
///         println!("No similar word found.");
///     },
///     Err(e) => {
///         println!("Database error: {}", e);
///     }
/// }
/// ```
///
/// Please note: This example assumes a specific database schema and runtime environment, including an instantiated
/// `DbTranslationPairRepository`, and thus is not directly runnable.
fn find_similar(
    non_verb_matching_suffixes: &str,
    learning_lang: &str,
) -> Result<Option<TranslationPair>, DieselError> {
    let pair_repo = DbTranslationPairRepository;

    let learning = learning_lang.to_lowercase();

    // Find the original suffix and proceed if there is a match, ex: gato will be matched by the 'o' suffix
    if let Some(ori_suffix) = non_verb_matching_suffixes
        .split(',')
        .find(|suffix| learning.ends_with(suffix))
    {
        // The learning word was matched to a suffix, now iterate over all suffixes, looking for alternatives.
        for alt_suffix in non_verb_matching_suffixes.split(',') {
            // Skip the original suffix to avoid redundant checks.
            if alt_suffix == ori_suffix {
                continue;
            }

            // Construct the alternative word by replacing the original suffix with the alternative suffix.
            if let Some(stem) = learning.strip_suffix(ori_suffix) {  // ex: gato becomes gat
                let alt_word = format!("{}{}", stem, alt_suffix);  // ex: gat becomes gata, gatos, gatas

                // Attempt to search for a translation pair using the newly contructed alternative
                if let Ok(Some(translation_pair)) =
                    pair_repo.find_translation_pair_by_learning_language(alt_word)
                {
                    return Ok(Some(translation_pair)); // Found a similar word form, return it
                }
            }
        }
    }

    Ok(None)
}

/// Searches for an existing translation pair corresponding to a given word or phrase.
///
/// This function attempts to find a `TranslationPair` by sequentially searching in three ways:
/// 1. Directly by the learning language,
/// 2. By any known alternatives for the word, (already combined)
/// 3. By similar words, using defined suffix rules for variations (e.g., plural forms, genders).
///
/// The search strategy aims to maximize the chances of finding a relevant translation pair without creating duplicates
/// for words that are essentially the same but may appear in different forms.
///
/// # Arguments
///
/// * `vocab_config` - A reference to the `VocabConfig` struct that contains configuration settings for vocabulary
///   processing, including suffixes for matching similar words.
/// * `item` - A reference to a `VocabOverview` struct providing details about the word or phrase for which a
///   translation pair is sought.
///
/// # Returns
///
/// Returns a `Result` with an `Option<TranslationPair>` on success:
/// - `Some(TranslationPair)` if a matching translation pair is found by any of the search strategies.
/// - `None` if no matching translation pair exists.
/// An `Err` is returned with a descriptive error message in case of a failure during database operations.
///
/// # Errors
///
/// Returns an error (encapsulated in a `String`) if there's a failure at any point during the database query
/// operations, including issues with finding similar word matches.
///
/// # Examples
///
/// Due to its dependency on external configurations (`VocabConfig`) and a database environment, directly runnable
/// examples of this function cannot be easily provided. Below is a conceptual example of how this function might
/// be used within a larger application context:
///
/// ```ignore
/// // Assume `vocab_config` is an instance of VocabConfig with relevant settings,
/// // and `item` is a VocabOverview filled with information about a specific word.
///
/// match find_translation_pair(&vocab_config, &item) {
///     Ok(Some(translation_pair)) => {
///         println!("Found a matching translation pair: {:?}", translation_pair);
///     },
///     Ok(None) => {
///         println!("No matching translation pair found.");
///     },
///     Err(e) => {
///         println!("An error occurred while searching: {}", e);
///     }
/// }
/// ```
pub fn find_translation_pair(
    vocab_config: &VocabConfig,
    item: &VocabOverview,
) -> Result<Option<TranslationPair>, String> {
    let pair_repo = DbTranslationPairRepository;

    // First, try finding the translation pair by the learning language directly.
    if let Ok(Some(current)) =
        pair_repo.find_translation_pair_by_learning_language(item.word_string.clone())
    {
        return Ok(Some(current));
    }

    // Second, try finding if it's already an alternative
    if let Ok(Some(current)) =
        pair_repo.find_translation_pair_by_alternative(item.word_string.clone())
    {
        return Ok(Some(current));
    }

    // Can this word or phrase be safely matched against alternatives or plural forms?
    // And did the config specify the suffixes?
    // Then thirdly, attempt to find a similar translation pair.
    if let Some(ref suffixes) = vocab_config.non_verb_matching_suffixes {
        // Check if the word is a candidate for matching against alternatives or plural forms.
        if item.infinitive.is_none() && item.gender.is_some() && item.word_string.len() > 2 {
            return find_similar(&suffixes, &item.word_string).map_err(|err| err.to_string());
        }
    }

    Ok(None)
}

// Creating a mutex to guard the complex logic within process_duo_vocab
lazy_static! {
    static ref PROCESS_MUTEX: Mutex<()> = Mutex::new(());
}

/// Processes vocabulary data and integrates it into the database.
///
/// This function iterates over a given set of vocabulary items, performing a series of steps to ensure each item is correctly
/// stored in the database. It aims to prevent duplication and maintain consistency in how vocabulary information, including
/// translations and alternative forms, is managed. The process includes:
///
/// 1. Attempting to find an existing translation pair for each vocabulary item.
/// 2. Merging new data with existing entries if found, or creating new entries otherwise.
/// 3. Handling plural forms and gender variations through configurable suffixes.
///
/// Before processing, the function acquires a mutex lock to ensure thread safety during the import and update operations.
///
/// # Arguments
///
/// * `vocab_config` - A reference to a `VocabConfig` structure containing settings and preferences for vocabulary processing. From `vocab_config.json`
/// * `vocabulary_overview` - A reference to a `LanguageData` structure that encapsulates the vocabulary items to be processed. Imported DuoVocab from `https://www.duolingo.com/vocabulary/overview`
/// * `translation_map` - A reference to a `HashMap<String, String>` that maps learning language terms to their translations in the first language. Loaded from CSV files using `translations_config.json`
///
/// # Returns
///
/// Returns `Ok(())` if all vocabulary items have been successfully processed and integrated into the database.
/// Returns `Err(String)` if an error occurs at any point during the processing, with an error message indicating the cause of the failure.
///
/// # Errors
///
/// This function may return an error if:
/// - It fails to acquire the mutex lock necessary for thread-safe operations.
/// - There are issues accessing the database, such as connection problems or errors in query execution.
/// - Any of the utilized subroutines (`find_translation_pair`, `merge`, or `create`) encounter errors in their execution.
///
/// # Examples
///
/// Due to dependencies on external configurations (`VocabConfig`), a database, and a specific
/// runtime environment, directly runnable examples are not provided here. However, a conceptual usage scenario might look like this:
///
/// ```ignore
/// // Assume the existence of a `vocab_config` object, a `vocabulary_overview` structure filled with data, and
/// // a `translation_map` that provides translations for the vocabulary terms.
///
/// if let Err(e) = process_duo_vocab(&vocab_config, &vocabulary_overview, &translation_map) {
///     println!("Failed to process vocabulary data: {}", e);
/// } else {
///     println!("Vocabulary data processed successfully.");
/// }
/// ```
fn process_duo_vocab(
    vocab_config: &VocabConfig,
    vocabulary_overview: &LanguageData,
    translation_map: &HashMap<String, String>,
) -> Result<(), String> {
    // Acquire the lock before proceeding with import and updates.
    let _lock = PROCESS_MUTEX.lock().map_err(|e| e.to_string())?;

    // Get the dal repo for translation pairs. It requires a database connection.
    let pair_repo = DbTranslationPairRepository;

    for item in &vocabulary_overview.vocab_overview {
        let learning_lang = &item.word_string;

        // Fetch the translated first language if available
        let translated_first = translation_map
            .get(learning_lang)
            .cloned()
            .unwrap_or_default();

        // Attempt to find an existing translation pair by learning language.
        match find_translation_pair(vocab_config, item) {
            Ok(Some(mut current)) => {
                merge(vocab_config, &mut current, item, translated_first);
                pair_repo.update_translation_pair(current)
                        .map_err(|err| err.to_string())?;
            }
            Ok(None) => create(item, translated_first)?,
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(())
}

/// Loads translations into a `HashMap` from CSV or XML files as specified by configuration.
///
/// This function reads translation data from files whose paths and parsing details are provided
/// in `translation_configs`. It consolidates translations into a single `HashMap` where each key-value
/// pair represents a term in the learning language and its translation in the user's first language.
///
/// # Arguments
///
/// * `translation_configs` - An optional vector of `TranslationsConfig` detailing the paths of the translation
///   source files (either CSV or XML) and how they should be parsed. Each `TranslationsConfig` should specify:
///   - `file_name`: Path to the translation file.
///   - `header_lines`: Number of header lines to skip (useful for CSV files).
///   - `delimiter`: Delimiter used in CSV files. Leave empty for XML files.
///   - `learning_index` and `first_index`: In CSV files, the column indices of the learning and first languages.
///      In XML, these are ignored.
///   - `learning_regex` and `first_regex`: Regular expressions to extract translation pairs from XML files. These
///      should form capturing groups for the learning and first language terms. Ignored for CSV files.
///
/// # Returns
///
/// Returns a `HashMap<String, String>` where the key is a word or phrase in the learning language, and the value is its
/// corresponding translation in the user's first language. If `translation_configs` is `None` or empty, or if all specified
/// files fail to load or parse, this map will be empty.
///
/// # Example Configuration
///
/// ```
/// use palabras::config::TranslationsConfig;
/// use palabras::sl::sync_vocab::load_translations;
/// let configs: Vec<TranslationsConfig> = vec![
///    TranslationsConfig {
///        file_name: "data/mananoreboton/short-es-en.xml".to_string(),
///        header_lines: 0,
///        delimiter: "".to_string(),
///        learning_index: 0,
///        first_index: 0,
///        learning_regex: Some("<src>([^<]+)</src>".to_string()),
///        first_regex: Some("<tgt>([^<]+)</tgt>".to_string()),
///    },
///    TranslationsConfig {
///        file_name: "tests/data/es_en_mapping/llm_import.csv".to_string(),
///        header_lines: 1,
///        delimiter: ",".to_string(),
///        learning_index: 0,
///        first_index: 1,
///        learning_regex: None,
///        first_regex: None,
///    }
/// ];
/// let translations_map = load_translations(Some(configs));
/// ```
///
/// This example configuration demonstrates how to specify an XML and a CSV file from which to load translations. The function
/// will parse these files according to the provided configurations, aggregating all translations into a single `HashMap`.
///
/// # Error Handling
///
/// Errors encountered while reading or parsing files (e.g., file not found, malformed content) will result in those specific translations
/// not being included in the returned `HashMap`. However, such errors will not halt the execution of `load_translations`; instead, the function
/// attempts to process each configured file and aggregates as many translations as possible.
pub fn load_translations(
    translation_configs: Option<Vec<TranslationsConfig>>,
) -> HashMap<String, String> {
    let mut translation_map: HashMap<String, String> = HashMap::new();

    if translation_configs.is_some() {
        for config in translation_configs.unwrap() {
            if let Ok(map) = find_first_lang_translations(&config) {
                for (key, value) in map {
                    translation_map.entry(key).or_insert(value);
                }
            }
        }
    }

    translation_map
}

/// Imports vocabulary data into the database, with translations provided through external files.
///
/// This high-level function orchestrates the process of loading vocabulary items from a JSON file,
/// translating them according to configurations specified in external CSV or XML files, and
/// integrating the translated vocabulary into the database. The operation leverages configurations
/// defined in `vocab_config` and `translation_configs` for processing details.
///
/// # Arguments
///
/// * `vocab_config` - A reference to a `VocabConfig` struct that details the configurations necessary for
/// parsing the vocabulary JSON file and processing vocabulary items. It includes fields such as:
///   - `vocab_json_file_name`: The path to the JSON file containing vocabulary data to be imported.
///   - `plural_suffix`: A suffix indicating plural forms in the learning language, used for processing.
///   - `non_verb_matching_suffixes`: A comma-separated list of suffixes to identify and process similar
///     words of different genders or plural forms in the learning language.
///
/// * `translation_configs` - An optional vector of `TranslationsConfig` instances, each specifying a configuration
/// for loading translations from external sources (CSV or XML files). Each configuration defines how to parse
/// translations, providing details such as file paths, header information, delimiters for CSV files, and regular
/// expressions for extracting terms from XML files.
///
/// # Returns
///
/// Returns `Ok(())` if the import, translation, and database integration process completes successfully.
/// Returns `Err(String)` with an error description if any step in the process encounters a problem, such
/// as issues with reading files, parsing data, or updating the database.
///
/// # Example Usage
///
/// Assuming proper setup and availability of the necessary config JSON files and vocabulary/translation sources:
///
/// ```ignore
/// let vocab_config = VocabConfig {
///     vocab_json_file_name: "data/vocab.json".to_string(),
///     plural_suffix: "s".to_string(),
///     non_verb_matching_suffixes: "o,a,os,as,e,es".to_string(),
///     // Other fields...
/// };
///
/// let translation_configs: Option<Vec<TranslationsConfig>> = Some(vec![
///     TranslationsConfig {
///         file_name: "data/mananoreboton/short-es-en.xml".to_string(),
///         header_lines: 4,
///         delimiter: "".to_string(),
///         learning_index: 0,
///         first_index: 0,
///         learning_regex: "<c>([^<]+)</c>".to_string(),
///         first_regex: "<d>([^<]+)</d>".to_string(),
///         // Other fields...
///     },
///     TranslationsConfig {
///         file_name: "data/llm_import.csv".to_string(),
///         header_lines: 1,
///         delimiter: ",".to_string(),
///         learning_index: 0,
///         first_index: 1,
///         learning_regex: None,
///         first_regex: None,
///         // Other fields...
///     }
/// ]);
///
/// match import_duo_vocab(&vocab_config, translation_configs) {
///     Ok(_) => println!("Vocabulary import and processing completed successfully."),
///     Err(e) => println!("Failed during vocabulary import and processing: {}", e),
/// }
/// ```
pub fn import_duo_vocab(
    vocab_config: &VocabConfig,
    translation_configs: Option<Vec<TranslationsConfig>>,
) -> Result<(), String> {
    let vocab = load_vocab_from_json(&vocab_config.vocab_json_file_name)?;
    let translation_map = load_translations(translation_configs);
    process_duo_vocab(vocab_config, &vocab, &translation_map)
}

/// Exports translation pairs with missing "first language" fields to a CSV file.
///
/// This function queries the database for translation pairs lacking "first language" information
/// and writes the results to a specified CSV file. Each row in the CSV file contains the learning language,
/// infinitive form (if available), and part of speech (if available) for each translation pair.
/// The CSV file is created with this header: `learning, infinitive, pos\n`
///
/// # Parameters
/// - `file_path: &str` - The path to the file where the CSV will be written. The file must not already exist.
///
/// # Returns
/// A `Result<(), Box<dyn Error>>` indicating the outcome of the operation:
/// - `Ok(())` on success.
/// - An error boxed in `Box<dyn Error>` on failure, which could arise from issues such as database access errors,
/// file IO errors, or data serialization problems.
///
/// # Example
///
/// See integration test `tests/export_first_lang_missing_test.rs`
pub fn export_missing_first_lang_pairs(file_path: &str) -> Result<(), Box<dyn Error>> {
    // Get the dal repo for translation pairs. It requires a database connection.
    let pair_repo = DbTranslationPairRepository;
    // Find all the pairs with missing first language fields.
    let pairs = pair_repo.get_empty_first_lang_pairs(i64::MAX)?;

    write_missing_first_export(file_path, pairs)?;

    Ok(())
}
