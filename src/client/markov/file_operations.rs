use super::{
    global_data::{MARKOV_BLACKLISTED_USERS_PATH, MARKOV_DATA_SET_PATH, MARKOV_EXPORT_PATH},
    markov_chain::filter_string_for_markov_file,
};
use crate::client::file_operations::create_file_if_missing;
use dashmap::DashSet;
use markov_strings::{ImportExport, InputData};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
};

/// Append a sentence to the markov file
pub fn append_to_markov_file(str: &str) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(MARKOV_DATA_SET_PATH)?;

    Ok(writeln!(file, "{}\n", str)?)
}

/// If the way that messages are filtered before being added to the data set is changed then
/// it's helpful to call this function when the bot starts so the filtering is consistent across the file.
#[allow(dead_code)]
pub fn clean_markov_file() {
    let file = fs::read_to_string(MARKOV_DATA_SET_PATH)
        .expect("Something went wrong while reading the file.");
    let messages = file.split("\n\n").collect::<Vec<&str>>();

    fs::write(MARKOV_DATA_SET_PATH, "").expect("Something went wrong while writing to file.");

    let filtered_messages: Vec<String> = messages
        .into_par_iter()
        .map(filter_string_for_markov_file)
        .collect();

    for message in filtered_messages {
        if let Err(why) = append_to_markov_file(&message) {
            eprintln!("{why}");
        }
    }
}

#[allow(dead_code)]
pub fn export_to_markov_file(export: &ImportExport) -> Result<(), std::io::Error> {
    fs::write(
        MARKOV_EXPORT_PATH,
        serde_json::to_string(&export).expect("Serialization failed"),
    )
}

/// Reads the Markov data set from [`MARKOV_DATA_SET_PATH`]
pub fn import_chain_from_file() -> Result<Vec<InputData>, Box<dyn Error>> {
    let text_from_file = fs::read_to_string(create_file_if_missing(MARKOV_DATA_SET_PATH, "")?)?;
    let text_array: Vec<&str> = text_from_file.split("\n\n").collect();
    Ok(text_array
        .into_par_iter()
        .map(|message| InputData {
            text: message.to_owned(),
            meta: None,
        })
        .collect())
}

pub fn save_markov_blacklisted_users(
    blacklisted_users: &DashSet<u64>,
) -> Result<(), std::io::Error> {
    fs::write(
        MARKOV_BLACKLISTED_USERS_PATH,
        serde_json::to_string(blacklisted_users).expect("Serialization failed"),
    )
}
