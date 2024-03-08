use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::dal::file_access::load_buffer_from_file;

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
    pub infinitive: Option<String>,
    #[serde(default)]
    normalized_string: String,
    pub pos: Option<String>,
    #[serde(default)]
    last_practiced_ms: u64,
    #[serde(default)]
    pub skill: String,
    #[serde(default = "default_related_lexemes")]
    related_lexemes: Vec<String>,
    #[serde(default)]
    pub last_practiced: Option<String>,
    pub strength: f64,
    skill_url_title: Option<String>,
    pub gender: Option<String>,
    #[serde(default)]
    id: String,
    #[serde(default)]
    lexeme_id: String,
    pub word_string: String,
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
/// use palabras::sl::duo_import::load_vocab_from_json;
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
