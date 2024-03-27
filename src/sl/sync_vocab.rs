use crate::config::{TranslationsConfig, VocabConfig};
use crate::dal::awesome_person::{AwesomePersonRepository, DbAwesomePersonRepository};
use crate::dal::file_access::{find_first_lang_translations, write_missing_first_export};
use crate::dal::vocab::{DbVocabRepository, VocabRepository};
use crate::dal::vocab_study::{DbVocabStudyRepository, VocabStudyRepository};
use crate::models::{AwesomePerson, NewVocabStudy, Vocab};
use crate::sl::fuzzy_match_vocab::WELL_KNOWN_THRESHOLD;
use diesel::result::Error as DieselError;
use std::collections::HashMap;
use std::error::Error;

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
pub fn merge_learning(current: &mut Vocab, additional_learning: String, plural_suffix: &str) {
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

    vocab_study_repo.create_vocab_study(&new_vocab_study)?;

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
fn _find_similar(
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
                if let Ok(Some(vocab)) = vocab_repo.find_vocab_by_learning_language(alt_word) {
                    return Ok(Some(vocab)); // Found a similar word form, return it
                }
            }
        }
    }

    Ok(None)
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

    let awesome_person = repo.get_awesome_person_by_id(awesome_person_id)?;

    if awesome_person.is_none() {
        Err(format!(
            "No Awesome person was found with id {}",
            awesome_person_id
        ))
    } else {
        Ok(awesome_person.unwrap())
    }
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
