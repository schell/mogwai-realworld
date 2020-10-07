use serde::{de::DeserializeOwned, Serialize};
use serde_json;
use snafu::{OptionExt, ResultExt, Snafu};
use web_sys::Storage;

/// An enumeration of all storage errors.
#[derive(Debug, Snafu)]
pub enum Error {
    SecurityViolation,
    CantSerialize { source: serde_json::Error },
    CantDeserialize { source: serde_json::Error },
    NoLocalStorage,
    CantStore,
    CantGetKey,
    NoSuchKey,
}

pub fn write_item<T: Serialize>(key: &str, item: &T) -> Result<(), Error> {
    let str_value = serde_json::to_string(item).with_context(|| CantSerialize)?;
    let storage: Storage = mogwai::utils::window()
        .local_storage()
        .ok()
        .with_context(|| NoLocalStorage)?
        .with_context(|| SecurityViolation)?;
    storage
        .set_item(key, &str_value)
        .ok()
        .with_context(|| CantStore)
}

pub fn read_item<T: DeserializeOwned>(key: &str) -> Result<T, Error> {
    let storage: Storage = mogwai::utils::window()
        .local_storage()
        .ok()
        .with_context(|| NoLocalStorage)?
        .with_context(|| SecurityViolation)?;

    let item_str: String = storage
        .get_item(key)
        .ok()
        .with_context(|| CantGetKey)?
        .with_context(|| NoSuchKey)?;

    serde_json::from_str(&item_str).with_context(|| CantDeserialize)
}
