use serde::{de::DeserializeOwned, Serialize};
use serde_json;
use snafu::{OptionExt, ResultExt, Snafu};
use web_sys::Storage;

use crate::api::User;

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

fn write_item<T: Serialize>(key: &str, item: &T) -> Result<(), Error> {
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

fn read_item<T: DeserializeOwned>(key: &str) -> Result<T, Error> {
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

fn remove_item(key: &str) -> Result<(),Error> {
    let storage: Storage = mogwai::utils::window()
        .local_storage()
        .ok()
        .with_context(|| NoLocalStorage)?
        .with_context(|| SecurityViolation)?;

    storage
        .remove_item(key)
        .ok()
        .with_context(|| CantGetKey)
}

pub fn write_user(user: &User) -> Result<(), Error> {
    write_item("user", user)
}

pub fn read_user() -> Result<User, Error> {
    read_item("user")
}

pub fn delete_user() -> Result<(), Error> {
    remove_item("user")
}
