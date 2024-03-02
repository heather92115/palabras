use crate::config::TranslationsConfig;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Lines, Write};
use std::iter::Skip;
use std::string::ToString;
use crate::models::TranslationPair;

pub fn find_first_lang_translations(
    config: &TranslationsConfig,
) -> Result<HashMap<String, String>, String> {
    let buf_reader = load_buffer_from_file(&config.file_name)?;

    // Skip the header
    let mut lines = buf_reader.lines().skip(config.header_lines);

    if config.learning_regex.is_some() && config.first_regex.is_some() {
        find_with_pattern(&mut lines, config)
    } else {
        find_with_splitter(&mut lines, config)
    }
}

pub fn find_with_pattern(
    lines: &mut Skip<Lines<BufReader<File>>>,
    config: &TranslationsConfig,
) -> Result<HashMap<String, String>, String> {
    let learning_regex = Regex::new(
        config
            .learning_regex
            .as_ref()
            .ok_or("Learning regex is required")?,
    )
    .map_err(|e| e.to_string())?;

    let first_regex = Regex::new(
        config
            .first_regex
            .as_ref()
            .ok_or("First Lang regex is required")?,
    )
    .map_err(|e| e.to_string())?;

    let mut translation_map: HashMap<String, String> = HashMap::new();

    // Temporary storage for unmatched captures
    let mut pending_learning: Option<String> = None;
    let mut pending_first: Option<String> = None;

    for line_result in lines {
        let line = line_result.map_err(|e| e.to_string())?;
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        pending_learning = capture_phrase(pending_learning.clone(), &learning_regex, trimmed_line);
        pending_first = capture_phrase(pending_first.clone(), &first_regex, trimmed_line);

        // Check if both captures are found, either on this line or from previous lines
        match (&pending_learning, &pending_first) {
            (Some(learning), Some(first)) => {
                translation_map.insert(learning.clone(), first.clone());

                // Reset pending captures after successful match
                pending_learning = None;
                pending_first = None;
            }
            _ => {} // Do nothing if either or both captures are still pending
        }
    }

    Ok(translation_map)
}

fn capture_phrase(pending: Option<String>, r: &Regex, line: &str) -> Option<String> {
    // Short circuit if we already have some value.
    if pending.is_some() {
        return pending;
    }

    r.captures(&line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

pub fn find_with_splitter(
    lines: &mut Skip<Lines<BufReader<File>>>,
    config: &TranslationsConfig,
) -> Result<HashMap<String, String>, String> {
    if config.learning_index.eq(&config.first_index) {
        return Err(format!("Indices are both {}.", config.learning_index));
    }

    let mut translation_map: HashMap<String, String> = HashMap::new();

    while let Some(Ok(line)) = lines.next() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = if config.delimiter.is_empty() {
            line.split_whitespace().collect()
        } else {
            line.split(&config.delimiter).collect()
        };

        if fields.len() < config.learning_index || fields.len() < config.first_index {
            return Err(format!(
                "Found {} fields, but learning_index {} or first_index {} is out of range.",
                fields.len(),
                config.learning_index,
                config.first_index
            ));
        }

        translation_map
            .entry(fields[config.learning_index].to_string())
            .or_insert(fields[config.first_index].to_string());
    }

    Ok(translation_map)
}

static CSV_HEADER: &str = "learning, infinitive, pos\n";
pub fn write_missing_first_export(file_path: &str, pairs: Vec<TranslationPair>)
    -> Result<(), Box<dyn Error>> {

    let mut buf_writer = open_writing_file_buffer(file_path)?;
    buf_writer.write(CSV_HEADER.as_ref())?;

    pairs.iter().try_for_each(|pair| -> io::Result<()> {
        let out_line = format!(
            "{},{},{}\n",
            pair.learning_lang,
            pair.infinitive.as_deref().unwrap_or_default(),
            pair.pos.as_deref().unwrap_or_default()
        );

        buf_writer.write_all(out_line.as_bytes())
    })?;

    buf_writer.flush()?;

    Ok(())
}

/// Loads a file into a `BufReader` for efficient reading.
///
/// This function opens a file specified by `file_name` and wraps it in a `BufReader`.
/// `BufReader` provides an efficient reading interface by buffering input, which can
/// significantly improve performance when reading from files line by line or in small chunks.
///
/// # Parameters
///
/// * `file_name` - A string slice that holds the path to the file to be opened.
///
/// # Returns
///
/// If successful, this function returns a `Result` wrapping a `BufReader<File>`, allowing
/// for buffered reading operations on the opened file. If an error occurs while opening the
/// file, it returns an error message wrapped in a `Result`.
///
/// # Errors
///
/// This function returns an error if the file specified by `file_name` cannot be opened.
/// This can happen for various reasons, including but not limited to:
/// - The file does not exist.
/// - The current user lacks the necessary permissions to read the file.
/// - There are system-level errors in accessing the file (e.g., issues with the file system).
///
/// The error is returned as a `String` describing the nature of the failure.
///
/// # Example
///
/// ```rust
/// use std::io::{self, BufRead};
///
/// # fn main() -> Result<(), String> {
/// use palabras::dal::file_access::load_buffer_from_file;
///
/// let file_name = "tests/data/test_file.txt";
/// let reader = load_buffer_from_file(file_name)?;
///
/// for line in reader.lines() {
///     let line = line.map_err(|err| err.to_string())?;
///     println!("{}", line);
/// }
/// # Ok(())
/// # }
/// ```
pub fn load_buffer_from_file(file_name: &str) -> Result<BufReader<File>, String> {
    let file = File::open(file_name).map_err(|err| err.to_string())?;
    Ok(BufReader::new(file))
}

pub fn open_writing_file_buffer(path: &str) -> Result<BufWriter<File>, String> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true) // This ensures the file is created only if it does not exist
        .open(path)
        .map_err(|err| err.to_string())?;

    Ok(BufWriter::new(file))
}
