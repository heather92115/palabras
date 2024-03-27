use crate::dal::file_access::load_buffer_from_file;
use serde::Deserialize;

/// Configuration for Duolingo vocabulary import.
///
/// This struct defines the configuration needed to import vocabulary from a Duolingo JSON file.
/// It includes details such as the file name of the Duolingo vocabulary JSON,
/// optional suffixes for plural forms, and suffixes for matching non-verbs.
///
/// # Fields
///
/// - `duo_vocab_json_file_name`: The file name of the Duolingo vocabulary JSON to be imported.
/// - `plural_suffix`: An optional string specifying the suffix used to identify plural forms of words. This reduces redundant words.
/// - `non_verb_matching_suffixes`: An optional string specifying suffixes used for matching non-verbs. This reduces redundant words.
///
/// # Example
///
/// ```
/// // This config will attempt combine similar words
/// use palabras::config::VocabConfig;
/// let config = VocabConfig {
///     plural_suffix: Some("s".to_string()),
///     non_verb_matching_suffixes: Some("o,a,os,as,e,es".to_string()),
///     pronouns: None
/// };
///
/// // This config will not combine similar words
/// let config = VocabConfig {
///     plural_suffix: None,
///     non_verb_matching_suffixes: None,
///     pronouns: None
/// };
/// ```
///
/// This example demonstrates how to create and serialize a `DuoVocabConfig` struct. It outlines
/// the basic setup required to import vocabulary from a Duolingo JSON file, with optional
/// configurations for handling plural forms and non-verb word suffixes.

#[derive(Deserialize)]
pub struct Pronoun {
    pub name: String,
    pub instances: String,
}

#[derive(Deserialize)]
pub struct VocabConfig {
    pub plural_suffix: Option<String>,
    pub non_verb_matching_suffixes: Option<String>,
    pub pronouns: Option<Vec<Pronoun>>,
}

static VOCAB_CONFIG_FILENAME: &str = "vocab_config.json";

/// Loads the Duolingo vocabulary configuration from a JSON file.
///
/// This function reads the Duolingo vocabulary configuration from a file named `vocab_config.json`
/// located in the project's root directory. The configuration specifies how vocabulary data
/// from Duolingo should be processed, including handling of plural forms and non-verb matching suffixes.
///
/// # Returns
///
/// Returns a `Result` containing either:
/// - `Ok(VocabConfig)`: The loaded `VocabConfig` instance if the file was successfully read and deserialized.
/// - `Err(String)`: An error message string if there was an issue loading or parsing the configuration file.
///
/// # Errors
///
/// This function can return an error if:
/// - The `vocab_config.json` file does not exist.
/// - There is an issue reading the file.
/// - The JSON data in the file does not match the `VocabConfig` structure.
///
/// This example shows how to call `load_vocab_config` to load the Duolingo vocabulary configuration.
/// It handles the result by printing the loaded configuration on success or an error message on failure.
pub fn load_vocab_config() -> Result<VocabConfig, String> {
    // Deserialize into config list
    let reader = load_buffer_from_file(VOCAB_CONFIG_FILENAME).map_err(|err| err.to_string())?;
    let configs: VocabConfig = serde_json::from_reader(reader).map_err(|err| err.to_string())?;

    Ok(configs)
}

/// Configuration for loading translation pairs from various file formats.
///
/// This struct encapsulates the necessary details to correctly parse and extract translation
/// pairs from files. It supports both delimited text files and files where regular expressions
/// are needed to extract translations.
///
/// # Fields
///
/// - `file_name`: The path to the file containing translation data.
/// - `header_lines`: The number of lines at the beginning of the file to skip, typically used to ignore headers.
/// - `delimiter`: The character or string used to separate fields in the file. Relevant for CSV or similar formats.
/// - `learning_index`: The index (starting from 0) of the column containing the learning language words in a delimited file.
/// - `first_index`: The index (starting from 0) of the column containing the primary language (translation) words in a delimited file.
/// - `learning_regex`: An optional regular expression pattern used to extract the learning language words from non-delimited files.
/// - `first_regex`: An optional regular expression pattern used to extract the primary language (translation) words from non-delimited files.
///
/// # Example
///
/// ```
/// use palabras::config::TranslationsConfig;
///
///
/// // Example of a TranslationsConfig for a CSV file
/// let csv_config = TranslationsConfig {
///     file_name: "translations.csv".to_string(),
///     header_lines: 1,
///     delimiter: ",".to_string(),
///     learning_index: 0,
///     first_index: 1,
///     learning_regex: None,
///     first_regex: None,
/// };
///
/// // Example of a TranslationsConfig for a file requiring regex extraction
/// let regex_config = TranslationsConfig {
///     file_name: "translations.html".to_string(),
///     header_lines: 0,
///     delimiter: "".to_string(), // Delimiter is not used
///     learning_index: 0, // Not used in regex extraction
///     first_index: 0, // Not used in regex extraction
///     learning_regex: Some("<span class='learning'>\\s*(.+?)\\s*</span>".to_string()),
///     first_regex: Some("<span class='first'>\\s*(.+?)\\s*</span>".to_string()),
/// };
/// ```
///
/// This struct is designed to be flexible, allowing for the configuration of both simple delimited
/// files and more complex structured files requiring regular expressions for data extraction.
#[derive(Deserialize)]
pub struct TranslationsConfig {
    pub file_name: String,
    pub header_lines: usize,
    pub delimiter: String,
    pub learning_index: usize,
    pub first_index: usize,
    pub learning_regex: Option<String>,
    pub first_regex: Option<String>,
}

impl Default for TranslationsConfig {
    fn default() -> Self {
        Self {
            file_name: Default::default(),
            header_lines: Default::default(),
            delimiter: Default::default(),
            learning_index: Default::default(),
            first_index: Default::default(),
            learning_regex: None,
            first_regex: None,
        }
    }
}

static TRANSLATIONS_CONFIG_FILENAME: &str = "translations_config.json";

/// Loads translation configurations from a JSON file.
///
/// This function reads a JSON file specified by `TRANSLATIONS_CONFIG_FILENAME`, which contains
/// an array of `TranslationsConfig` objects. Each `TranslationsConfig` object defines how to
/// parse and load translation pairs from a specific file. This allows for loading multiple
/// translation data sources with varying formats.
///
/// # Returns
///
/// A `Result` wrapping a vector of `TranslationsConfig` if successful, or an error message string if an error occurs.
///
/// # Errors
///
/// Errors can occur due to:
/// - File not found or inaccessible.
/// - Issues reading the file.
/// - JSON parsing errors.
///
/// # Example
///
/// Assuming a JSON file named `translations_config.json` exists in the same directory and is
/// correctly formatted, you can load the configurations as follows:
///
///
/// This example demonstrates basic usage of `load_translations_config` to load translation
/// configurations and iterate over them. Each configuration can then be used to
/// load translation pairs from its respective file.
///
/// Note: Ensure the `translations_config.json` file is correctly formatted and accessible at the
/// path specified by `TRANSLATIONS_CONFIG_FILENAME`.
pub fn load_translations_config() -> Result<Option<Vec<TranslationsConfig>>, String> {
    // Deserialize into config list
    let reader =
        load_buffer_from_file(TRANSLATIONS_CONFIG_FILENAME).map_err(|err| err.to_string())?;
    let configs: Vec<TranslationsConfig> =
        serde_json::from_reader(reader).map_err(|err| err.to_string())?;

    Ok(Some(configs))
}
