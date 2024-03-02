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
    strength_bars: u8,
    infinitive: Option<String>,
    normalized_string: String,
    pos: Option<String>,
    last_practiced_ms: u64,
    skill: String,
    related_lexemes: Vec<String>,
    last_practiced: Option<String>,
    strength: f64,
    skill_url_title: Option<String>,
    gender: Option<String>,
    id: String,
    lexeme_id: String,
    word_string: String,
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

fn merge(
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

fn create(item: &VocabOverview, first: String) -> Result<(), String> {
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

/// Attempt to combine similar words to keep from repeating the same word
/// in different genders or plural forms.
fn find_similar(
    non_verb_matching_suffixes: &str,
    learning_lang: &str,
) -> Result<Option<TranslationPair>, DieselError> {
    let pair_repo = DbTranslationPairRepository;

    let learning = learning_lang.to_lowercase();

    // Find the original suffix and proceed if found.
    if let Some(ori_suffix) = non_verb_matching_suffixes
        .split(',')
        .find(|suffix| learning.ends_with(suffix))
    {
        // Iterate over all suffixes, looking for alternatives.
        for alt_suffix in non_verb_matching_suffixes.split(',') {
            // Skip the original suffix to avoid redundant checks.
            if alt_suffix == ori_suffix {
                continue;
            }

            // Construct the alternative word by replacing the original suffix with the alternative suffix.
            if let Some(stem) = learning.strip_suffix(ori_suffix) {
                let alt_word = format!("{}{}", stem, alt_suffix);

                // Attempt to find a translation pair with the alternative word.
                if let Ok(Some(translation_pair)) =
                    pair_repo.find_translation_pair_by_learning_language(alt_word)
                {
                    return Ok(Some(translation_pair));
                }
            }
        }
    }

    Ok(None)
}

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
