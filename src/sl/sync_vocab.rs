use crate::config::{TranslationsConfig, VocabConfig};
use crate::dal::file_access::{
    find_first_lang_translations, write_missing_first_export,
};
use crate::dal::vocab::{DbVocabRepository, VocabRepository};
use crate::dal::vocab_study::{DbVocabStudyRepository, VocabStudyRepository};
use crate::models::{AwesomePerson, NewVocab, NewVocabStudy, Vocab, VocabStudy};
use crate::sl::fuzzy_match_vocab::{WELL_KNOWN_THRESHOLD};
use diesel::result::Error as DieselError;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;
use crate::dal::awesome_person::{AwesomePersonRepository, DbAwesomePersonRepository};
use crate::sl::duo_import::{LanguageData, load_vocab_from_json, VocabOverview};

/// Determines hints for a given phrase by analyzing its length and the presence of specific pronouns.
///
/// This function splits the input phrase into words, counts them, and searches for any specified pronouns
/// within the phrase. If the phrase contains more than one word, it returns a hint consisting of the word count
/// and the names of any pronoun categories found. This can help identify the grammatical structure or complexity
/// of the phrase.
///
/// # Parameters
/// - `vocab_config: &VocabConfig` - Configuration containing pronoun information.
/// - `learning: &str` - The learning phrase to be analyzed.
///
/// # Returns
/// An `Option<String>` that contains a hint if the phrase has more than one word. The hint includes the word count
/// and names of any matching pronoun categories found in the phrase. Returns `None` if the phrase consists of a single word.
///
/// # Examples
/// ```
///
/// use palabras::config::{Pronoun, VocabConfig};
/// use palabras::sl::sync_vocab::determine_hint;
///
/// let vocab_config = VocabConfig {
///     vocab_json_file_name: "data/vocab.json".to_string(),
///     plural_suffix: Some("s".to_string()),
///     non_verb_matching_suffixes: Some("o,a,os,as,e,es".to_string()),
///     pronouns: Some(vec![
///         Pronoun {
///             name: "reflexive pronoun".to_string(),
///             instances: "me, te, se, nos, os".to_string(),
///         },
///         // Additional pronouns not shown for brevity
///     ]),
/// };
///
/// let learning_phrase = "se acuerdan";
/// let (hint, num_words) = determine_hint(&vocab_config, &learning_phrase);
/// let hint = hint.unwrap_or_default();
/// assert_eq!(hint, "phrase, reflexive pronoun");
/// assert_eq!(num_words, 2);
/// ```
/// This example demonstrates how `determine_hint` generates a hint for the phrase "tú y yo", indicating that it contains
/// two words and matches the "subject pronoun" category.
pub fn determine_hint(vocab_config: &VocabConfig, learning: &str) -> (Option<String>, i32) {
    let binding = learning.to_lowercase();
    let words: Vec<&str> = binding.split_whitespace().collect();
    let num_words = words.len() as i32;

    if num_words > 1 {
        let mut hint = "phrase".to_string();

        if let Some(pronouns) = &vocab_config.pronouns {
            for pronoun in pronouns {
                for instance in pronoun.instances.split(", ") {
                    if words.contains(&instance) {
                        hint = format!("{}, {}", hint, pronoun.name);
                    }
                }
            }
        }
        return (Some(hint), num_words);
    }

    (None, num_words)
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
/// use palabras::models::Vocab;
/// use palabras::sl::sync_vocab::merge_learning;
/// let mut pair = Vocab {
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
    current: &mut Vocab,
    additional_learning: String,
    plural_suffix: &str,
) {
    if current.learning_lang.ne(&additional_learning) {
        // See if the learning lang is in plural form and should be swapped with the new word.
        let (learning, additional) = if current
            .learning_lang
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
/// use palabras::models::Vocab;
/// use palabras::sl::duo_import::VocabOverview;
/// use palabras::sl::sync_vocab::{merge};
/// let vocab_config = VocabConfig {
///     vocab_json_file_name: "".to_string(),
///     plural_suffix: Some("s".to_string()),
///     non_verb_matching_suffixes: None,
///     pronouns: None
/// };
///
/// let mut vocab = Vocab {
///     learning_lang: "gato".to_string(),
///     first_lang: "".to_string(),
///     ..Default::default()
/// };
///
/// let json_str = r#"{"infinitive": null,"pos": "Noun","strength": 0.9999,"gender": "Feminine","word_string": "gatos"}"#;
/// let vocab_overview: VocabOverview =
/// serde_json::from_str(json_str).expect("JSON should deserialize");
///
/// merge(&vocab_config, &mut vocab, &vocab_overview, "cat".to_string());
///
/// assert_eq!(vocab.learning_lang, "gato");
/// assert_eq!(vocab.first_lang, "cat");
/// assert_eq!(vocab.alternatives, Some("gatos".to_string()));
/// ```
pub fn merge(
    vocab_config: &VocabConfig,
    current: &mut Vocab,
    item: &VocabOverview,
    first: String,
) {
    // Determine placement of new learning word or phrase
    merge_learning(
        current,
        item.word_string.clone(),
        &vocab_config.plural_suffix.clone().unwrap_or_default(),
    );

    // Add some help to determine the correct learning lang
    (current.hint, current.num_learning_words) = determine_hint(vocab_config, &current.learning_lang);

    // Update the translation back to the first language if it wasn't already translated
    if current.first_lang.clone().is_empty() {
        current.first_lang = first;
    }

    current.infinitive = Some(item.infinitive.clone().unwrap_or_default());
    current.pos = Some(item.pos.clone().unwrap_or_default());
}

pub fn create_vocab_study(vocab_id: i32, awesome_id: i32, percentage: f64) -> Result<(), String> {

    let vocab_study_repo = DbVocabStudyRepository;

    let new_vocab_study = NewVocabStudy {
        vocab_id,
        awesome_person_id: awesome_id,
        percentage_correct: Some(percentage),
        well_known: percentage > WELL_KNOWN_THRESHOLD,

        // Other fields use their default values
        ..Default::default()
    };

    vocab_study_repo.create_vocab_study(&new_vocab_study)
        .map_err(|err| err.to_string())?;

    Ok(())
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
/// * `vocab_config` - A reference to the configuration settings for vocabulary handling, including pluralization rules. Settings are found at `vocab_config.json`
/// * `item` - A reference to a `VocabOverview` instance containing data for the learning language side
/// of the translation pair.
/// * `first` - A `String` representing the translation of the item in the first language.
/// * `awesome_person_id` - Primary key for the awesome person table. The current user's id.
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
pub fn create(
    vocab_config: &VocabConfig,
    item: &VocabOverview,
    first: String,
    awesome_person_id: i32,
    kn_lang: String,
    ln_lang: String
) -> Result<(), String> {
    // Get the dal repo for translation pairs. It requires a database connection.
    let vocab_repo = DbVocabRepository;
    let learning_lang = item.word_string.clone();
    let (hint, num_learning_words) = determine_hint(vocab_config, &learning_lang);

    let new_vocab = NewVocab {
        first_lang: first,
        learning_lang,

        skill: Some(item.skill.clone()),
        infinitive: Some(item.infinitive.clone().unwrap_or_default()),
        pos: Some(item.pos.clone().unwrap_or_default()),
        hint,
        num_learning_words,
        known_lang_code: kn_lang,
        learning_lang_code: ln_lang,
        // Other fields use their default values
        ..Default::default()
    };

    let vocab = vocab_repo
        .create_vocab(&new_vocab)
        .map_err(|err| err.to_string())?;

    // Now create the matching vocab study for this user, since the vocab is new
    // it cannot already exist.
    create_vocab_study(vocab.id, awesome_person_id, item.strength)
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
/// // If "gato, gata or gatas" is in the database, its vocab will be found and returned.
/// let non_verb_suffixes = "o,a,os,as,e,es";
/// let learning_lang_word = "gatos";
///
/// match find_similar(non_verb_suffixes, learning_lang_word) {
///     Ok(Some(vocab)) => {
///         println!("Found a similar word: {}", vocab.learning_lang);
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
) -> Result<Option<Vocab>, DieselError> {
    let vocab_repo = DbVocabRepository;

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
            if let Some(stem) = learning.strip_suffix(ori_suffix) {
                // ex: gato becomes gat
                let alt_word = format!("{}{}", stem, alt_suffix); // ex: gat becomes gata, gatos, gatas

                // Attempt to search for a translation pair using the newly contructed alternative
                if let Ok(Some(vocab)) =
                    vocab_repo.find_vocab_by_learning_language(alt_word)
                {
                    return Ok(Some(vocab)); // Found a similar word form, return it
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
/// match find_vocab(&vocab_config, &item) {
///     Ok(Some(vocab)) => {
///         println!("Found a matching translation pair: {:?}", vocab);
///     },
///     Ok(None) => {
///         println!("No matching translation pair found.");
///     },
///     Err(e) => {
///         println!("An error occurred while searching: {}", e);
///     }
/// }
/// ```
pub fn find_vocab(
    vocab_config: &VocabConfig,
    item: &VocabOverview,
) -> Result<Option<Vocab>, String> {
    let vocab_repo = DbVocabRepository;

    // First, try finding the translation pair by the learning language directly.
    if let Ok(Some(current)) =
        vocab_repo.find_vocab_by_learning_language(item.word_string.clone())
    {
        return Ok(Some(current));
    }

    // Second, try finding if it's already an alternative
    if let Ok(Some(current)) =
        vocab_repo.find_vocab_by_alternative(item.word_string.clone())
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

pub fn sync_vocab_study(vocab_id: i32, awesome_person_id: i32, percentage: f64) -> Result<(), String> {

    let vocab_study_repo = DbVocabStudyRepository;

    if let Some(vocab_study) = vocab_study_repo
        .get_vocab_study_by_foreign_refs(vocab_id, awesome_person_id)
        .map_err(|e| e.to_string())? {

        // Update the strength if it is higher
        if percentage > vocab_study.percentage_correct.unwrap_or_default() {

            let updating = VocabStudy {
                percentage_correct: Some(percentage),
                well_known: percentage > WELL_KNOWN_THRESHOLD,

                ..vocab_study // Keep the rest of the values the same
            };

            vocab_study_repo
                .update_vocab_study(updating)
                .map_err(|err| err.to_string())?;
        }
    } else {
        // This user doesn't already have a mapping to the vocab, so it
        // needs to be added.
        create_vocab_study(vocab_id, awesome_person_id, percentage)
            .map_err(|err| err.to_string())?;
    }

    Ok(())
}

// Creating a mutex to guard the complex logic within process_duo_vocab
lazy_static! {
    static ref PROCESS_IMPORT_MUTEX: Mutex<()> = Mutex::new(());
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
/// * `awesome_id` - Primary key for the awesome person table. The current user's id.
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
/// - Any of the utilized subroutines (`find_vocab`, `merge`, or `create`) encounter errors in their execution.
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
    awesome_id: i32
) -> Result<(), String> {
    // Acquire the lock before proceeding with import and updates.
    let _lock = PROCESS_IMPORT_MUTEX.lock().map_err(|e| e.to_string())?;

    // Get the dal repo for translation pairs. It requires a database connection.
    let vocab_repo = DbVocabRepository;

    for item in &vocabulary_overview.vocab_overview {
        let learning_lang = &item.word_string;

        // Fetch the translated first language if available
        let translated_first = translation_map
            .get(learning_lang)
            .cloned()
            .unwrap_or_default();

        // Attempt to find an existing translation pair by learning language.
        match find_vocab(vocab_config, item) {
            Ok(Some(mut current)) => {
                let vocab_id = current.id;
                merge(vocab_config, &mut current, item, translated_first);
                vocab_repo
                    .update_vocab(current)
                    .map_err(|err| err.to_string())?;

                sync_vocab_study(vocab_id, awesome_id, item.strength)
                    .map_err(|err| err.to_string())?;
            }
            Ok(None) => create(
                vocab_config,
                item,
                translated_first,
                awesome_id,
                vocabulary_overview.from_language.clone(),
                vocabulary_overview.learning_language.clone(),
            )?,

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

/// Verifies if an `AwesomePerson` exists by their ID.
///
/// This function searches for an `AwesomePerson` in the database using a given ID. If the `AwesomePerson`
/// is found, it returns `Ok(AwesomePerson)`. If not found, it returns an `Err` with a message indicating
/// no AwesomePerson was found with the given ID.
///
/// # Arguments
///
/// * `awesome_person_id` - The ID of the `AwesomePerson` to verify.
///
/// # Returns
///
/// * `Ok(AwesomePerson)` if the `AwesomePerson` is found.
/// * `Err(String)` if no `AwesomePerson` is found, with a message including the ID.
pub fn verify_awesome_person(awesome_person_id: i32) -> Result<AwesomePerson, String> {

    let repo = DbAwesomePersonRepository;

    let awesome_person
        = repo.get_awesome_person_by_id(awesome_person_id).map_err(|e| e.to_string())?;

    if awesome_person.is_none() {
        Err(format!("No Awesome person was found with id {}", awesome_person_id))
    } else {
        Ok(awesome_person.unwrap())
    }
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
/// * `awesome_id` - Primary key for the awesome person table. The current user's id.
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
    awesome_id: i32,
) -> Result<(), String> {
    let vocab = load_vocab_from_json(&vocab_config.vocab_json_file_name)?;
    let translation_map = load_translations(translation_configs);

    let awesome_person = verify_awesome_person(awesome_id)?;
    process_duo_vocab(vocab_config, &vocab, &translation_map, awesome_person.id)
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
    let vocab_repo = DbVocabRepository;
    // Find all the pairs with missing first language fields.
    let pairs = vocab_repo.get_empty_first_lang(i64::MAX)?;

    write_missing_first_export(file_path, pairs)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config::{Pronoun, VocabConfig};
    use crate::sl::sync_vocab::determine_hint;

    #[derive(Debug)]
    struct HintTestCase {
        learning_phrase: &'static str,
        expected_hint: &'static str,
        expected_length: i32,
    }

    #[test]
    fn unit_test_determine_hint() {
        let vocab_config = VocabConfig {
            vocab_json_file_name: "data/vocab.json".to_string(),
            plural_suffix: Some("s".to_string()),
            non_verb_matching_suffixes: Some("o,a,os,as,e,es".to_string()),
            pronouns: Some(vec![
                Pronoun {
                    name: "subject pronoun".to_string(),
                    instances:
                        "yo, tú, él, ella, nosotros, nosotras, vosotros, vosotras, ellos, ellas"
                            .to_string(),
                },
                Pronoun {
                    name: "formal subject pronoun".to_string(),
                    instances: "usted, ustedes".to_string(),
                },
                Pronoun {
                    name: "reflexive pronoun".to_string(),
                    instances: "me, te, se, nos, os".to_string(),
                },
                Pronoun {
                    name: "object pronoun".to_string(),
                    instances: "lo, la, los, las, le, nos, os, les".to_string(),
                },
            ]),
        };

        let test_cases = vec![
            HintTestCase {
                learning_phrase: "usted hace",
                expected_hint: "phrase, formal subject pronoun",
                expected_length: 2,
            },
            HintTestCase {
                learning_phrase: "yo corro",
                expected_hint: "phrase, subject pronoun",
                expected_length: 2,
            },
            HintTestCase {
                learning_phrase: "ellos juegan en el parque",
                expected_hint: "phrase, subject pronoun",
                expected_length: 5,
            },
            HintTestCase {
                learning_phrase: "cómo se dice",
                expected_hint: "phrase, reflexive pronoun",
                expected_length: 3,
            },
            HintTestCase {
                learning_phrase: "fáciles",
                expected_hint: "",
                expected_length: 1,
            },
            HintTestCase {
                learning_phrase: "especial del día",
                expected_hint: "phrase",
                expected_length: 3,
            },
            HintTestCase {
                learning_phrase: "no lo sé",
                expected_hint: "phrase, object pronoun",
                expected_length: 3,
            },
        ];

        for test_case in test_cases {
            let (hint, num_learning_words) = determine_hint(&vocab_config, test_case.learning_phrase);
            let hint = hint.unwrap_or_default();
            assert_eq!(
                hint, test_case.expected_hint,
                "Failed on learning_phrase: {:?}",
                test_case.learning_phrase
            );
            assert_eq!(
                num_learning_words, test_case.expected_length,
                "Word count mismatch, expected {}, actual {}",
                test_case.expected_length,
                num_learning_words
            );
        }
    }
}
